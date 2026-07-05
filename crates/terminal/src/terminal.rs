//! Real PTY terminal session: spawns the user's shell, feeds its output
//! through `alacritty_terminal`'s VT100 parser, and exposes the resulting
//! screen grid as plain text.
//!
//! Deliberately NOT a port of Zed's `terminal` crate (9kLOC across
//! `terminal.rs`/`alacritty.rs`/`pty_info.rs`/`mappings/*`) — that crate is
//! entangled with Zed's `settings`/`task`/`theme_settings`/`release_channel`
//! crates (none of which exist in this workspace) and with GPUI view/input
//! code that belongs in `crates/ui`, not here. This crate reproduces only
//! the PTY session + VT100 state core, written against the published
//! `alacritty_terminal` crate (Zed itself uses a private fork, so exact API
//! shapes were verified here via `cargo check`, not assumed from Zed's
//! source).
//!
//! PTY syscalls (`forkpty` on Unix, `ConPTY` on Windows) are entirely
//! internal to `alacritty_terminal::tty::new` — this crate never writes
//! `#[cfg(target_os)]` itself, satisfying the "no platform gates outside
//! `gpui_platform`/backend crates" convention without needing a facade
//! layer, because the branching never appears in code this crate owns.
//!
//! Cross-platform status: implemented and manually verified on macOS only
//! (this session's environment has no Linux/Windows to test on). Per
//! `docs/code-standards.md` review criteria, treat Linux/Windows as
//! UNVERIFIED until someone runs it there — do not assume ConPTY behaves
//! identically to the Unix path this was built against.

use std::cell::Cell as StdCell;
use std::collections::HashMap;
use std::sync::Arc;

use alacritty_terminal::event::{Event, EventListener, Notify, WindowSize};
use alacritty_terminal::event_loop::{EventLoop, Msg, Notifier};
use alacritty_terminal::grid::Dimensions;
use alacritty_terminal::sync::FairMutex;
use alacritty_terminal::term::cell::Flags;
use alacritty_terminal::term::{Config, Term};
use alacritty_terminal::tty;
use alacritty_terminal::vte::ansi::{Color as AnsiColor, NamedColor};

/// Terminal grid size in rows/columns, plus the pixel size of one cell
/// (alacritty needs cell pixel size to compute `WindowSize` for the PTY,
/// even though this crate never rasterizes anything itself).
#[derive(Clone, Copy, Debug)]
pub struct TerminalSize {
    pub rows: u16,
    pub columns: u16,
    pub cell_width: u16,
    pub cell_height: u16,
}

impl TerminalSize {
    fn window_size(&self) -> WindowSize {
        WindowSize {
            num_lines: self.rows,
            num_cols: self.columns,
            cell_width: self.cell_width,
            cell_height: self.cell_height,
        }
    }
}

impl Dimensions for TerminalSize {
    fn total_lines(&self) -> usize {
        self.rows as usize
    }

    fn screen_lines(&self) -> usize {
        self.rows as usize
    }

    fn columns(&self) -> usize {
        self.columns as usize
    }
}

/// A resolved 24-bit color for one cell's foreground or background.
/// `None` (see [`TerminalCell`]) means "use the caller's default text
/// color" — this crate has no opinion on what that default is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

