//! Zed's `util::debug_panic!` has no equivalent in this workspace's `util`
//! crate (it diverged from Zed's own `util` — see `docs/codebase-summary.md`
//! for the crate list). Rather than adding it to the shared `util` crate for
//! one caller, it is reproduced locally: panics in debug builds (surfacing
//! rope invariant violations immediately during development), logs an error
//! in release builds (matching Zed's original fail-soft behavior for
//! malformed-but-recoverable text state).
macro_rules! debug_panic {
    ($($arg:tt)+) => {
        if cfg!(debug_assertions) {
            panic!($($arg)+);
        } else {
            log::error!($($arg)+);
        }
    };
}

pub(crate) use debug_panic;
