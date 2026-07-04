use gpui::{Context, FocusHandle, Focusable, KeyDownEvent, MouseButton, Render};

use crate::prelude::*;

/// Fixed-length OTP input with per-slot focus navigation and paste-split.
///
/// Paste handling splits sanitized alphanumeric characters across remaining
/// slots starting at the focused slot (not only slot 0). Tab and arrow keys
/// move focus between slots.
///
/// Stateful view — create with `cx.new(|cx| InputOtp::new(cx, 6))`.
#[derive(RegisterComponent)]
pub struct InputOtp {
    slots: Vec<SharedString>,
    focus_index: usize,
    focus_handle: FocusHandle,
    disabled: bool,
}

impl InputOtp {
    pub fn new(cx: &mut Context<Self>, length: usize) -> Self {
        let length = length.clamp(1, 12);
        Self {
            slots: vec![SharedString::default(); length],
            focus_index: 0,
            focus_handle: cx.focus_handle(),
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn value(&self) -> String {
        self.slots.iter().map(|s| s.as_ref()).collect()
    }

    pub fn set_value(&mut self, value: impl AsRef<str>, cx: &mut Context<Self>) {
        let chars: Vec<char> = value
            .as_ref()
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .take(self.slots.len())
            .collect();
        for (i, slot) in self.slots.iter_mut().enumerate() {
            *slot = chars
                .get(i)
                .map(|c| c.to_string().into())
                .unwrap_or_default();
        }
        self.focus_index = chars.len().min(self.slots.len().saturating_sub(1));
        cx.notify();
    }

    fn move_focus(&mut self, delta: isize, cx: &mut Context<Self>) {
        let len = self.slots.len() as isize;
        let next = (self.focus_index as isize + delta).clamp(0, len - 1) as usize;
        if next != self.focus_index {
            self.focus_index = next;
            cx.notify();
        }
    }

    fn fill_from_index(&mut self, text: &str, start: usize, cx: &mut Context<Self>) {
        let sanitized: Vec<char> = text.chars().filter(|c| c.is_ascii_alphanumeric()).collect();
        let mut idx = start;
        for c in sanitized {
            if idx >= self.slots.len() {
                break;
            }
            self.slots[idx] = c.to_string().into();
            idx += 1;
        }
        if idx > start {
            self.focus_index = idx.min(self.slots.len().saturating_sub(1));
            cx.notify();
        }
    }

    fn on_key_down(&mut self, event: &KeyDownEvent, _window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        let keystroke = &event.keystroke;

        if keystroke.modifiers.platform || keystroke.modifiers.control {
            if keystroke.key.as_str() == "v" {
                if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                    self.fill_from_index(&text, self.focus_index, cx);
                }
            }
            return;
        }

        match keystroke.key.as_str() {
            "left" | "backtab" => self.move_focus(-1, cx),
            "right" | "tab" => self.move_focus(1, cx),
            "backspace" => {
                if !self.slots[self.focus_index].is_empty() {
                    self.slots[self.focus_index] = SharedString::default();
                } else if self.focus_index > 0 {
                    self.focus_index -= 1;
                    self.slots[self.focus_index] = SharedString::default();
                }
                cx.notify();
            }
            _ => {
                if let Some(text) = &keystroke.key_char {
                    let mut chars = text.chars().filter(|c| c.is_ascii_alphanumeric());
                    if let Some(c) = chars.next() {
                        self.slots[self.focus_index] = c.to_string().into();
                        if self.focus_index + 1 < self.slots.len() {
                            self.focus_index += 1;
                        }
                        cx.notify();
                    }
                }
            }
        }
    }
}

impl Focusable for InputOtp {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for InputOtp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let focused = self.focus_handle.is_focused(window);
        let focus_index = self.focus_index;
        let disabled = self.disabled;
        let slots = self.slots.clone();

        let mut row = h_flex().gap_2().items_center();
        for (i, slot) in slots.iter().enumerate() {
            let display = if slot.is_empty() { " " } else { slot.as_ref() };
            let slot_focused = focused && i == focus_index;
            row = row.child(
                div()
                    .id(("otp-slot", i))
                    .w(px(40.))
                    .h(px(44.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded_md()
                    .border_1()
                    .border_color(if slot_focused {
                        palette::primary(500)
                    } else {
                        semantic::border(cx)
                    })
                    .bg(semantic::surface(cx))
                    .text_ui(cx)
                    .text_color(semantic::text(cx))
                    .when(!disabled, |this| {
                        this.cursor_pointer().on_mouse_down(
                            MouseButton::Left,
                            cx.listener(move |this, _, window, cx| {
                                this.focus_index = i;
                                window.focus(&this.focus_handle, cx);
                                cx.notify();
                            }),
                        )
                    })
                    .child(Label::new(display).size(LabelSize::Large)),
            );
            if i == 2 && slots.len() > 4 {
                row = row.child(Label::new("—").color(Color::Muted));
            }
        }

        let field = div()
            .id("input-otp")
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::on_key_down))
            .when(disabled, |this| this.opacity(0.5))
            .child(row);

        focus_ring(field, focused, palette::primary(500))
    }
}

impl Component for InputOtp {
    fn scope() -> ComponentScope {
        ComponentScope::Input
    }

    fn description() -> Option<&'static str> {
        Some("A fixed-length one-time-password input with slot focus navigation and paste-split.")
    }

    fn preview(_window: &mut Window, cx: &mut App) -> Option<AnyElement> {
        Some(
            v_flex()
                .gap_4()
                .child(cx.new(|cx| InputOtp::new(cx, 6)))
                .into_any_element(),
        )
    }
}