/// Resolves an ANSI color (named/indexed/direct-RGB) to a concrete 24-bit
/// color. `NamedColor::Foreground`/`Background` resolve to `None` (the
/// cell should use the terminal's default color, not an explicit one) —
/// everything else, including `Cursor` and the `Dim*`/`Bright*` variants,
/// gets a concrete value so cells never silently lose color. The 16
/// standard values are the conventional xterm defaults; the 256-color
/// cube/grayscale ramp formulas (indices 16-231 / 232-255) are the
/// standard ANSI-256 formulas, not something this crate invented.
fn resolve_color(color: AnsiColor) -> Option<Rgb> {
    const fn rgb(r: u8, g: u8, b: u8) -> Rgb {
        Rgb { r, g, b }
    }

    match color {
        AnsiColor::Spec(rgb_color) => Some(Rgb {
            r: rgb_color.r,
            g: rgb_color.g,
            b: rgb_color.b,
        }),
        AnsiColor::Named(NamedColor::Foreground | NamedColor::Background) => None,
        AnsiColor::Named(named) => Some(match named {
            NamedColor::Black | NamedColor::DimBlack => rgb(0x00, 0x00, 0x00),
            NamedColor::Red | NamedColor::DimRed => rgb(0xCD, 0x00, 0x00),
            NamedColor::Green | NamedColor::DimGreen => rgb(0x00, 0xCD, 0x00),
            NamedColor::Yellow | NamedColor::DimYellow => rgb(0xCD, 0xCD, 0x00),
            NamedColor::Blue | NamedColor::DimBlue => rgb(0x00, 0x00, 0xEE),
            NamedColor::Magenta | NamedColor::DimMagenta => rgb(0xCD, 0x00, 0xCD),
            NamedColor::Cyan | NamedColor::DimCyan => rgb(0x00, 0xCD, 0xCD),
            NamedColor::White | NamedColor::DimWhite => rgb(0xE5, 0xE5, 0xE5),
            NamedColor::BrightBlack => rgb(0x7F, 0x7F, 0x7F),
            NamedColor::BrightRed => rgb(0xFF, 0x00, 0x00),
            NamedColor::BrightGreen => rgb(0x00, 0xFF, 0x00),
            NamedColor::BrightYellow => rgb(0xFF, 0xFF, 0x00),
            NamedColor::BrightBlue => rgb(0x5C, 0x5C, 0xFF),
            NamedColor::BrightMagenta => rgb(0xFF, 0x00, 0xFF),
            NamedColor::BrightCyan => rgb(0x00, 0xFF, 0xFF),
            NamedColor::BrightWhite => rgb(0xFF, 0xFF, 0xFF),
            // Cursor / BrightForeground / DimForeground and any future
            // variant: fall back to a neutral mid-gray rather than
            // silently dropping color information.
            _ => rgb(0xBF, 0xBF, 0xBF),
        }),
        AnsiColor::Indexed(index) => Some(indexed_color(index)),
    }
}

/// Standard ANSI-256 palette formula: 0-15 mirror the named colors above,
/// 16-231 are a 6x6x6 RGB cube, 232-255 are a 24-step grayscale ramp.
fn indexed_color(index: u8) -> Rgb {
    const CUBE_STEPS: [u8; 6] = [0x00, 0x5F, 0x87, 0xAF, 0xD7, 0xFF];

    match index {
        0..=15 => resolve_color(AnsiColor::Named(NAMED_BY_INDEX[index as usize])).unwrap_or(Rgb {
            r: 0,
            g: 0,
            b: 0,
        }),
        16..=231 => {
            let i = index - 16;
            let r = CUBE_STEPS[(i / 36) as usize];
            let g = CUBE_STEPS[((i / 6) % 6) as usize];
            let b = CUBE_STEPS[(i % 6) as usize];
            Rgb { r, g, b }
        }
        232..=255 => {
            let level = 8 + 10 * (index - 232);
            Rgb {
                r: level,
                g: level,
                b: level,
            }
        }
    }
}

const NAMED_BY_INDEX: [NamedColor; 16] = [
    NamedColor::Black,
    NamedColor::Red,
    NamedColor::Green,
    NamedColor::Yellow,
    NamedColor::Blue,
    NamedColor::Magenta,
    NamedColor::Cyan,
    NamedColor::White,
    NamedColor::BrightBlack,
    NamedColor::BrightRed,
    NamedColor::BrightGreen,
    NamedColor::BrightYellow,
    NamedColor::BrightBlue,
    NamedColor::BrightMagenta,
    NamedColor::BrightCyan,
    NamedColor::BrightWhite,
];

/// One rendered terminal cell: its character plus resolved style. `fg`/`bg`
/// are `None` when the cell uses the terminal's default colors (not an
/// explicit ANSI color) — the renderer's own default text/background color
/// should apply, not black-on-black or similar.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TerminalCell {
    pub text: char,
    pub fg: Option<Rgb>,
    pub bg: Option<Rgb>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
}

/// Forwards `alacritty_terminal`'s internal notifications (redraw-needed,
/// title change, PTY exited, bell) to an [`async_channel`] so a caller (the
/// GPUI-side `TerminalView`) can `cx.notify()` on the next one without this
/// crate depending on `gpui` at all.
#[derive(Clone)]
struct EventProxy(async_channel::Sender<Event>);

impl EventListener for EventProxy {
    fn send_event(&self, event: Event) {
        // A full channel or a dropped receiver both just mean "nobody is
        // listening for redraws right now" (e.g. the view was torn down
        // mid-shutdown) — not an error worth propagating.
        let _ = self.0.try_send(event);
    }
}

