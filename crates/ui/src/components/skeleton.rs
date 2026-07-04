use gpui::{Animation, AnimationExt, pulsating_between};
use std::time::Duration;

use crate::prelude::*;

/// A pulsing placeholder block shown while content is loading.
#[derive(IntoElement, RegisterComponent)]
pub struct Skeleton {
    id: ElementId,
    width: Option<DefiniteLength>,
    height: Option<DefiniteLength>,
    rounded: bool,
}

impl Skeleton {
    pub fn new() -> Self {
        Self {
            id: ElementId::Name("skeleton".into()),
            width: None,
            height: None,
            rounded: true,
        }
    }

    /// Overrides the element/animation id (defaults to `"skeleton"`). Set
    /// this to a unique value when rendering multiple `Skeleton`s as
    /// siblings so each gets its own animation key instead of sharing one.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn width(mut self, width: impl Into<DefiniteLength>) -> Self {
        self.width = Some(width.into());
        self
    }

    pub fn height(mut self, height: impl Into<DefiniteLength>) -> Self {
        self.height = Some(height.into());
        self
    }

    pub fn rounded(mut self, rounded: bool) -> Self {
        self.rounded = rounded;
        self
    }
}

impl Default for Skeleton {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for Skeleton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let muted = semantic::muted_bg(cx);

        let id = self.id;

        div()
            .id(id.clone())
            .map(|this| {
                if let Some(width) = self.width {
                    this.w(width)
                } else {
                    this.w_full()
                }
            })
            .h(self.height.unwrap_or_else(|| rems(1.).into()))
            .when(self.rounded, |this| this.rounded_md())
            .bg(muted)
            .with_animation(
                id,
                Animation::new(Duration::from_millis(1500))
                    .repeat()
                    .with_easing(pulsating_between(0.4, 1.0)),
                |el, delta| el.opacity(delta),
            )
    }
}

impl Component for Skeleton {
    fn scope() -> ComponentScope {
        ComponentScope::Loading
    }

    fn description() -> Option<&'static str> {
        Some("A pulsing placeholder block shown while content is loading.")
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        Some(
            v_flex()
                .gap_3()
                .w(px(320.))
                .child(Skeleton::new().height(rems(1.)))
                .child(Skeleton::new().width(relative(0.75)).height(rems(1.)))
                .child(Skeleton::new().width(relative(0.5)).height(rems(1.)))
                .into_any_element(),
        )
    }
}
