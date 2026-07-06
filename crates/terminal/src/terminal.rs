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

// Re-export `alacritty_terminal`'s own pure-data index/selection/search/mode
// types directly (ADR #1: no custom wrapper types — they add a lossy
// conversion layer for zero behavioural gain, and the "no GPUI" boundary is
// about not depending on `gpui`, not about hiding alacritty's data shapes).
pub use alacritty_terminal::grid::Scroll;
pub use alacritty_terminal::index::{Column, Direction, Line, Point, Side};
pub use alacritty_terminal::selection::{Selection, SelectionRange, SelectionType};
pub use alacritty_terminal::term::TermMode;
pub use alacritty_terminal::term::search::{Match, RegexSearch};
pub use alacritty_terminal::vte::ansi::CursorShape;

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

/// Converts one `alacritty_terminal` grid cell into a style-resolved
/// [`TerminalCell`]. Shared by [`Terminal::screen_cells`],
/// [`Terminal::scrollback_cells`] and [`Terminal::cell_at`].
fn convert_cell(cell: &alacritty_terminal::term::cell::Cell) -> TerminalCell {
    TerminalCell {
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
    }
}

/// A mouse button, for [`Terminal::mouse_report_sgr`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Middle,
    Right,
}

/// The kind of mouse event being reported, for [`Terminal::mouse_report_sgr`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MouseEventKind {
    Down,
    Up,
    Drag,
    ScrollUp,
    ScrollDown,
}

/// Keyboard modifiers active during a mouse event (SGR encodes them into the
/// button byte).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MouseModifiers {
    pub shift: bool,
    pub alt: bool,
    pub control: bool,
}

/// A detected hyperlink span in the grid — text plus its inclusive grid
/// `Point` range. Data-only: this crate never opens anything (the view layer
/// does, gated on an explicit click).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HyperlinkMatch {
    pub text: String,
    /// True for URL-scheme matches (`https://…`); reserved `false` for future
    /// path-like matches, which the view layer resolves against a cwd.
    pub is_url: bool,
    pub start: Point,
    pub end: Point,
}

/// The cursor's grid position, shape, and visibility — read by the view layer
/// to paint it.
#[derive(Clone, Copy, Debug)]
pub struct CursorState {
    pub point: Point,
    pub shape: CursorShape,
    pub visible: bool,
}

/// Encodes a mouse event in the SGR mouse protocol (`\x1b[<{cb};{col};{row}{M|m}`,
/// 1-based coordinates). Pure function (no mode gating) so it can be
/// byte-exact unit-tested; [`Terminal::mouse_report_sgr`] gates it on the
/// terminal actually having a mouse-reporting mode enabled.
fn encode_sgr_mouse(
    button: MouseButton,
    kind: MouseEventKind,
    point: Point,
    mods: MouseModifiers,
) -> Vec<u8> {
    let mut cb: u8 = match kind {
        MouseEventKind::ScrollUp => 64,
        MouseEventKind::ScrollDown => 65,
        _ => match button {
            MouseButton::Left => 0,
            MouseButton::Middle => 1,
            MouseButton::Right => 2,
        },
    };
    if matches!(kind, MouseEventKind::Drag) {
        cb += 32; // motion bit
    }
    if mods.shift {
        cb += 4;
    }
    if mods.alt {
        cb += 8;
    }
    if mods.control {
        cb += 16;
    }
    // SGR coordinates are 1-based; `m` is the release final byte, `M` press.
    let col = point.column.0 + 1;
    let row = point.line.0 + 1;
    let final_byte = if matches!(kind, MouseEventKind::Up) {
        'm'
    } else {
        'M'
    };
    format!("\x1b[<{cb};{col};{row}{final_byte}").into_bytes()
}

/// Removes any embedded bracketed-paste end-marker (`\x1b[201~`) from pasted
/// text, so untrusted clipboard content cannot terminate paste mode early and
/// have its remainder run as typed input (bracketed-paste injection — xterm
/// applies the same guard).
fn strip_paste_terminator(text: &str) -> String {
    text.replace("\x1b[201~", "")
}

