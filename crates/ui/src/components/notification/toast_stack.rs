use crate::AnnouncementToast;
use crate::prelude::*;
use gpui::AnyElement;
use smallvec::SmallVec;

/// Default cap on simultaneously visible toasts (Tailwind reference spec: max 3 visible).
pub const TOAST_STACK_MAX_VISIBLE: usize = 3;

/// A bottom-right stacking container for [`crate::AnnouncementToast`] instances.
///
/// The caller owns the toast list (and its lifetime/state) and passes already-built
/// `AnnouncementToast` elements as children on every render; [`ToastStack`] only
/// handles layout (vertical stack, `gap-3`, bottom-right corner) and caps how many
/// are shown at once via [`Self::max_visible`] — any elements beyond the cap are
/// simply not rendered (the caller's own list still holds them, e.g. for a "N more"
/// indicator).
///
/// Auto-dismiss timing is intentionally **caller-driven**: this component has no
/// timer of its own. Callers wanting a timed dismissal should schedule it themselves
/// (e.g. via `cx.spawn` + `cx.background_executor().timer(..)`, the same pattern used
/// elsewhere in this crate) and remove the toast from their owned list when it fires.
#[derive(IntoElement, RegisterComponent)]
pub struct ToastStack {
    children: SmallVec<[AnyElement; 4]>,
    max_visible: usize,
}

impl ToastStack {
    pub fn new() -> Self {
        Self {
            children: SmallVec::new(),
            max_visible: TOAST_STACK_MAX_VISIBLE,
        }
    }

    /// Caps how many toasts are rendered at once. Values below `1` are clamped to `1`.
    pub fn max_visible(mut self, max: usize) -> Self {
        self.max_visible = max.max(1);
        self
    }
}

impl Default for ToastStack {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for ToastStack {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements)
    }
}

impl RenderOnce for ToastStack {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let visible = self.children.into_iter().take(self.max_visible);

        div().absolute().bottom_0().right_0().p_4().child(
            v_flex()
                .id("toast-stack")
                .gap(DynamicSpacing::Base12.rems(cx))
                .items_end()
                .children(visible),
        )
    }
}

impl Component for ToastStack {
    fn scope() -> ComponentScope {
        ComponentScope::Notification
    }

    fn description() -> Option<&'static str> {
        Some(
            "Stacks AnnouncementToast instances bottom-right, capping visible count; auto-dismiss and list ownership stay with the caller.",
        )
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        Some(
            example_group(vec![single_example(
                "Basic",
                div()
                    .relative()
                    .w(px(480.))
                    .h(px(320.))
                    .overflow_hidden()
                    .child(
                        ToastStack::new()
                            .child(
                                div().w_80().child(
                                    AnnouncementToast::new()
                                        .severity(Severity::Success)
                                        .heading("Saved")
                                        .description("Your changes were saved."),
                                ),
                            )
                            .child(
                                div().w_80().child(
                                    AnnouncementToast::new()
                                        .severity(Severity::Info)
                                        .heading("Sync complete")
                                        .description("All projects are up to date."),
                                ),
                            ),
                    )
                    .into_any_element(),
            )])
            .into_any_element(),
        )
    }
}
