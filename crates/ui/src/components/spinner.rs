use crate::{CommonAnimationExt, prelude::*};

/// Size preset for a [`Spinner`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SpinnerSize {
    /// 12px
    Sm,
    /// 16px
    #[default]
    Default,
    /// 24px
    Lg,
}

impl SpinnerSize {
    fn icon_size(self) -> IconSize {
        match self {
            SpinnerSize::Sm => IconSize::XSmall,
            SpinnerSize::Default => IconSize::Medium,
            SpinnerSize::Lg => IconSize::Custom(rems_from_px(24.)),
        }
    }
}

/// A rotating loader icon for inline loading states.
#[derive(IntoElement, RegisterComponent)]
pub struct Spinner {
    id: ElementId,
    size: SpinnerSize,
}

impl Spinner {
    pub fn new() -> Self {
        Self {
            id: ElementId::Name("spinner".into()),
            size: SpinnerSize::default(),
        }
    }

    /// Overrides the element/animation id (defaults to `"spinner"`). Set
    /// this to a unique value when rendering multiple `Spinner`s as
    /// siblings so each gets its own animation key instead of sharing one.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    pub fn size(mut self, size: SpinnerSize) -> Self {
        self.size = size;
        self
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for Spinner {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        Icon::new(IconName::LoadCircle)
            .size(self.size.icon_size())
            .color(Color::Custom(semantic::text_muted(cx)))
            .with_keyed_rotate_animation(self.id, 1)
    }
}

impl Component for Spinner {
    fn scope() -> ComponentScope {
        ComponentScope::Loading
    }

    fn description() -> Option<&'static str> {
        Some("A rotating loader icon for inline loading states.")
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        Some(
            h_flex()
                .gap_4()
                .items_center()
                .child(Spinner::new().size(SpinnerSize::Sm))
                .child(Spinner::new())
                .child(Spinner::new().size(SpinnerSize::Lg))
                .into_any_element(),
        )
    }
}
