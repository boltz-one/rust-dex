use std::cell::Cell;
use std::rc::Rc;

use gpui::{
    Bounds, Context, Entity, Focusable, KeyDownEvent, MouseButton, Pixels, Render, SharedString,
    canvas,
};

use crate::components::ai::completion_popover::{
    CompletionItem, CompletionPopover, filter_completions,
};
use crate::prelude::*;
use crate::{InputGroup, TextInput};

/// Locates the `/`-command token currently being typed: the last
/// whitespace-delimited word in `text`, if it starts with `/`. Returns the
/// query substring after the `/` (empty for a bare `/`).
fn detect_command_query(text: &str) -> Option<&str> {
    let token_start = text
        .rfind(char::is_whitespace)
        .map(|ix| ix + 1)
        .unwrap_or(0);
    text[token_start..].strip_prefix('/')
}

/// Replaces the in-progress `/`-command token (see [`detect_command_query`])
/// with `insert_text`, followed by a trailing space.
fn apply_completion(text: &str, insert_text: &str) -> String {
    let token_start = text
        .rfind(char::is_whitespace)
        .map(|ix| ix + 1)
        .unwrap_or(0);
    let mut result = text[..token_start].to_string();
    result.push_str(insert_text);
    result.push(' ');
    result
}

/// A multiline agent chat input: a multiline `TextInput` inside `InputGroup`
/// chrome with a send button, plus a `/`-command [`CompletionPopover`]
/// triggered by typing `/` at the start of a word. Enter submits (clearing
/// the draft); Shift/Ctrl/Cmd+Enter inserts a newline.
///
/// This input has no cursor-position tracking beyond "typed so far" (see
/// `TextInput`), so `/`-completion is scoped to the trailing token of the
/// whole buffer rather than the token under a cursor — a pragmatic v1 limit,
/// not a full replication of a cursor-aware editor's completion scoping.
///
/// Stateful view — create with `cx.new(|cx| AgentChatInput::new(cx, ..))`.
pub struct AgentChatInput {
    input: Entity<TextInput>,
    sending: bool,
    commands: Vec<CompletionItem>,
    completion_open: bool,
    completion_selected_ix: usize,
    /// Real screen bounds of the input row, captured via an invisible
    /// `canvas()` measurement child every render and read back on the
    /// *next* render to position the completion popover (see
    /// `Combobox::trigger_bounds` for the full rationale).
    input_bounds: Rc<Cell<Option<Bounds<Pixels>>>>,
    on_submit: Option<Rc<dyn Fn(SharedString, &mut Window, &mut App) + 'static>>,
    /// Focus the text field on the next render (once), so opening the input
    /// leaves the caret ready to type without a click first.
    focus_pending: bool,
}

impl AgentChatInput {
    pub fn new(cx: &mut Context<Self>, placeholder: impl Into<SharedString>) -> Self {
        let input = cx.new(|cx| {
            TextInput::new(cx)
                .multiline(true)
                .submit_on_enter(true)
                .placeholder(placeholder)
        });
        cx.observe(&input, |_, _, cx| cx.notify()).detach();
        Self {
            input,
            sending: false,
            commands: Vec::new(),
            completion_open: false,
            completion_selected_ix: 0,
            input_bounds: Rc::new(Cell::new(None)),
            on_submit: None,
            focus_pending: true,
        }
    }

    /// Disables the send button while `true`.
    pub fn sending(mut self, sending: bool) -> Self {
        self.sending = sending;
        self
    }

    /// Registers a callback fired with the drafted text when the user
    /// submits (plain Enter, or clicking the send button). The draft is
    /// cleared immediately after.
    pub fn on_submit(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_submit = Some(Rc::new(handler));
        self
    }

    /// Supplies the `/`-command list offered by the completion popover. The
    /// caller sources these however it likes (e.g. from an agent runtime
    /// event) — `boltz-ui` takes plain [`CompletionItem`]s only.
    pub fn set_commands(&mut self, commands: Vec<CompletionItem>, cx: &mut Context<Self>) {
        self.commands = commands;
        cx.notify();
    }

    /// The current draft text.
    pub fn text(&self, cx: &App) -> SharedString {
        self.input.read(cx).text().to_string().into()
    }

