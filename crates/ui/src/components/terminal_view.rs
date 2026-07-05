use gpui::{AnyElement, Context, FocusHandle, Focusable, KeyDownEvent, Keystroke, Render, rgb};

use crate::prelude::*;

/// Monospace font used for terminal output (matches [`crate::CodeEditor`]'s
/// `CODE_FONT_FAMILY` so panes in the same [`crate::PaneGroup`] look
/// consistent).
const TERMINAL_FONT_FAMILY: &str = "IBM Plex Mono";
const TERMINAL_FONT_SIZE: Pixels = px(12.5);
const TERMINAL_LINE_HEIGHT: f32 = 1.5;
/// Fixed grid size for this pass — live resize-to-pane-size is a follow-up
/// (see this file's module doc).
const DEFAULT_ROWS: u16 = 24;
const DEFAULT_COLUMNS: u16 = 80;

/// Encodes a keystroke into the bytes a real terminal program expects on
/// stdin. Intentionally minimal: printable characters, Enter/Backspace/Tab/
/// Escape/arrows, and Ctrl+letter control codes. NOT covered (unlike a real
/// terminal emulator): Alt-as-Meta key combos, function keys, Option-as-Meta
/// on macOS (Zed's `mappings/keys.rs` handles this — out of scope here),
/// bracketed-paste mode. `cmd`/`platform`-modified keystrokes are passed
/// through (returns `None`) so OS-level shortcuts on the surrounding app
/// keep working instead of being swallowed by the terminal.
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
        // Approximate cell pixel size at `TERMINAL_FONT_SIZE` — only used by
        // `alacritty_terminal` to report `WindowSize` to the PTY (some
        // programs query it), not for actual rendering, since this pass
        // renders plain text lines rather than a real per-cell grid.
        cell_width: 8,
        cell_height: 18,
    }
}

/// A real terminal: spawns the user's shell through a PTY
/// ([`terminal::Terminal`]) and renders its live output as plain monospace
/// text. This is the Phase C counterpart to [`crate::TerminalPanel`]'s
/// static chrome — see `plans/20260705-1722-zed-ui-component-enrichment/
/// phase-03-real-terminal-pty-and-text-buffer.md` for what this
/// intentionally does NOT cover yet: real per-cell ANSI color rendering
/// (output is monochrome plain text — `terminal::Terminal::screen_lines`
/// drops color/attribute state by design), mouse support, hyperlinks,
/// live resize-to-pane-size (grid is fixed at 24x80), and any verification
/// on Linux/Windows (this was built and tested on macOS only).
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
                }
            }
            Err(error) => Self {
                terminal: None,
                spawn_error: Some(error.to_string()),
                focus_handle,
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
}

impl Focusable for TerminalView {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TerminalView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let body: AnyElement = if let Some(error) = &self.spawn_error {
            div()
                .text_color(rgb(0xE06C75))
                .child(format!("Failed to start terminal: {error}"))
                .into_any_element()
        } else {
            let lines = self
                .terminal
                .as_ref()
                .map(terminal::Terminal::screen_lines)
                .unwrap_or_default();
            v_flex()
                .children(lines.into_iter().map(|line| div().child(line)))
                .into_any_element()
        };

        div()
            .id(("terminal-view", cx.entity_id()))
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_key_down))
            .w_full()
            .h_full()
            .overflow_y_scroll()
            .p_3()
            .bg(rgb(0x0D1117))
            .text_color(rgb(0xC9D1D9))
            .font_family(TERMINAL_FONT_FAMILY)
            .text_size(TERMINAL_FONT_SIZE)
            .line_height(relative(TERMINAL_LINE_HEIGHT))
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
