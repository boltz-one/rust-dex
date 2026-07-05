//! `fs/read_text_file` / `fs/write_text_file` handlers with cwd-sandboxed
//! path resolution. Ports `others/acpx/src/filesystem.ts`.
//!
//! Security (see phase-03's Security Considerations): the target path is
//! canonicalized *before* the cwd-boundary check, never after — resolving
//! `..`/symlinks post-hoc would let a false "safe" boundary compare succeed
//! against the pre-resolution string. A symlink inside `cwd` pointing
//! outside it is therefore always **rejected** here: this deliberately
//! tightens acpx's Node `fs`-module behavior (acpx only lexically
//! normalizes via `path.resolve`, so Node's `fs.readFile`/`writeFile` would
//! silently follow such a symlink out of the sandbox).
//!
//! acpx's CLI-only `ClientOperation`/`onOperation` event stream (used for
//! `--json` progress output) is not ported — it's CLI/commander surface out
//! of this crate's scope (see plan.md).

use std::path::{Path, PathBuf};

use agent_client_protocol::schema::v1::{
    ReadTextFileRequest, ReadTextFileResponse, WriteTextFileRequest, WriteTextFileResponse,
};

use crate::error::{AcpError, Result};
use crate::permissions::{PermissionRequestHandler, confirm_action};
use crate::types::{NonInteractivePermissionPolicy, PermissionMode};

/// Owns the sandboxed root and the current permission policy for
/// `fs/read_text_file` / `fs/write_text_file`. Ports the state
/// `FileSystemHandlers` in `filesystem.ts` closes over.
pub struct FilesystemHandlers {
    root_dir: PathBuf,
    permission_mode: PermissionMode,
    non_interactive_policy: NonInteractivePermissionPolicy,
    handler: Option<std::sync::Arc<dyn PermissionRequestHandler>>,
}

impl FilesystemHandlers {
    /// `cwd` must already exist (it's the session's working directory);
    /// canonicalized once up front so every later boundary check compares
    /// against a symlink-resolved root.
    pub fn new(
        cwd: impl AsRef<Path>,
        permission_mode: PermissionMode,
        non_interactive_policy: NonInteractivePermissionPolicy,
        handler: Option<std::sync::Arc<dyn PermissionRequestHandler>>,
    ) -> Result<Self> {
        let root_dir = cwd.as_ref().canonicalize().map_err(|source| {
            io_err(
                format!("failed to resolve cwd {}", cwd.as_ref().display()),
                source,
            )
        })?;
        Ok(Self {
            root_dir,
            permission_mode,
            non_interactive_policy,
            handler,
        })
    }

    /// Ports `updatePermissionPolicy`.
    pub fn update_permission_policy(
        &mut self,
        permission_mode: PermissionMode,
        non_interactive_policy: NonInteractivePermissionPolicy,
    ) {
        self.permission_mode = permission_mode;
        self.non_interactive_policy = non_interactive_policy;
    }

    /// Ports `readTextFile`.
    pub async fn read_text_file(
        &self,
        params: ReadTextFileRequest,
    ) -> Result<ReadTextFileResponse> {
        let path = self.resolve_path_within_root(&params.path)?;
        if self.permission_mode == PermissionMode::DenyAll {
            return Err(AcpError::PermissionDenied(
                "fs/read_text_file (--deny-all)".to_string(),
            ));
        }

        let content = std::fs::read_to_string(&path)
            .map_err(|source| io_err(format!("failed to read {}", path.display()), source))?;
        Ok(ReadTextFileResponse::new(slice_content(
            &content,
            params.line,
            params.limit,
        )))
    }

    /// Ports `writeTextFile`.
    pub async fn write_text_file(
        &self,
        params: WriteTextFileRequest,
    ) -> Result<WriteTextFileResponse> {
        let path = self.resolve_path_within_root(&params.path)?;
        let title = format!("Allow write to {}", path.display());
        let approved = confirm_action(
            self.permission_mode,
            self.non_interactive_policy,
            self.handler.as_deref(),
            params.session_id.clone(),
            title,
        )
        .await?;
        if !approved {
            return Err(AcpError::PermissionDenied("fs/write_text_file".to_string()));
        }

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|source| {
                io_err(format!("failed to create {}", parent.display()), source)
            })?;
        }
        std::fs::write(&path, &params.content)
            .map_err(|source| io_err(format!("failed to write {}", path.display()), source))?;
        Ok(WriteTextFileResponse::new())
    }

    /// Ports `resolvePathWithinRoot`, tightened per the module doc's
    /// symlink-escape decision: canonicalize (following symlinks) before
    /// comparing against `root_dir`.
    fn resolve_path_within_root(&self, raw_path: &Path) -> Result<PathBuf> {
        if !raw_path.is_absolute() {
            return Err(AcpError::Other(anyhow::anyhow!(
                "path must be absolute: {}",
                raw_path.display()
            )));
        }
        let resolved = canonicalize_for_sandbox(raw_path)?;
        if !resolved.starts_with(&self.root_dir) {
            return Err(AcpError::PermissionDenied(format!(
                "path is outside allowed cwd subtree: {}",
                resolved.display()
            )));
        }
        Ok(resolved)
    }
}

/// Resolves `path` to its canonical (symlink-free) form even when the path
/// itself doesn't exist yet (needed for `write_text_file` targeting a new
/// file): walks up to the longest existing ancestor, canonicalizes that,
/// then re-appends the remaining path components lexically.
fn canonicalize_for_sandbox(path: &Path) -> Result<PathBuf> {
    let mut existing = path;
    let mut remainder: Vec<&std::ffi::OsStr> = Vec::new();

    loop {
        if let Ok(canon) = existing.canonicalize() {
            let mut resolved = canon;
            for part in remainder.into_iter().rev() {
                resolved.push(part);
            }
            return Ok(resolved);
        }

        let Some(parent) = existing.parent().filter(|p| *p != existing) else {
            return Err(AcpError::Other(anyhow::anyhow!(
                "failed to resolve path: {}",
                path.display()
            )));
        };
        if let Some(name) = existing.file_name() {
            remainder.push(name);
        }
        existing = parent;
    }
}

fn io_err(context: String, source: std::io::Error) -> AcpError {
    AcpError::Other(anyhow::anyhow!("{context}: {source}"))
}

/// Ports `sliceContent`: 1-based `line` + `limit` windowing over the file's
/// lines; `None`/`None` returns the whole file untouched.
fn slice_content(content: &str, line: Option<u32>, limit: Option<u32>) -> String {
    if line.is_none() && limit.is_none() {
        return content.to_string();
    }

    let lines: Vec<&str> = content.split('\n').collect();
    let start_index = line.map(|l| l.max(1) - 1).unwrap_or(0) as usize;
    if start_index >= lines.len() {
        return String::new();
    }
    if limit == Some(0) {
        return String::new();
    }

    let end_index = match limit {
        Some(max_lines) => lines.len().min(start_index + max_lines as usize),
        None => lines.len(),
    };
    lines[start_index..end_index].join("\n")
}

// Split out per the workspace's <200-line file guideline; logically still
// part of this module (`super::*` sees its private items).
#[cfg(test)]
#[path = "filesystem_tests.rs"]
mod tests;
