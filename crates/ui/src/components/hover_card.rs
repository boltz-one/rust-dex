use std::rc::Rc;

use gpui::{Context, Render, StatefulInteractiveElement};

use crate::{Popover, prelude::*};

struct HoverCardTooltip {
    content_builder: Rc<dyn Fn(&mut Window, &mut App) -> AnyElement + 'static>,
}

impl Render for HoverCardTooltip {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content = (self.content_builder)(window, cx);
        Popover::new().child(div().max_w(px(320.)).p_3().child(content))
    }
}

/// Rich content shown on hover with delay, composing tooltip hover timing and
/// [`Popover`] content styling.
///
/// Uses GPUI's `hoverable_tooltip` (500ms show delay, hoverable content) rather
/// than inventing a third overlay primitive.
#[derive(IntoElement, RegisterComponent)]
pub struct HoverCard {
    trigger: AnyElement,
    content_builder: Rc<dyn Fn(&mut Window, &mut App) -> AnyElement + 'static>,
}

impl HoverCard {
    pub fn new(trigger: impl IntoElement) -> Self {
        Self {
            trigger: trigger.into_any_element(),
            content_builder: Rc::new(|_, _| div().into_any_element()),
        }
    }

    pub fn content(
        mut self,
        content: impl Fn(&mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.content_builder = Rc::new(content);
        self
    }
}

impl RenderOnce for HoverCard {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let content_builder = self.content_builder.clone();
        div()
            .id("hover-card")
            .child(self.trigger)
            .hoverable_tooltip(move |_, cx| {
                cx.new(|_| HoverCardTooltip {
                    content_builder: content_builder.clone(),
                })
                .into()
            })
    }
}

impl Component for HoverCard {
    fn scope() -> ComponentScope {
        ComponentScope::Overlays
    }

    fn description() -> Option<&'static str> {
        Some("Rich hover content with delay, composing tooltip timing and Popover styling.")
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        Some(
            HoverCard::new(
                Button::new("hover-card-trigger", "@nextjs").style(ButtonStyle::Transparent),
            )
            .content(|_, _| {
                v_flex()
                    .gap_2()
                    .child(Label::new("Next.js").size(LabelSize::Small))
                    .child(
                        Label::new("The React Framework for the Web — built by Vercel.")
                            .size(LabelSize::XSmall)
                            .color(Color::Muted),
                    )
                    .child(
                        h_flex()
                            .gap_3()
                            .child(Label::new("Joined December 2021").size(LabelSize::XSmall))
                            .child(Label::new("1.2k followers").size(LabelSize::XSmall)),
                    )
                    .into_any_element()
            })
            .into_any_element(),
        )
    }
}
