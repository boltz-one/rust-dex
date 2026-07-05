//! Ports `pruneSessions`.

use std::fs;
use std::path::Path;

use crate::error::{AcpError, Result};
use crate::session::record::SessionRecord;
use crate::session::store_options::{
    AcpFileSessionStoreOptions, ensure_session_dir, safe_session_id,
};

use super::load_record_from_file;
use crate::session::persistence::index::{
    SessionIndexEntry, load_or_rebuild_session_index, rebuild_session_index,
};

pub struct PruneOptions {
    pub agent_command: Option<String>,
    pub before: Option<String>,
    pub include_history: bool,
    pub dry_run: bool,
}

pub struct PruneResult {
    pub pruned: Vec<SessionRecord>,
    pub bytes_freed: u64,
    pub dry_run: bool,
}

fn is_session_stream_file(file_name: &str, safe_id: &str) -> bool {
    file_name == format!("{safe_id}.stream.ndjson")
        || file_name == format!("{safe_id}.stream.lock")
        || file_name.starts_with(&format!("{safe_id}.stream."))
}

fn unlink_counting_bytes(path: &Path) -> u64 {
    let bytes = fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    let _ = fs::remove_file(path);
    bytes
}

/// Ports `pruneSessions`.
pub fn prune_sessions(
    options: &AcpFileSessionStoreOptions,
    query: &PruneOptions,
) -> Result<PruneResult> {
    ensure_session_dir(options).map_err(|err| AcpError::Other(err.into()))?;
    let entries =
        load_or_rebuild_session_index(options).map_err(|err| AcpError::Other(err.into()))?;

    let eligible: Vec<&SessionIndexEntry> = entries
        .iter()
        .filter(|entry| {
            entry.closed
                && query
                    .agent_command
                    .as_deref()
                    .is_none_or(|cmd| entry.agent_command == cmd)
        })
        .collect();

    let records: Vec<SessionRecord> = eligible
        .iter()
        .filter_map(|entry| load_record_from_file(options, &entry.file))
        .filter(|record| {
            query
                .before
                .as_deref()
                .is_none_or(|cutoff| record.closed_at_or_last_used_at() < cutoff)
        })
        .collect();

    if query.dry_run {
        return Ok(PruneResult {
            pruned: records,
            bytes_freed: 0,
            dry_run: true,
        });
    }

    let session_dir = options.session_dir();
    let dir_entries: Vec<String> = if query.include_history {
        fs::read_dir(&session_dir)
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter_map(|e| e.file_name().into_string().ok())
                    .collect()
            })
            .unwrap_or_default()
    } else {
        Vec::new()
    };

    let mut bytes_freed = 0;
    for record in &records {
        let safe_id = safe_session_id(&record.acpx_record_id);
        bytes_freed += unlink_counting_bytes(&session_dir.join(format!("{safe_id}.json")));
        if query.include_history {
            for name in dir_entries
                .iter()
                .filter(|name| is_session_stream_file(name, &safe_id))
            {
                bytes_freed += unlink_counting_bytes(&session_dir.join(name));
            }
        }
    }

    let _ = rebuild_session_index(options);
    Ok(PruneResult {
        pruned: records,
        bytes_freed,
        dry_run: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::persistence::repository::write::write_session_record;
    use crate::session::persistence::serialize::test_support::sample_session_record;
    use crate::session::store_options::session_file_path;

    fn options(dir: &tempfile::TempDir) -> AcpFileSessionStoreOptions {
        AcpFileSessionStoreOptions::new(dir.path())
    }

    #[test]
    fn prune_dry_run_reports_without_deleting() {
        let dir = tempfile::tempdir().unwrap();
        let options = options(&dir);
        let mut record = sample_session_record();
        record.closed = true;
        write_session_record(&options, &record).unwrap();

        let result = prune_sessions(
            &options,
            &PruneOptions {
                agent_command: None,
                before: None,
                include_history: false,
                dry_run: true,
            },
        )
        .unwrap();
        assert!(result.dry_run);
        assert_eq!(result.pruned.len(), 1);
        assert!(session_file_path(&options, &record.acpx_record_id).exists());
    }

    #[test]
    fn prune_real_run_deletes_closed_sessions() {
        let dir = tempfile::tempdir().unwrap();
        let options = options(&dir);
        let mut record = sample_session_record();
        record.closed = true;
        write_session_record(&options, &record).unwrap();

        let result = prune_sessions(
            &options,
            &PruneOptions {
                agent_command: None,
                before: None,
                include_history: false,
                dry_run: false,
            },
        )
        .unwrap();
        assert!(!result.dry_run);
        assert_eq!(result.pruned.len(), 1);
        assert!(!session_file_path(&options, &record.acpx_record_id).exists());
    }

    #[test]
    fn prune_skips_open_sessions() {
        let dir = tempfile::tempdir().unwrap();
        let options = options(&dir);
        let record = sample_session_record();
        write_session_record(&options, &record).unwrap();

        let result = prune_sessions(
            &options,
            &PruneOptions {
                agent_command: None,
                before: None,
                include_history: false,
                dry_run: false,
            },
        )
        .unwrap();
        assert!(result.pruned.is_empty());
        assert!(session_file_path(&options, &record.acpx_record_id).exists());
    }
}
