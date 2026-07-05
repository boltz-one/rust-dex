use gpui::{AnyElement, Context, Render, rgb};

use crate::prelude::*;

/// Monospace font used for terminal output (matches [`crate::CodeEditor`]'s
/// `CODE_FONT_FAMILY` so panes in the same [`crate::PaneGroup`] look
/// consistent).
const TERMINAL_FONT_FAMILY: &str = "IBM Plex Mono";
const TERMINAL_FONT_SIZE: Pixels = px(12.5);
const TERMINAL_LINE_HEIGHT: f32 = 1.5;

/// Chrome-only terminal panel: renders whatever text is set via
/// [`Self::set_output`]/[`Self::append_output`] in a monospace block with a
/// dark terminal-style background. There is NO real process behind this —
/// no PTY, no shell spawn, no ANSI escape-sequence parsing. That is Phase C
/// (`plans/20260705-1722-zed-ui-component-enrichment/phase-03-*.md`), which
/// requires `alacritty_terminal` + platform PTY syscalls routed through
/// `gpui_platform` and is gated on the user accepting that dependency and
/// the cross-platform risk it carries (see the plan's Unresolved Questions).
/// This exists purely so a `PaneGroup` layout can be composed and reviewed
/// before Phase C lands.
///
/// Stateful view — create with `cx.new(|_| TerminalPanel::new())` and store
/// the resulting `Entity<TerminalPanel>`.
pub struct TerminalPanel {
    output: SharedString,
}

impl TerminalPanel {
    pub fn new() -> Self {
        Self {
            output: SharedString::default(),
        }
    }

    /// Replaces the displayed output entirely.
    pub fn set_output(&mut self, output: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.output = output.into();
        cx.notify();
    }

    /// Appends a line to the displayed output (e.g. echoing a command a
    /// caller wants to simulate having "run").
    pub fn append_output(&mut self, line: impl AsRef<str>, cx: &mut Context<Self>) {
        let mut output = self.output.to_string();
        if !output.is_empty() {
            output.push('\n');
        }
        output.push_str(line.as_ref());
        self.output = output.into();
        cx.notify();
    }
}

impl Default for TerminalPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl Render for TerminalPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id(("terminal-panel", cx.entity_id()))
            .w_full()
            .h_full()
            .overflow_y_scroll()
            .p_3()
            .bg(rgb(0x0D1117))
            .text_color(rgb(0xC9D1D9))
            .font_family(TERMINAL_FONT_FAMILY)
            .text_size(TERMINAL_FONT_SIZE)
            .line_height(relative(TERMINAL_LINE_HEIGHT))
            .child(if self.output.is_empty() {
                div()
                    .text_color(rgb(0x545D68))
                    .child("No output.")
                    .into_any_element()
            } else {
                self.output.clone().into_any_element()
            })
    }
}

/// Standalone gallery preview for `TerminalPanel` (not registered in the
/// `Component` catalog since it is a stateful `Entity`, matching
/// `CodeEditor`/`SearchInput`'s existing convention in this crate).
pub fn terminal_panel_preview(_window: &mut Window, cx: &mut App) -> AnyElement {
    div()
        .h(px(200.))
        .rounded_lg()
        .overflow_hidden()
        .child(cx.new(|cx| {
            let mut panel = TerminalPanel::new();
            panel.set_output(
                "$ echo \"chrome only — no PTY yet\"\nchrome only — no PTY yet\n$ ",
                cx,
            );
            panel
        }))
        .into_any_element()
}
