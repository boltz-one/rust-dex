//! Reconnect state machine. Ports
//! `others/acpx/src/runtime/engine/reconnect.ts`'s `connectAndLoadSession`
//! orchestration (`loadOrCreateRuntimeSession` and its helpers) — the most
//! state-machine-heavy file in the whole scoped port (Risk Assessment #1).
//!
//! ## State machine (Step 6 — designed before transliterating)
//!
//! Rather than acpx's flat sequence of nested `if`/`try`/`catch`, every
//! state acpx implicitly tracks is named here so a reviewer can check each
//! transition is handled, instead of trusting a 400-line imperative
//! function reads them all correctly:
//!
//! ```text
//! entry: caller already confirmed there is no live in-memory
//!        ConnectedSession for this record (see connected_session.rs docs —
//!        unlike acpx's daemon, this crate's host is one continuously
//!        running process, so "already connected" is checked one layer up,
//!        not inside this module).
//!
//! [ChooseAcquisitionPath] -- based on agent_capabilities + same_session_only
//!   -> Resume            (agent advertises session_capabilities.resume)
//!   -> Load              (agent advertises load_session, no resume)
//!   -> RequireSameSession (neither, and same_session_only)
//!   -> CreateFresh        (neither, same_session_only == false)
//!
//! [Resume] / [Load] -- send session/resume or session/load
//!   -> Acquired { resumed: true }                    on success
//!   -> propagate AcpError::Timeout                    hard failure (never falls back)
//!   -> Err(SessionResumeRequired)                     same_session_only == true
//!   -> [CreateFresh] with load_error recorded          fallback-eligible RPC error
//!                                                       (resource-not-found / unsupported
//!                                                       method / query-closed-before-response
//!                                                       or generic internal error on a session
//!                                                       with no agent replies yet)
//!   -> propagate normalized AcpError                   anything else (would silently lose
//!                                                       real conversation history)
//!
//! [RequireSameSession] -> Err(SessionResumeRequired)
//!
//! [CreateFresh] -- session/new
//!   -> Acquired { resumed: false, created_fresh: true }  (no further fallback if this fails)
//!
//! [Acquired { created_fresh: true }] -- only if the record has any desired
//! mode/model/config-option state to replay
//!   -> replay mode, then model, then config options (in that order)
//!   -> Done                                            replay succeeded (or nothing to replay)
//!   -> Err(SessionModeReplay | SessionModelReplay | SessionConfigOptionReplay)
//!      + record rolled back to its pre-attempt acp_session_id/agent_session_id/acpx
//!
//! [Acquired { created_fresh: false }] -> Done            (resumed/loaded session already has
//!                                                          these settings; skip replay)
//! ```

pub mod liveness;
pub mod replay;

use std::path::PathBuf;
use std::time::Duration;

use agent_client_protocol::Error as AcpRpcError;
use agent_client_protocol::schema::v1::{
    AgentCapabilities, LoadSessionRequest, McpServer, ResumeSessionRequest, SessionId,
};

use crate::agent_session_id::extract_agent_session_id;
use crate::client::AcpClient;
use crate::control::with_timeout;
use crate::error::{AcpError, Result};
use crate::error_normalization::{
    is_acp_query_closed_before_response_error, normalize_agent_error,
};
use crate::error_shapes::is_acp_resource_not_found_error;
use crate::session::config_options::apply_config_options_to_record;
use crate::session::conversation_model::SessionConversation;
use crate::session::record::SessionRecord;
use crate::types::SessionResumePolicy;

use replay::{has_preferences_to_replay, replay_fresh_session_preferences};

/// Result of [`connect_and_load_session`].
pub struct ConnectAndLoadSessionResult {
    pub session_id: SessionId,
    pub agent_session_id: Option<String>,
    pub resumed: bool,
    pub load_error: Option<String>,
}

const UNSUPPORTED_SESSION_LOAD_CODES: [i32; 2] = [-32601, -32602];

fn session_resume_required(record: &SessionRecord, reason: &str) -> AcpError {
    AcpError::SessionResumeRequired(format!(
        "Persistent ACP session {} could not be resumed: {reason}",
        record.acp_session_id
    ))
}