/// The shell to spawn. Resolves `$SHELL` on Unix (falling back to
/// `/bin/sh`, matching the POSIX baseline every Unix has); Windows has no
/// `$SHELL` convention, so falls back to `cmd.exe`. This crate stays out of
/// user-configurable shell selection — a caller can build its own
/// `tty::Shell` and use [`Terminal::spawn_with_shell`] instead.
fn default_shell() -> tty::Shell {
    #[cfg(unix)]
    {
        let program = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        tty::Shell::new(program, Vec::new())
    }
    #[cfg(windows)]
    {
        tty::Shell::new("cmd.exe".to_string(), Vec::new())
    }
}

/// A live PTY session: a spawned shell process, its VT100-parsed screen
/// state, and the channel to write keystrokes / resize / shut it down.
///
/// Owns a background OS thread (spawned by `alacritty_terminal`'s
/// `EventLoop`) for the lifetime of the terminal — dropping every clone of
/// the returned `Terminal` does NOT kill that thread or the child process;
/// call [`Terminal::shutdown`] explicitly before dropping to avoid leaking
/// the child shell (see this crate's tests for the expected teardown path).
pub struct Terminal {
    term: Arc<FairMutex<Term<EventProxy>>>,
    notifier: Notifier,
    /// Mirrors the size last sent via `spawn`/`resize`, so callers (e.g.
    /// `TerminalView`, deciding whether a new pane size actually changed
    /// the grid) don't need to keep their own copy in sync by hand. A
    /// plain `Cell` is enough — `Terminal` isn't `Sync`-shared across
    /// threads (only its internal `Arc<FairMutex<_>>` grid state is,
    /// with the background I/O thread).
    current_size: StdCell<TerminalSize>,
}

impl Terminal {
    /// Spawns `$SHELL` (or `cmd.exe` on Windows) as a PTY child process.
    /// Returns the `Terminal` handle plus a receiver for redraw/exit
    /// notifications — the caller is responsible for polling it (e.g. via
    /// `cx.background_spawn` bridging into `cx.notify()`).
    pub fn spawn(size: TerminalSize) -> anyhow::Result<(Self, async_channel::Receiver<Event>)> {
        Self::spawn_with_shell(default_shell(), size)
    }

    pub fn spawn_with_shell(
        shell: tty::Shell,
        size: TerminalSize,
    ) -> anyhow::Result<(Self, async_channel::Receiver<Event>)> {
        let (tx, rx) = async_channel::unbounded();

        let pty_options = tty::Options {
            shell: Some(shell),
            working_directory: std::env::current_dir().ok(),
            drain_on_exit: true,
            env: HashMap::new(),
        };

        let pty = tty::new(&pty_options, size.window_size(), 0)?;

        let config = Config::default();
        let term = Arc::new(FairMutex::new(Term::new(
            config,
            &size,
            EventProxy(tx.clone()),
        )));

        let event_loop = EventLoop::new(term.clone(), EventProxy(tx.clone()), pty, false, false)?;
        let notifier = Notifier(event_loop.channel());
        // `EventLoop::spawn` starts the dedicated PTY-reading OS thread and
        // returns its `JoinHandle`; this crate doesn't need to join it —
        // `Msg::Shutdown` (sent from `shutdown`) is what makes it exit.
        let _io_thread = event_loop.spawn();

        Ok((
            Self {
                term,
                notifier,
                current_size: StdCell::new(size),
            },
            rx,
        ))
    }

    /// The size last passed to `spawn`/`spawn_with_shell`/`resize`.
    pub fn current_size(&self) -> TerminalSize {
        self.current_size.get()
    }

    /// Writes raw bytes to the PTY as if the user typed them (already
    /// encoded — e.g. `b"\r"` for Enter, `b"\x1b[A"` for an up-arrow escape
    /// sequence). Encoding keystrokes into the right bytes is the caller's
    /// job (`crates/ui`'s `TerminalView`), matching this crate's boundary of
    /// "PTY session only, no input-handling policy."
    pub fn write_input(&self, bytes: impl Into<std::borrow::Cow<'static, [u8]>>) {
        self.notifier.notify(bytes);
    }

    /// Informs both the VT100 grid and the PTY (so the child process's
    /// `SIGWINCH`/console-resize sees it) of a new size.
    pub fn resize(&self, size: TerminalSize) {
        self.term.lock().resize(size);
        self.notifier.0.send(Msg::Resize(size.window_size())).ok();
        self.current_size.set(size);
    }

    /// Extracts the current screen as plain lines of text (colors/
    /// attributes dropped). Kept alongside [`Self::screen_cells`] (which
    /// has the real per-cell style) because it's a much simpler match for
    /// call sites that only want the text — e.g. tests that just check a
    /// substring appeared, not how it's colored.
    pub fn screen_lines(&self) -> Vec<String> {
        let term = self.term.lock();
        let grid = term.grid();
        (0..grid.screen_lines())
            .map(|line_ix| {
                grid[alacritty_terminal::index::Line(line_ix as i32)]
                    .into_iter()
                    .map(|cell| cell.c)
                    .collect::<String>()
                    .trim_end()
                    .to_string()
            })
            .collect()
    }