/// All regex matches in the grid (incl. scrollback), left-to-right/top-to-
/// bottom, computed against an ALREADY-LOCKED `Term`. Taking the locked term
/// (rather than re-locking) lets callers that need the match text keep the
/// same grid snapshot for the subsequent `bounds_to_string` — the grid's ring
/// buffer can rotate between separate `lock()` calls (background PTY reader),
/// which would invalidate the returned history-relative `Point`s.
fn search_all_locked(term: &Term<EventProxy>, search: &mut RegexSearch) -> Vec<Match> {
    let grid = term.grid();
    let cols = grid.columns();
    let start = Point {
        line: Line(-(grid.history_size() as i32)),
        column: Column(0),
    };
    let end = Point {
        line: Line(grid.screen_lines() as i32 - 1),
        column: Column(cols.saturating_sub(1)),
    };
    let mut matches = Vec::new();
    let mut origin = start;
    while let Some(m) = term.regex_search_right(search, origin, end) {
        let match_end = *m.end();
        matches.push(m);
        // Advance one cell past the match end (wrapping at the last column) so
        // the next search doesn't re-find the same span.
        origin = if match_end.column.0 + 1 >= cols {
            Point {
                line: Line(match_end.line.0 + 1),
                column: Column(0),
            }
        } else {
            Point {
                line: match_end.line,
                column: Column(match_end.column.0 + 1),
            }
        };
        if origin.line.0 > end.line.0 || matches.len() >= 10_000 {
            break;
        }
    }
    matches
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

        let event_loop = EventLoop::new(term.clone(), EventProxy(tx), pty, false, false)?;
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
                grid[Line(line_ix as i32)]
                    .into_iter()
                    .map(convert_cell)
                    .collect()
            })
            .collect()
    }

    // ---- Selection (mouse-driven text selection; view layer supplies grid
    // points from pixel coordinates via `point_for_pixel`) ----

    /// Begins a selection of the given kind anchored at `point`/`side`.
    /// Replaces any existing selection.
    pub fn selection_start(&self, ty: SelectionType, point: Point, side: Side) {
        self.term.lock().selection = Some(Selection::new(ty, point, side));
    }

    /// Extends the in-progress selection to `point`/`side` (no-op if none).
    pub fn selection_update(&self, point: Point, side: Side) {
        if let Some(selection) = self.term.lock().selection.as_mut() {
            selection.update(point, side);
        }
    }

    /// Clears any active selection.
    pub fn selection_clear(&self) {
        self.term.lock().selection = None;
    }

    /// The current selection's text (for copy), or `None` if nothing selected.
    pub fn selection_text(&self) -> Option<String> {
        self.term.lock().selection_to_string()
    }

    /// The current selection's resolved grid range, or `None`.
    pub fn selection_range(&self) -> Option<SelectionRange> {
        let term = self.term.lock();
        term.selection.as_ref().and_then(|s| s.to_range(&term))
    }

    // ---- Scroll & scrollback ----

    /// Scrolls the display (wheel / scrollbar / keys). [`Scroll::Delta`] with a
    /// positive delta moves up into history.
    pub fn scroll(&self, scroll: Scroll) {
        self.term.lock().scroll_display(scroll);
    }

    /// How many lines the display is scrolled up into history (0 = bottom).
    pub fn display_offset(&self) -> usize {
        self.term.lock().grid().display_offset()
    }

    /// Total lines available including scrollback history (screen + history).
    pub fn total_lines(&self) -> i32 {
        self.term.lock().grid().total_lines() as i32
    }

    /// Styled cells for a range of grid lines. Line indices are alacritty's:
    /// `0..screen_lines` is the visible screen, NEGATIVE indices reach into
    /// scrollback history. Out-of-range lines are clamped to the valid
    /// `-(history) ..= screen_lines-1` window (empty result if the clamp
    /// collapses the range).
    pub fn scrollback_cells(&self, lines: std::ops::Range<i32>) -> Vec<Vec<TerminalCell>> {
        let term = self.term.lock();
        let grid = term.grid();
        let min = -(grid.history_size() as i32);
        let max = grid.screen_lines() as i32; // exclusive
        let start = lines.start.max(min);
        let end = lines.end.min(max);
        if start >= end {
            return Vec::new();
        }
        (start..end)
            .map(|l| grid[Line(l)].into_iter().map(convert_cell).collect())
            .collect()
    }

    /// A single cell at `point` (for hyperlink hover / hit-testing), or `None`
    /// if the point is outside the grid.
    pub fn cell_at(&self, point: Point) -> Option<TerminalCell> {
        let term = self.term.lock();
        let grid = term.grid();
        if point.line.0 < -(grid.history_size() as i32)
            || point.line.0 >= grid.screen_lines() as i32
            || point.column.0 >= grid.columns()
        {
            return None;
        }
        Some(convert_cell(&grid[point.line][point.column]))
    }

    // ---- Search ----

    /// Compiles a search pattern into a reusable [`RegexSearch`] handle (kept
    /// live by the caller across next/prev navigation, matching Zed).
    pub fn search_compile(pattern: &str) -> anyhow::Result<RegexSearch> {
        RegexSearch::new(pattern).map_err(|e| anyhow::anyhow!("invalid search regex: {e}"))
    }

    /// Finds the next match from `from` in `direction` (the origin cell is
    /// included), searching the whole grid incl. scrollback. `None` if no
    /// match.
    pub fn search_next(
        &self,
        search: &mut RegexSearch,
        from: Point,
        direction: Direction,
    ) -> Option<Match> {
        let term = self.term.lock();
        let grid = term.grid();
        let cols = grid.columns();
        match direction {
            Direction::Right => {
                let end = Point {
                    line: Line(grid.screen_lines() as i32 - 1),
                    column: Column(cols.saturating_sub(1)),
                };
                term.regex_search_right(search, from, end)
            }
            Direction::Left => {
                let end = Point {
                    line: Line(-(grid.history_size() as i32)),
                    column: Column(0),
                };
                term.regex_search_left(search, from, end)
            }
        }
    }

    /// All matches in the grid (incl. scrollback), left-to-right, top-to-
    /// bottom. Bounded to avoid pathological loops.
    pub fn search_all(&self, search: &mut RegexSearch) -> Vec<Match> {
        let term = self.term.lock();
        search_all_locked(&term, search)
    }

    // ---- Modes, mouse reporting, pixel hit-testing ----

    /// The terminal's current mode flags (bracketed paste, alt-screen, mouse
    /// reporting, etc.) — read-only state the child process sets.
    pub fn mode(&self) -> TermMode {
        *self.term.lock().mode()
    }

    /// Encodes a mouse event as SGR mouse-report bytes to write to the PTY, or
    /// `None` if the program has not negotiated SGR mouse reporting (the view
    /// layer should then handle the event locally, e.g. as a text selection).
    /// SGR only (ADR #2): a program that enabled a tracking mode (1000/1002/
    /// 1003) WITHOUT SGR (1006) expects the legacy X10/UTF8 binary encoding,
    /// which this crate does not emit — so it returns `None` and degrades to
    /// local selection rather than sending bytes the program would misparse.
    pub fn mouse_report_sgr(
        &self,
        button: MouseButton,
        kind: MouseEventKind,
        point: Point,
        mods: MouseModifiers,
    ) -> Option<Vec<u8>> {
        let mode = self.mode();
        // Require BOTH a tracking mode (what events to report) AND SGR
        // encoding (how) — SGR_MOUSE is a separate flag from MOUSE_MODE.
        if !mode.intersects(TermMode::MOUSE_MODE) || !mode.contains(TermMode::SGR_MOUSE) {
            return None;
        }
        Some(encode_sgr_mouse(button, kind, point, mods))
    }

    /// Maps a pixel offset within the terminal's content area to a grid
    /// [`Point`], accounting for the current scrollback `display_offset`.
    pub fn point_for_pixel(
        &self,
        x_px: f32,
        y_px: f32,
        cell_width: f32,
        cell_height: f32,
    ) -> Point {
        let term = self.term.lock();
        let grid = term.grid();
        let cols = grid.columns() as i32;
        let screen = grid.screen_lines() as i32;
        let display_offset = grid.display_offset() as i32;
        let col = ((x_px / cell_width).floor() as i32).clamp(0, (cols - 1).max(0));
        let visible_row = ((y_px / cell_height).floor() as i32).clamp(0, (screen - 1).max(0));
        Point {
            // Displayed row 0 is line `-display_offset` when scrolled up.
            line: Line(visible_row - display_offset),
            column: Column(col as usize),
        }
    }

    // ---- Bracketed paste ----

    /// Writes pasted text to the PTY, wrapping it in bracketed-paste markers
    /// (`\x1b[200~`…`\x1b[201~`) iff the child process enabled
    /// [`TermMode::BRACKETED_PASTE`] — otherwise writes it raw.
    pub fn write_paste(&self, text: &str) {
        if self.mode().contains(TermMode::BRACKETED_PASTE) {
            self.write_input(b"\x1b[200~".to_vec());
            self.write_input(strip_paste_terminator(text).into_bytes());
            self.write_input(b"\x1b[201~".to_vec());
        } else {
            self.write_input(text.as_bytes().to_vec());
        }
    }

    // ---- Hyperlinks (data-only) ----

    /// Detects URL hyperlinks in the grid (fixed scheme set: http/https/ssh/
    /// git/file — not caller-configurable, see plan). Returns text + inclusive
    /// grid `Point` range; opening is the view layer's job. Path-like targets
    /// (needing a cwd) are resolved in the view layer, not here.
    pub fn find_hyperlinks(&self) -> Vec<HyperlinkMatch> {
        const URL_REGEX: &str = "(https|http|ssh|git|file)://[^ \t]+";
        let Ok(mut regex) = RegexSearch::new(URL_REGEX) else {
            return Vec::new();
        };
        // Search AND resolve match text under a single lock: the grid ring
        // buffer can rotate between separate `lock()` calls, invalidating the
        // history-relative match `Point`s (TOCTOU → wrong text or a panic in
        // `bounds_to_string`'s grid indexing).
        let term = self.term.lock();
        search_all_locked(&term, &mut regex)
            .into_iter()
            .map(|m| {
                let start = *m.start();
                let end = *m.end();
                HyperlinkMatch {
                    text: term.bounds_to_string(start, end),
                    is_url: true,
                    start,
                    end,
                }
            })
            .collect()
    }

    // ---- Cursor ----

    /// The cursor's current grid position, shape, and visibility (the view
    /// layer paints it).
    pub fn cursor(&self) -> CursorState {
        let term = self.term.lock();
        let content = term.renderable_content();
        CursorState {
            point: content.cursor.point,
            shape: content.cursor.shape,
            // `shape` is alacritty's single source of truth for visibility: it
            // is `Hidden` exactly when the cursor should not paint (folds in
            // both SHOW_CURSOR and the vi-mode exception), so derive from it.
            visible: content.cursor.shape != CursorShape::Hidden,
        }
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

    /// Byte-exact SGR mouse encoding for hand-verified cases (pure function,
    /// no PTY). Left press at col 5 / row 3 (1-based) -> `\x1b[<0;5;3M`.
    #[test]
    fn encode_sgr_mouse_matches_documented_format() {
        let down = encode_sgr_mouse(
            MouseButton::Left,
            MouseEventKind::Down,
            Point {
                line: Line(2),
                column: Column(4),
            },
            MouseModifiers::default(),
        );
        assert_eq!(down, b"\x1b[<0;5;3M");

        // Release uses the lowercase final byte `m`.
        let up = encode_sgr_mouse(
            MouseButton::Left,
            MouseEventKind::Up,
            Point {
                line: Line(2),
                column: Column(4),
            },
            MouseModifiers::default(),
        );
        assert_eq!(up, b"\x1b[<0;5;3m");

        // Right button + shift: cb = 2 + 4 = 6.
        let shift_right = encode_sgr_mouse(
            MouseButton::Right,
            MouseEventKind::Down,
            Point {
                line: Line(0),
                column: Column(0),
            },
            MouseModifiers {
                shift: true,
                ..Default::default()
            },
        );
        assert_eq!(shift_right, b"\x1b[<6;1;1M");
    }

    /// Polls the visible screen until `marker` appears, returning its (row,
    /// col) grid position. Panics if not found within the timeout.
    fn wait_for_marker(terminal: &Terminal, marker: &str) -> (i32, usize) {
        for _ in 0..50 {
            std::thread::sleep(Duration::from_millis(20));
            for (row, line) in terminal.screen_cells().iter().enumerate() {
                let text: String = line.iter().map(|c| c.text).collect();
                if let Some(col) = text.find(marker) {
                    return (row as i32, col);
                }
            }
        }
        panic!("marker {marker:?} never appeared on screen");
    }

    /// Real PTY: print a marker, select its cells, assert `selection_text`
    /// round-trips the printed text.
    #[test]
    fn selection_round_trip_captures_printed_text() {
        let shell = tty::Shell::new("/bin/sh".to_string(), Vec::new());
        let (terminal, _events) =
            Terminal::spawn_with_shell(shell, small_size()).expect("failed to spawn PTY shell");
        terminal.write_input(b"printf 'HELLOSEL\\n'\n".as_slice());

        let marker = "HELLOSEL";
        let (row, col) = wait_for_marker(&terminal, marker);
        terminal.selection_start(
            SelectionType::Simple,
            Point {
                line: Line(row),
                column: Column(col),
            },
            Side::Left,
        );
        terminal.selection_update(
            Point {
                line: Line(row),
                column: Column(col + marker.len() - 1),
            },
            Side::Right,
        );
        let selected = terminal.selection_text().unwrap_or_default();
        assert!(terminal.selection_range().is_some(), "range should resolve");
        terminal.shutdown();
        assert!(
            selected.contains(marker),
            "selection text {selected:?} should contain {marker:?}"
        );
    }

    /// Real PTY: search finds a printed marker via a compiled `RegexSearch`.
    #[test]
    fn search_finds_printed_marker() {
        let shell = tty::Shell::new("/bin/sh".to_string(), Vec::new());
        let (terminal, _events) =
            Terminal::spawn_with_shell(shell, small_size()).expect("failed to spawn PTY shell");
        terminal.write_input(b"printf 'FINDMEMARKER\\n'\n".as_slice());
        wait_for_marker(&terminal, "FINDMEMARKER");

        let mut regex = Terminal::search_compile("FINDMEMARKER").expect("valid regex");
        let hit = terminal.search_next(
            &mut regex,
            Point {
                line: Line(0),
                column: Column(0),
            },
            Direction::Right,
        );
        terminal.shutdown();
        assert!(hit.is_some(), "search should locate the printed marker");
    }

    /// Real PTY: printing more lines than the screen holds pushes rows into
    /// scrollback; scrolling up raises `display_offset` and `scrollback_cells`
    /// exposes negative-index history rows (concrete proof scrollback capacity
    /// exists — ADR #3).
    #[test]
    fn scroll_and_scrollback_expose_history() {
        let shell = tty::Shell::new("/bin/sh".to_string(), Vec::new());
        let (terminal, _events) =
            Terminal::spawn_with_shell(shell, small_size()).expect("failed to spawn PTY shell");
        // 100 lines into a 24-row screen -> ~76 lines scroll into history.
        terminal.write_input(b"seq 1 100\n".as_slice());
        wait_for_marker(&terminal, "100");

        assert_eq!(terminal.display_offset(), 0, "starts pinned at the bottom");
        terminal.scroll(Scroll::Delta(5));
        let offset = terminal.display_offset();
        let history = terminal.scrollback_cells(-5..0);
        terminal.shutdown();

        assert!(offset >= 1, "scrolling up should raise display_offset");
        assert!(
            !history.is_empty(),
            "scrollback_cells must expose history rows after 100 lines"
        );
    }

    /// Real PTY: a printed URL is detected by `find_hyperlinks`.
    #[test]
    fn find_hyperlinks_detects_printed_url() {
        let shell = tty::Shell::new("/bin/sh".to_string(), Vec::new());
        let (terminal, _events) =
            Terminal::spawn_with_shell(shell, small_size()).expect("failed to spawn PTY shell");
        terminal.write_input(b"printf 'https://example.com/foo\\n'\n".as_slice());
        wait_for_marker(&terminal, "https://example.com");

        let links = terminal.find_hyperlinks();
        terminal.shutdown();
        assert!(
            links
                .iter()
                .any(|l| l.is_url && l.text.contains("https://example.com")),
            "expected a URL hyperlink match, got {links:?}"
        );
    }

    /// Bracketed-paste injection guard: an embedded end-marker in pasted text
    /// is stripped so it can't terminate paste mode early.
    #[test]
    fn strip_paste_terminator_removes_embedded_end_marker() {
        assert_eq!(
            strip_paste_terminator("safe\x1b[201~rm -rf /\n"),
            "saferm -rf /\n"
        );
        assert_eq!(strip_paste_terminator("no marker here"), "no marker here");
    }

    /// A freshly spawned `/bin/sh` (no readline bracketed-paste) does not have
    /// `BRACKETED_PASTE` set — proves `mode()` reads real terminal state.
    #[test]
    fn mode_reads_bracketed_paste_state() {
        let shell = tty::Shell::new("/bin/sh".to_string(), Vec::new());
        let (terminal, _events) =
            Terminal::spawn_with_shell(shell, small_size()).expect("failed to spawn PTY shell");
        let mode = terminal.mode();
        terminal.shutdown();
        assert!(
            !mode.contains(TermMode::BRACKETED_PASTE),
            "plain /bin/sh should not enable bracketed paste"
        );
    }
}
