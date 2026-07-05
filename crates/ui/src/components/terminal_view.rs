use std::cell::Cell;
use std::ops::Range;
use std::rc::Rc;

use gpui::{
    AnyElement, Bounds, Context, FocusHandle, Focusable, HighlightStyle, KeyDownEvent, Keystroke,
    Render, StyledText, canvas, rgb,
};

use crate::prelude::*;

/// Monospace font used for terminal output (matches [`crate::CodeEditor`]'s
/// `CODE_FONT_FAMILY` so panes in the same [`crate::PaneGroup`] look
/// consistent).
const TERMINAL_FONT_FAMILY: &str = "IBM Plex Mono";
const TERMINAL_FONT_SIZE: Pixels = px(12.5);
const TERMINAL_LINE_HEIGHT: f32 = 1.5;
/// Approximate cell pixel size at `TERMINAL_FONT_SIZE`/`TERMINAL_FONT_FAMILY`
/// (monospace, so a single advance width applies to every glyph). Used both
/// to seed the initial PTY size and to convert the pane's measured pixel
/// bounds into a row/column count on resize — an approximation, not a real
/// text-shaping measurement, so it can be off by a cell or two vs. what the
/// font actually renders at. Good enough for a terminal grid (unlike, say,
/// cursor-position math in a real text editor).
const CELL_WIDTH: u16 = 8;
const CELL_HEIGHT: u16 = 18;
const DEFAULT_ROWS: u16 = 24;
const DEFAULT_COLUMNS: u16 = 80;

/// Encodes a keystroke into the bytes a real terminal program expects on
/// stdin. Covers printable characters, Enter/Backspace/Tab/Escape, arrows,
/// Home/End/PageUp/PageDown, F1-F12, and Ctrl+letter control codes. NOT
/// covered (unlike a real terminal emulator): Option-as-Meta on macOS
/// (Zed's `mappings/keys.rs` handles this via a settings-driven toggle —
/// out of scope here, this always sends the plain character), bracketed-
/// paste mode, modified arrow/function keys (e.g. Shift+F5). `cmd`/
/// `platform`-modified keystrokes are passed through (returns `None`) so
/// OS-level shortcuts on the surrounding app keep working instead of being
/// swallowed by the terminal.
fn encode_keystroke(keystroke: &Keystroke) -> Option<Vec<u8>> {
    if keystroke.modifiers.platform {
        return None;
    }
    if keystroke.modifiers.control
        && let Some(ch) = keystroke.key.chars().next()
        && ch.is_ascii_alphabetic()
    {
        let code = ch.to_ascii_uppercase() as u8 - b'A' + 1;
        return Some(vec![code]);
    }
    match keystroke.key.as_str() {
        "enter" => Some(b"\r".to_vec()),
        "backspace" => Some(b"\x7f".to_vec()),
        "tab" => Some(b"\t".to_vec()),
        "escape" => Some(b"\x1b".to_vec()),
        "up" => Some(b"\x1b[A".to_vec()),
        "down" => Some(b"\x1b[B".to_vec()),
        "right" => Some(b"\x1b[C".to_vec()),
        "left" => Some(b"\x1b[D".to_vec()),
        "home" => Some(b"\x1b[H".to_vec()),
        "end" => Some(b"\x1b[F".to_vec()),
        "pageup" => Some(b"\x1b[5~".to_vec()),
        "pagedown" => Some(b"\x1b[6~".to_vec()),
        "delete" => Some(b"\x1b[3~".to_vec()),
        "insert" => Some(b"\x1b[2~".to_vec()),
        "f1" => Some(b"\x1bOP".to_vec()),
        "f2" => Some(b"\x1bOQ".to_vec()),
        "f3" => Some(b"\x1bOR".to_vec()),
        "f4" => Some(b"\x1bOS".to_vec()),
        "f5" => Some(b"\x1b[15~".to_vec()),
        "f6" => Some(b"\x1b[17~".to_vec()),
        "f7" => Some(b"\x1b[18~".to_vec()),
        "f8" => Some(b"\x1b[19~".to_vec()),
        "f9" => Some(b"\x1b[20~".to_vec()),
        "f10" => Some(b"\x1b[21~".to_vec()),
        "f11" => Some(b"\x1b[23~".to_vec()),
        "f12" => Some(b"\x1b[24~".to_vec()),
        "space" => Some(b" ".to_vec()),
        _ => keystroke
            .key_char
            .as_ref()
            .map(|text| text.as_bytes().to_vec()),
    }
}