    /// Extracts the current screen as rows of styled cells — real per-cell
    /// foreground/background color and bold/italic/underline, resolved
    /// from the VT100 grid's ANSI color state via [`resolve_color`]. This
    /// is what `crates/ui`'s `TerminalView` renders instead of the
    /// monochrome `screen_lines`.
    pub fn screen_cells(&self) -> Vec<Vec<TerminalCell>> {
        let term = self.term.lock();
        let grid = term.grid();
        (0..grid.screen_lines())
            .map(|line_ix| {
                grid[alacritty_terminal::index::Line(line_ix as i32)]
                    .into_iter()
                    .map(|cell| TerminalCell {
                        text: cell.c,
                        fg: resolve_color(cell.fg),
                        bg: resolve_color(cell.bg),
                        bold: cell.flags.contains(Flags::BOLD),
                        italic: cell.flags.contains(Flags::ITALIC),
                        underline: cell.flags.intersects(
                            Flags::UNDERLINE
                                | Flags::DOUBLE_UNDERLINE
                                | Flags::UNDERCURL
                                | Flags::DOTTED_UNDERLINE
                                | Flags::DASHED_UNDERLINE,
                        ),
                    })
                    .collect()
            })
            .collect()
    }

    /// Ends the PTY session: signals the child process's controlling
    /// terminal to hang up and stops the background I/O thread. Exposed as
    /// a public method for callers that want to shut down explicitly
    /// (before dropping); [`Terminal`] also does this automatically on
    /// `Drop` (see below) as a leak-prevention safety net — `Terminal` is
    /// not `Clone`, so there is exactly one owner and `Drop` fires exactly
    /// once, unlike the internal `Arc<FairMutex<Term<_>>>` grid state, which
    /// is shared with the background I/O thread and outlives this call.
    pub fn shutdown(&self) {
        self.notifier.0.send(Msg::Shutdown).ok();
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        self.shutdown();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn small_size() -> TerminalSize {
        TerminalSize {
            rows: 24,
            columns: 80,
            cell_width: 8,
            cell_height: 16,
        }
    }

    /// Spawns a real shell, runs a command through it, and asserts the
    /// command's actual output shows up in the parsed screen grid. This is
    /// the one test in this crate that exercises the real PTY end-to-end —
    /// everything else about VT100 parsing is already covered by
    /// `alacritty_terminal`'s own test suite, which this crate doesn't
    /// re-test.
    #[test]
    fn spawns_shell_and_reads_real_output() {
        let shell = tty::Shell::new("/bin/sh".to_string(), Vec::new());
        let (terminal, _events) =
            Terminal::spawn_with_shell(shell, small_size()).expect("failed to spawn PTY shell");

        terminal.write_input(b"echo terminal_smoke_test_marker\n".as_slice());

        // No PTY output completion signal to await deterministically without
        // pulling in a full async runtime here — poll with a short real
        // sleep, matching the "spawn a real shell" scope of this specific
        // test (unlike the rest of this crate, this one test's raison
        // d'être is exercising real timing-sensitive OS process I/O).
        let mut found = false;
        for _ in 0..50 {
            std::thread::sleep(Duration::from_millis(20));
            if terminal
                .screen_lines()
                .iter()
                .any(|line| line.contains("terminal_smoke_test_marker"))
            {
                found = true;
                break;
            }
        }

        terminal.shutdown();
        assert!(
            found,
            "expected echoed marker in PTY output, got: {:?}",
            terminal.screen_lines()
        );
    }

    /// `Terminal::resize` must update both what `current_size()` reports
    /// AND the grid `alacritty_terminal` actually renders into — the two
    /// are set in different places internally (`Cell::set` vs. `Term::
    /// resize`), so this specifically checks the grid's real row/column
    /// count changed, not just the bookkeeping field.
    #[test]
    fn resize_changes_reported_size_and_real_grid_dimensions() {
        let shell = tty::Shell::new("/bin/sh".to_string(), Vec::new());
        let (terminal, _events) =
            Terminal::spawn_with_shell(shell, small_size()).expect("failed to spawn PTY shell");

        assert_eq!(terminal.current_size().rows, 24);
        assert_eq!(terminal.current_size().columns, 80);
        assert_eq!(terminal.screen_lines().len(), 24);

        let bigger = TerminalSize {
            rows: 40,
            columns: 120,
            cell_width: 8,
            cell_height: 16,
        };
        terminal.resize(bigger);

        assert_eq!(terminal.current_size().rows, 40);
        assert_eq!(terminal.current_size().columns, 120);
        assert_eq!(
            terminal.screen_lines().len(),
            40,
            "grid should now report 40 screen lines, not just the bookkeeping field"
        );

        terminal.shutdown();
    }

    #[test]
    fn resolve_color_maps_named_ansi_colors_to_concrete_rgb() {
        assert_eq!(
            resolve_color(AnsiColor::Named(NamedColor::Red)),
            Some(Rgb {
                r: 0xCD,
                g: 0x00,
                b: 0x00
            })
        );
        assert_eq!(
            resolve_color(AnsiColor::Named(NamedColor::BrightGreen)),
            Some(Rgb {
                r: 0x00,
                g: 0xFF,
                b: 0x00
            })
        );
        // Default fg/bg intentionally resolve to `None` — the renderer's
        // own default color should apply, not an explicit black/white.
        assert_eq!(
            resolve_color(AnsiColor::Named(NamedColor::Foreground)),
            None
        );
        assert_eq!(
            resolve_color(AnsiColor::Named(NamedColor::Background)),
            None
        );
    }

    #[test]
    fn resolve_color_passes_through_direct_rgb_spec() {
        let spec = alacritty_terminal::vte::ansi::Rgb {
            r: 0x12,
            g: 0x34,
            b: 0x56,
        };
        assert_eq!(
            resolve_color(AnsiColor::Spec(spec)),
            Some(Rgb {
                r: 0x12,
                g: 0x34,
                b: 0x56
            })
        );
    }

    #[test]
    fn indexed_color_covers_standard_16_cube_and_grayscale_ranges() {
        // Index 1 mirrors the standard "red" named color.
        assert_eq!(
            indexed_color(1),
            resolve_color(AnsiColor::Named(NamedColor::Red)).unwrap()
        );
        // Index 16 is the cube's first entry: pure black.
        assert_eq!(indexed_color(16), Rgb { r: 0, g: 0, b: 0 });
        // Index 231 is the cube's last entry: pure white.
        assert_eq!(
            indexed_color(231),
            Rgb {
                r: 0xFF,
                g: 0xFF,
                b: 0xFF
            }
        );
        // Index 232 is the grayscale ramp's darkest step.
        assert_eq!(indexed_color(232), Rgb { r: 8, g: 8, b: 8 });
    }

    /// Spawns a real shell, prints real SGR-colored text (`\x1b[31m...`),
    /// and asserts `screen_cells` resolved the printed character's
    /// foreground to the expected red — end-to-end proof that ANSI color
    /// escapes flow from the child process, through `alacritty_terminal`'s
    /// VT100 parser, to this crate's `TerminalCell` output.
    #[test]
    fn screen_cells_resolves_real_ansi_color_from_shell_output() {
        let shell = tty::Shell::new("/bin/sh".to_string(), Vec::new());
        let (terminal, _events) =
            Terminal::spawn_with_shell(shell, small_size()).expect("failed to spawn PTY shell");

        // `printf` (not `echo`) so the escape sequence is interpreted
        // consistently across shells without relying on echo's `-e` flag,
        // which isn't POSIX-portable.
        terminal.write_input(b"printf '\\033[31mZ\\033[0m\\n'\n".as_slice());

        // The PTY's line discipline echoes the *typed command line* back
        // before the shell even runs it — that echo contains the literal,
        // UNCOLORED text "...Z..." from the command source itself (no real
        // ESC byte, just the four characters backslash-0-3-3), and it lands
        // in the grid before printf's actual (colored) output does. So
        // matching on `cell.text == 'Z'` alone would find that echoed 'Z'
        // first, not the one this test actually cares about — filter on
        // `fg.is_some()` too, to target specifically the colored cell
        // printf produced.
        let mut red_cell = None;
        for _ in 0..50 {
            std::thread::sleep(Duration::from_millis(20));
            red_cell = terminal
                .screen_cells()
                .into_iter()
                .flatten()
                .find(|cell| cell.text == 'Z' && cell.fg.is_some());
            if red_cell.is_some() {
                break;
            }
        }

        terminal.shutdown();
        let cell = red_cell.expect("expected to find the printed 'Z' cell in the screen grid");
        assert_eq!(
            cell.fg,
            resolve_color(AnsiColor::Named(NamedColor::Red)),
            "expected 'Z' to be colored red via the real SGR escape sequence"
        );
    }
}
