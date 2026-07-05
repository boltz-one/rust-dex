//! Non-prompt-queue client lifecycle state. Mirrors the subset of acpx's
//! `client.ts` private fields that describe the agent process's lifecycle
//! (`agentStartedAt`, `lastKnownPid`, `lastAgentExit`) rather than
//! prompt/session bookkeeping (`activePrompt`, `cancellingSessionIds`, etc.
//! — those move to Phase 6's queue per ADR-4, not ported here).

use chrono::{DateTime, Utc};

/// Why the agent process exited, and how. Ports the fields acpx attaches to
/// `AgentDisconnectedError`/`getAgentLifecycleSnapshot().lastExit`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentExitInfo {
    pub reason: String,
    pub exit_code: Option<i32>,
    pub signal: Option<String>,
}

/// Lifecycle bookkeeping for one spawned agent process. Deliberately holds
/// no prompt-queue state (see module docs).
#[derive(Debug, Clone)]
pub struct ClientState {
    pub agent_started_at: DateTime<Utc>,
    pub last_known_pid: Option<u32>,
    pub last_agent_exit: Option<AgentExitInfo>,
}

impl ClientState {
    pub fn new(pid: u32) -> Self {
        Self {
            agent_started_at: Utc::now(),
            last_known_pid: Some(pid),
            last_agent_exit: None,
        }
    }

    pub fn record_exit(&mut self, exit: AgentExitInfo) {
        self.last_agent_exit = Some(exit);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_state_has_no_exit_recorded() {
        let state = ClientState::new(1234);
        assert_eq!(state.last_known_pid, Some(1234));
        assert!(state.last_agent_exit.is_none());
    }

    #[test]
    fn record_exit_stores_reason() {
        let mut state = ClientState::new(1);
        state.record_exit(AgentExitInfo {
            reason: "sigterm".to_string(),
            exit_code: None,
            signal: Some("SIGTERM".to_string()),
        });
        assert_eq!(state.last_agent_exit.unwrap().reason, "sigterm");
    }
}