fn default_terminal_size() -> terminal::TerminalSize {
    terminal::TerminalSize {
        rows: DEFAULT_ROWS,
        columns: DEFAULT_COLUMNS,
        cell_width: CELL_WIDTH,
        cell_height: CELL_HEIGHT,
    }
}

/// Converts measured pane pixel bounds into a row/column grid size, using
/// the fixed [`CELL_WIDTH`]/[`CELL_HEIGHT`] approximation. Never returns
/// zero rows/columns (a 0x0 PTY size is nonsensical and some programs
/// divide by it) — floors at 1x1.
fn size_for_bounds(bounds: Bounds<Pixels>) -> terminal::TerminalSize {
    let columns = (f32::from(bounds.size.width) / CELL_WIDTH as f32).floor() as u16;
    let rows = (f32::from(bounds.size.height) / CELL_HEIGHT as f32).floor() as u16;
    terminal::TerminalSize {
        rows: rows.max(1),
        columns: columns.max(1),
        cell_width: CELL_WIDTH,
        cell_height: CELL_HEIGHT,
    }
}

fn rgb_to_hsla(color: terminal::Rgb) -> gpui::Hsla {
    gpui::rgb(((color.r as u32) << 16) | ((color.g as u32) << 8) | color.b as u32).into()
}

/// Builds one combined multi-line string plus a coalesced list of
/// `(byte range, HighlightStyle)` spans from styled terminal cells — the
/// same shape [`crate::CodeEditor`]'s tree-sitter highlighting feeds into
/// `StyledText::with_highlights`. Adjacent cells sharing identical style
/// are merged into a single span instead of emitting one per character,
/// which would otherwise be thousands of spans for a full screen.
fn styled_screen_text(
    rows: Vec<Vec<terminal::TerminalCell>>,
) -> (String, Vec<(Range<usize>, HighlightStyle)>) {
    let mut text = String::new();
    let mut highlights = Vec::new();

    for (row_ix, row) in rows.into_iter().enumerate() {
        if row_ix > 0 {
            text.push('\n');
        }

        let mut span_start = text.len();
        let mut current_style: Option<(Option<terminal::Rgb>, bool, bool, bool)> = None;

        for cell in row {
            let style_key = (cell.fg, cell.bold, cell.italic, cell.underline);
            if current_style != Some(style_key) {
                if let Some((fg, bold, italic, underline)) = current_style
                    && text.len() > span_start
                {
                    highlights.push((
                        span_start..text.len(),
                        cell_highlight_style(fg, bold, italic, underline),
                    ));
                }
                span_start = text.len();
                current_style = Some(style_key);
            }
            text.push(cell.text);
        }

        if let Some((fg, bold, italic, underline)) = current_style
            && text.len() > span_start
        {
            highlights.push((
                span_start..text.len(),
                cell_highlight_style(fg, bold, italic, underline),
            ));
        }
    }

    (text, highlights)
}

fn cell_highlight_style(
    fg: Option<terminal::Rgb>,
    bold: bool,
    italic: bool,
    underline: bool,
) -> HighlightStyle {
    HighlightStyle {
        color: fg.map(rgb_to_hsla),
        font_weight: bold.then_some(gpui::FontWeight::BOLD),
        font_style: italic.then_some(gpui::FontStyle::Italic),
        underline: underline.then_some(gpui::UnderlineStyle::default()),
        ..Default::default()
    }
}

/// A real terminal: spawns the user's shell through a PTY
/// ([`terminal::Terminal`]) and renders its live output with real per-cell
/// ANSI colors/bold/italic/underline. This is the Phase C counterpart to
/// [`crate::TerminalPanel`]'s static chrome — see `plans/20260705-1722-zed-
/// ui-component-enrichment/phase-03-real-terminal-pty-and-text-buffer.md`
/// for what this intentionally does NOT cover: mouse support, hyperlinks,
/// scrollback UI (only the visible screen is rendered, no history buffer),
/// vi-mode, bracketed paste, and any verification on Linux/Windows (this
/// was built and tested on macOS only).
///
/// If the PTY fails to spawn (e.g. a sandboxed/headless host with no
/// controlling TTY), renders an inline error instead of panicking.
///
/// Stateful view — create with `cx.new(|cx| TerminalView::new(cx))` and
/// store the resulting `Entity<TerminalView>`.
pub struct TerminalView {
    terminal: Option<terminal::Terminal>,
    spawn_error: Option<String>,
    focus_handle: FocusHandle,
    /// Measured by a `canvas` child in `render` — read back at the START of
    /// the NEXT render (same one-frame-lag pattern `PaneGroup`/
    /// `ResizablePanelGroup` already use for their own bounds tracking) to
    /// decide whether the PTY needs `Terminal::resize`.
    container_bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
}