/// Ports `shouldFallbackToNewSession`'s RPC-error-classification half (the
/// hard-failure and same-session-only paths are handled by
/// [`acquire_via_rpc`] before this is consulted).
fn should_fallback_to_new_session(error: &AcpRpcError, conversation: &SessionConversation) -> bool {
    let code: i32 = error.code.into();
    if is_acp_resource_not_found_error(error) || UNSUPPORTED_SESSION_LOAD_CODES.contains(&code) {
        return true;
    }
    if crate::runtime::engine::lifecycle::session_has_agent_messages(conversation) {
        return false;
    }
    is_acp_query_closed_before_response_error(error) || code == -32603
}

enum AcquisitionPath {
    Resume,
    Load,
    RequireSameSession,
    CreateFresh,
}

/// Ports the branching at the top of `loadOrCreateRuntimeSession`.
fn choose_acquisition_path(
    capabilities: &AgentCapabilities,
    same_session_only: bool,
) -> AcquisitionPath {
    if capabilities.session_capabilities.resume.is_some() {
        AcquisitionPath::Resume
    } else if capabilities.load_session {
        AcquisitionPath::Load
    } else if same_session_only {
        AcquisitionPath::RequireSameSession
    } else {
        AcquisitionPath::CreateFresh
    }
}

/// Outcome of acquiring a live backend session id, before any preference
/// replay. Ports `RuntimeSessionLoadState`.
struct Acquired {
    session_id: SessionId,
    agent_session_id: Option<String>,
    resumed: bool,
    created_fresh: bool,
    load_error: Option<String>,
}

/// Ports the shared body of `resumeRuntimeSession`/`loadRuntimeSession`:
/// sends the raw request (bypassing [`AcpClient`]'s normalized wrappers so
/// this function can classify the *raw* JSON-RPC code, matching
/// `extractAcpError`'s use in acpx), then either succeeds, falls back to
/// [`create_fresh_session`], or propagates.
async fn acquire_via_rpc(
    client: &AcpClient,
    record: &mut SessionRecord,
    conversation: &SessionConversation,
    mcp_servers: &[McpServer],
    same_session_only: bool,
    timeout: Option<Duration>,
    use_resume: bool,
) -> Result<Acquired> {
    let session_id = SessionId::new(record.acp_session_id.clone());
    let cwd = PathBuf::from(&record.cwd);

    // `ResumeSessionResponse`/`LoadSessionResponse` share the same
    // `modes`/`config_options`/`meta` shape but are distinct types, so each
    // branch normalizes its response into a common `(meta, config_options)`
    // pair immediately rather than trying to unify the two response types
    // at the `if`/`else` boundary.
    let raw_result: std::result::Result<
        (
            Option<agent_client_protocol::schema::v1::Meta>,
            Option<Vec<agent_client_protocol::schema::v1::SessionConfigOption>>,
        ),
        AcpRpcError,
    > = if use_resume {
        let request =
            ResumeSessionRequest::new(session_id.clone(), cwd).mcp_servers(mcp_servers.to_vec());
        let timed = with_timeout(
            client.connection().send_request(request).block_task(),
            timeout,
        )
        .await?;
        timed.map(|response| (response.meta, response.config_options))
    } else {
        let request =
            LoadSessionRequest::new(session_id.clone(), cwd).mcp_servers(mcp_servers.to_vec());
        let timed = with_timeout(
            client.connection().send_request(request).block_task(),
            timeout,
        )
        .await?;
        timed.map(|response| (response.meta, response.config_options))
    };

    match raw_result {
        Ok((meta, config_options)) => {
            let agent_session_id = extract_agent_session_id(meta.as_ref());
            apply_config_options_to_record(record, config_options);
            Ok(Acquired {
                session_id,
                agent_session_id,
                resumed: true,
                created_fresh: false,
                load_error: None,
            })
        }
        Err(rpc_error) => {
            if same_session_only {
                return Err(session_resume_required(
                    record,
                    &format!("{} (ACP {})", rpc_error.message, i32::from(rpc_error.code)),
                ));
            }
            if should_fallback_to_new_session(&rpc_error, conversation) {
                let load_error =
                    format!("{} (ACP {})", rpc_error.message, i32::from(rpc_error.code));
                return create_fresh_session(
                    client,
                    record,
                    mcp_servers,
                    timeout,
                    Some(load_error),
                )
                .await;
            }
            Err(normalize_agent_error(
                rpc_error,
                record.acp_session_id.clone(),
            ))
        }
    }
}

