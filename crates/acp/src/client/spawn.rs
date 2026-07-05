//! Subprocess spawn for the ACP agent. Ports the spawn half of
//! `others/acpx/src/acp/client-process.ts` (`waitForSpawn`,
//! `requireAgentStdio`), reusing `util::command`/`util::process` per ADR-3
//! instead of re-deriving spawn/kill primitives.

use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;

use util::process::Child;

use crate::error::{AcpError, Result};

/// Everything needed to spawn one ACP agent process.
pub struct SpawnOptions<'a> {
    pub program: &'a str,
    pub args: &'a [String],
    pub cwd: &'a Path,
    /// Full environment for the child (already merged via
    /// [`crate::auth_env::build_agent_environment`] by the caller).
    pub env: &'a HashMap<String, String>,
}

/// Spawns the agent with piped stdin/stdout/stderr, matching acpx's
/// `buildAgentSpawnOptions` (`stdio: ["pipe", "pipe", "pipe"]`,
/// `windowsHide: true` — the latter is `util::command::new_std_command`'s
/// `CREATE_NO_WINDOW` flag on Windows).
pub fn spawn_agent_process(options: SpawnOptions<'_>) -> Result<Child> {
    let mut command = util::command::new_std_command(options.program);
    command.args(options.args);
    command.current_dir(options.cwd);
    command.env_clear();
    command.envs(options.env);

    Child::spawn(command, Stdio::piped(), Stdio::piped(), Stdio::piped()).map_err(|source| {
        AcpError::AgentSpawn {
            command: display_command(options.program, options.args),
            source: source
                .downcast::<std::io::Error>()
                .unwrap_or_else(|err| std::io::Error::other(err.to_string())),
        }
    })
}

fn display_command(program: &str, args: &[String]) -> String {
    std::iter::once(program.to_string())
        .chain(args.iter().cloned())
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawns_and_captures_stdio() {
        smol::block_on(async {
            let env = HashMap::from([(
                "PATH".to_string(),
                std::env::var("PATH").unwrap_or_default(),
            )]);
            let child = spawn_agent_process(SpawnOptions {
                program: "/bin/echo",
                args: &["hello".to_string()],
                cwd: Path::new("/tmp"),
                env: &env,
            })
            .expect("spawn should succeed");

            assert!(child.stdout.is_some());
            let output = child.into_inner().output().await.expect("output");
            assert!(output.status.success());
            assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "hello");
        });
    }

    #[test]
    fn missing_program_returns_agent_spawn_error() {
        let env = HashMap::new();
        let result = spawn_agent_process(SpawnOptions {
            program: "/definitely/not/a/real/binary-xyz",
            args: &[],
            cwd: Path::new("/tmp"),
            env: &env,
        });
        assert!(matches!(result, Err(AcpError::AgentSpawn { .. })));
    }
}