impl TerminalView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        match terminal::Terminal::spawn(default_terminal_size()) {
            Ok((terminal, events)) => {
                cx.spawn(async move |this, cx| {
                    while events.recv().await.is_ok() {
                        if this.update(cx, |_, cx| cx.notify()).is_err() {
                            break;
                        }
                    }
                })
                .detach();
                Self {
                    terminal: Some(terminal),
                    spawn_error: None,
                    focus_handle,
                    container_bounds: Rc::new(Cell::new(None)),
                }
            }
            Err(error) => Self {
                terminal: None,
                spawn_error: Some(error.to_string()),
                focus_handle,
                container_bounds: Rc::new(Cell::new(None)),
            },
        }
    }

    /// Moves keyboard focus onto the terminal so typed keys reach the PTY.
    /// Callers should invoke this once after mounting (e.g. on click),
    /// matching `TabSwitcher::focus`/`CommandPalette::focus_input`'s
    /// convention in this crate.
    pub fn focus(&self, window: &mut Window, cx: &mut App) {
        window.focus(&self.focus_handle, cx);
    }

    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(terminal) = &self.terminal else {
            return;
        };
        if let Some(bytes) = encode_keystroke(&event.keystroke) {
            terminal.write_input(bytes);
            cx.notify();
        }
    }

    /// Resizes the PTY if the last-measured container bounds imply a
    /// different row/column count than the terminal currently has. A
    /// no-op (skips `Terminal::resize`, which would otherwise send a
    /// `SIGWINCH`-equivalent on every single render) when the size hasn't
    /// actually changed.
    fn sync_size_to_container(&self) {
        let Some(terminal) = &self.terminal else {
            return;
        };
        let Some(bounds) = self.container_bounds.get() else {
            return;
        };
        let wanted = size_for_bounds(bounds);
        let current = terminal.current_size();
        if wanted.rows != current.rows || wanted.columns != current.columns {
            terminal.resize(wanted);
        }
    }
}

impl Focusable for TerminalView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TerminalView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_size_to_container();

        let body: AnyElement = if let Some(error) = &self.spawn_error {
            div()
                .text_color(rgb(0xE06C75))
                .child(format!("Failed to start terminal: {error}"))
                .into_any_element()
        } else {
            let (text, highlights) = self
                .terminal
                .as_ref()
                .map(|terminal| styled_screen_text(terminal.screen_cells()))
                .unwrap_or_default();
            StyledText::new(text)
                .with_highlights(highlights)
                .into_any_element()
        };

        let measure = self.container_bounds.clone();

        div()
            .id(("terminal-view", cx.entity_id()))
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_key_down))
            .relative()
            .w_full()
            .h_full()
            .overflow_y_scroll()
            .p_3()
            .bg(rgb(0x0D1117))
            .text_color(rgb(0xC9D1D9))
            .font_family(TERMINAL_FONT_FAMILY)
            .text_size(TERMINAL_FONT_SIZE)
            .line_height(relative(TERMINAL_LINE_HEIGHT))
            .child(
                canvas(
                    move |bounds, _, _| measure.set(Some(bounds)),
                    |_, _, _, _| {},
                )
                .absolute()
                .size_full(),
            )
            .child(body)
    }
}

/// Standalone gallery preview for `TerminalView` (not registered in the
/// `Component` catalog since it is a stateful `Entity` that spawns a real
/// process, matching `CodeEditor`/`TabSwitcher`'s existing convention in
/// this crate — but unlike those, mounting this in a gallery genuinely
/// spawns a real shell child process for as long as the entity is alive).
pub fn terminal_view_preview(_window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .h(px(280.))
        .rounded_lg()
        .overflow_hidden()
        .child(cx.new(|cx| TerminalView::new(cx)))
        .into_any_element()
}