/// Ports `createFreshRuntimeSession`.
async fn create_fresh_session(
    client: &AcpClient,
    record: &mut SessionRecord,
    mcp_servers: &[McpServer],
    timeout: Option<Duration>,
    load_error: Option<String>,
) -> Result<Acquired> {
    let cwd = PathBuf::from(&record.cwd);
    let response = with_timeout(client.session_new(cwd, mcp_servers.to_vec()), timeout).await??;
    let agent_session_id = extract_agent_session_id(response.meta.as_ref());
    apply_config_options_to_record(record, response.config_options.clone());
    Ok(Acquired {
        session_id: response.session_id,
        agent_session_id,
        resumed: false,
        created_fresh: true,
        load_error,
    })
}

async fn acquire_session(
    client: &AcpClient,
    record: &mut SessionRecord,
    conversation: &SessionConversation,
    mcp_servers: &[McpServer],
    same_session_only: bool,
    timeout: Option<Duration>,
) -> Result<Acquired> {
    // No backend session id has ever been assigned yet (a genuinely new
    // record, e.g. the first `ensure_session` call for this key) — there is
    // nothing to resume or load, so always create fresh regardless of
    // capabilities or `same_session_only`. Not present in acpx's own
    // `loadOrCreateRuntimeSession` because acpx's manager never calls this
    // path for a record it just minted itself (it inlines fresh-session
    // creation there instead); this port funnels both cases through the
    // same state machine, so the empty-id guard has to live here.
    if record.acp_session_id.trim().is_empty() {
        return create_fresh_session(client, record, mcp_servers, timeout, None).await;
    }

    let capabilities = client.init_response().agent_capabilities.clone();
    match choose_acquisition_path(&capabilities, same_session_only) {
        AcquisitionPath::Resume => {
            acquire_via_rpc(
                client,
                record,
                conversation,
                mcp_servers,
                same_session_only,
                timeout,
                true,
            )
            .await
        }
        AcquisitionPath::Load => {
            acquire_via_rpc(
                client,
                record,
                conversation,
                mcp_servers,
                same_session_only,
                timeout,
                false,
            )
            .await
        }
        AcquisitionPath::RequireSameSession => Err(session_resume_required(
            record,
            "agent does not support session/resume or session/load",
        )),
        AcquisitionPath::CreateFresh => {
            create_fresh_session(client, record, mcp_servers, timeout, None).await
        }
    }
}

/// Ports `connectAndLoadSession`. `conversation` is the connected session's
/// in-memory conversation model (used only to answer
/// `sessionHasAgentMessages` for the fallback-safety check — the caller
/// still owns writing turn content into it).
pub async fn connect_and_load_session(
    client: &AcpClient,
    record: &mut SessionRecord,
    conversation: &SessionConversation,
    mcp_servers: &[McpServer],
    resume_policy: SessionResumePolicy,
    timeout: Option<Duration>,
) -> Result<ConnectAndLoadSessionResult> {
    let same_session_only = matches!(resume_policy, SessionResumePolicy::SameSessionOnly)
        || record.imported_from.is_some();

    let original_session_id = record.acp_session_id.clone();
    let original_agent_session_id = record.agent_session_id.clone();
    let original_acpx = record.acpx.clone();
    let agent_command = record.agent_command.clone();

    let acquired = acquire_session(
        client,
        record,
        conversation,
        mcp_servers,
        same_session_only,
        timeout,
    )
    .await?;

    record.acp_session_id = acquired.session_id.0.to_string();
    if let Some(agent_session_id) = &acquired.agent_session_id {
        crate::runtime::engine::lifecycle::reconcile_agent_session_id(
            record,
            Some(agent_session_id),
        );
    }

    if acquired.created_fresh && has_preferences_to_replay(record) {
        let replay_result = replay_fresh_session_preferences(
            client,
            acquired.session_id.clone(),
            record,
            &agent_command,
        )
        .await;
        if let Err(err) = replay_result {
            record.acp_session_id = original_session_id;
            record.agent_session_id = original_agent_session_id;
            record.acpx = original_acpx;
            return Err(err);
        }
    }

    Ok(ConnectAndLoadSessionResult {
        session_id: acquired.session_id,
        agent_session_id: record.agent_session_id.clone(),
        resumed: acquired.resumed,
        load_error: acquired.load_error,
    })
}
