use gpui::StyledText;

use crate::prelude::*;

/// Monospace font for captured terminal output (mirrors `code_editor.rs`'s
/// `CODE_FONT_FAMILY` constant, duplicated locally since that constant is
/// private to its module).
const TERMINAL_OUTPUT_FONT_FAMILY: &str = "IBM Plex Mono";
const TERMINAL_OUTPUT_FONT_SIZE: Pixels = px(12.5);

/// Renders a tool call's captured terminal output as static text (Decision
/// #2b: no live PTY grid — see this component's call sites for that
/// trade-off's rationale). Sourced directly from already-captured
/// command/raw-output strings; this has no cursor, resize, or live-stream
/// semantics.
#[derive(IntoElement, RegisterComponent)]
pub struct TerminalOutputBlock {
    id: ElementId,
    command: Option<SharedString>,
    raw_output: SharedString,
}

impl TerminalOutputBlock {
    pub fn new(id: impl Into<ElementId>, raw_output: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            command: None,
            raw_output: raw_output.into(),
        }
    }

    /// The command line that produced `raw_output`, shown as a small header.
    pub fn command(mut self, command: impl Into<SharedString>) -> Self {
        self.command = Some(command.into());
        self
    }
}

impl RenderOnce for TerminalOutputBlock {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        v_flex()
            .id(self.id)
            .w_full()
            .gap_1()
            .p_2()
            .rounded_md()
            .bg(cx.theme().colors().editor_background)
            .when_some(self.command, |this, command| {
                this.child(
                    h_flex()
                        .gap_1()
                        .child(
                            Icon::new(IconName::ToolTerminal)
                                .size(IconSize::XSmall)
                                .color(Color::Muted),
                        )
                        .child(
                            div()
                                .font_family(TERMINAL_OUTPUT_FONT_FAMILY)
                                .text_size(TERMINAL_OUTPUT_FONT_SIZE)
                                .text_color(cx.theme().colors().text_muted)
                                .child(command),
                        ),
                )
            })
            .child(
                div()
                    .font_family(TERMINAL_OUTPUT_FONT_FAMILY)
                    .text_size(TERMINAL_OUTPUT_FONT_SIZE)
                    .text_color(cx.theme().colors().text)
                    .child(StyledText::new(self.raw_output)),
            )
    }
}

impl Component for TerminalOutputBlock {
    fn scope() -> ComponentScope {
        ComponentScope::Agent
    }

    fn description() -> Option<&'static str> {
        Some(
            "Renders captured terminal output as static text (no PTY grid) \
             for a tool-call's terminal content, with an optional command header.",
        )
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        Some(
            v_flex()
                .w_96()
                .gap_4()
                .child(single_example(
                    "With command",
                    TerminalOutputBlock::new(
                        "term-preview-1",
                        "Compiling boltz-ui v0.2.11\n    Finished dev profile in 3.21s",
                    )
                    .command("cargo build -p boltz-ui")
                    .into_any_element(),
                ))
                .into_any_element(),
        )
    }
}