    fn submit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let text = self.input.read(cx).text().to_string();
        if text.trim().is_empty() {
            return;
        }
        if let Some(on_submit) = self.on_submit.clone() {
            on_submit(text.into(), window, cx);
        }
        self.input.update(cx, |input, cx| input.clear(cx));
        self.completion_open = false;
        cx.notify();
    }

    fn insert_completion(
        &mut self,
        insert_text: SharedString,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let text = self.input.read(cx).text().to_string();
        let new_text = apply_completion(&text, &insert_text);
        self.input
            .update(cx, |input, cx| input.set_text(new_text, cx));
        self.completion_open = false;
        self.completion_selected_ix = 0;
        let focus_handle = self.input.read(cx).focus_handle(cx);
        window.focus(&focus_handle, cx);
        cx.notify();
    }

    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let text = self.input.read(cx).text().to_string();
        let query = detect_command_query(&text).map(str::to_string);
        self.completion_open = query.is_some();

        if let Some(query) = query {
            let matched = filter_completions(&self.commands, &query);
            match event.keystroke.key.as_str() {
                "up" if !matched.is_empty() => {
                    self.completion_selected_ix = if self.completion_selected_ix == 0 {
                        matched.len() - 1
                    } else {
                        self.completion_selected_ix - 1
                    };
                }
                "down" if !matched.is_empty() => {
                    self.completion_selected_ix = (self.completion_selected_ix + 1) % matched.len();
                }
                "enter" => {
                    if let Some(item) = matched.get(self.completion_selected_ix) {
                        let insert_text = item.insert_text.clone();
                        self.insert_completion(insert_text, window, cx);
                    }
                }
                "escape" => self.completion_open = false,
                _ => {}
            }
        } else {
            let modifiers = &event.keystroke.modifiers;
            let plain_enter = event.keystroke.key == "enter"
                && !modifiers.shift
                && !modifiers.control
                && !modifiers.platform;
            if plain_enter {
                self.submit(window, cx);
            }
        }
        cx.notify();
    }
}

impl Render for AgentChatInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.focus_pending {
            self.focus_pending = false;
            let focus_handle = self.input.read(cx).focus_handle(cx);
            window.focus(&focus_handle, cx);
        }
        let text = self.input.read(cx).text().to_string();
        let query = detect_command_query(&text).map(str::to_string);
        let disabled = self.sending || text.trim().is_empty();

        let send_button = IconButton::new("agent-chat-send", IconName::PlayFilled)
            .icon_size(IconSize::Small)
            .disabled(disabled)
            .on_click(cx.listener(|this, _event, window, cx| {
                this.submit(window, cx);
            }));

        let input_bounds = self.input_bounds.clone();
        let focus_handle = self.input.read(cx).focus_handle(cx);
        let field = div()
            .id("agent-chat-input-field")
            .relative()
            .w_full()
            .min_h(px(64.))
            .on_key_down(cx.listener(Self::handle_key_down))
            .on_mouse_down(MouseButton::Left, {
                let focus_handle = focus_handle.clone();
                move |_event, window, cx| {
                    window.focus(&focus_handle, cx);
                }
            })
            .child(self.input.clone())
            .child(
                canvas(
                    move |bounds, _window, _cx| input_bounds.set(Some(bounds)),
                    |_bounds, _state, _window, _cx| {},
                )
                .absolute()
                .top_0()
                .left_0()
                .size_full(),
            );

        let mut root = v_flex()
            .id("agent-chat-input")
            .w_full()
            .gap_1()
            .child(InputGroup::new(field).trailing(send_button));

        if let Some(query) = query
            && self.completion_open
            && let Some(bounds) = self.input_bounds.get()
        {
            let entity = cx.entity();
            let popover = CompletionPopover::new(
                self.commands.clone(),
                query,
                self.completion_selected_ix,
                bounds,
                move |insert_text, window, cx| {
                    entity.update(cx, |this, cx| {
                        this.insert_completion(insert_text, window, cx)
                    });
                },
            )
            .render(cx);
            root = root.child(popover);
        }

        root
    }
}

/// Standalone gallery preview for `AgentChatInput` (not registered in the
/// `Component` catalog since it is a stateful `Entity`, matching
/// `Combobox`/`Select`'s existing convention in this crate).
pub fn agent_chat_input_preview(_window: &mut Window, cx: &mut App) -> AnyElement {
    cx.new(|cx| {
        let mut input = AgentChatInput::new(cx, "Ask anything…");
        input.set_commands(
            vec![
                CompletionItem::new("help", "/help").description("Show available commands"),
                CompletionItem::new("explain", "/explain").description("Explain the selection"),
                CompletionItem::new("tests", "/tests").description("Generate tests"),
            ],
            cx,
        );
        input
    })
    .into_any_element()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_query_on_plain_text() {
        assert_eq!(detect_command_query("hello world"), None);
    }

    #[test]
    fn command_query_detected_at_start() {
        assert_eq!(detect_command_query("/run"), Some("run"));
    }

    #[test]
    fn command_query_detected_after_whitespace() {
        assert_eq!(detect_command_query("hello /run"), Some("run"));
    }

    #[test]
    fn bare_slash_has_empty_query() {
        assert_eq!(detect_command_query("/"), Some(""));
    }

    #[test]
    fn command_followed_by_space_is_not_in_progress() {
        assert_eq!(detect_command_query("/run tests"), None);
    }

    #[test]
    fn apply_completion_replaces_trailing_token() {
        assert_eq!(apply_completion("hello /ru", "/run"), "hello /run ");
    }

    #[test]
    fn apply_completion_on_bare_slash() {
        assert_eq!(apply_completion("/", "/help"), "/help ");
    }
}
