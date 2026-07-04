use std::time::Duration;

use crate::AnnouncementToast;
use crate::prelude::*;
use gpui::{AnyElement, Context, Render};
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

/// Default auto-dismiss duration for [`SonnerStack`] toasts.
pub const SONNER_DEFAULT_DISMISS: Duration = Duration::from_secs(4);

/// Sonner-style toast variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SonnerToastVariant {
    #[default]
    Default,
    Success,
    Error,
    Info,
    Loading,
}

impl SonnerToastVariant {
    fn severity(self) -> Option<Severity> {
        match self {
            SonnerToastVariant::Default | SonnerToastVariant::Loading => None,
            SonnerToastVariant::Success => Some(Severity::Success),
            SonnerToastVariant::Error => Some(Severity::Error),
            SonnerToastVariant::Info => Some(Severity::Info),
        }
    }

    fn icon(self) -> IconName {
        match self {
            SonnerToastVariant::Default => IconName::Bell,
            SonnerToastVariant::Success => IconName::CheckCircle,
            SonnerToastVariant::Error => IconName::XCircle,
            SonnerToastVariant::Info => IconName::Info,
            SonnerToastVariant::Loading => IconName::CountdownTimer,
        }
    }
}

/// A compact Sonner-style toast notification.
#[derive(IntoElement)]
pub struct SonnerToast {
    message: SharedString,
    variant: SonnerToastVariant,
}

impl SonnerToast {
    pub fn new(message: impl Into<SharedString>) -> Self {
        Self {
            message: message.into(),
            variant: SonnerToastVariant::Default,
        }
    }

    pub fn variant(mut self, variant: SonnerToastVariant) -> Self {
        self.variant = variant;
        self
    }
}

impl RenderOnce for SonnerToast {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let icon_color = match self.variant.severity() {
            Some(Severity::Success) => palette::success(600),
            Some(Severity::Error) => palette::danger(600),
            Some(Severity::Info) => palette::primary(600),
            Some(Severity::Warning) => palette::warning(600),
            None => palette::neutral(600),
        };

        h_flex()
            .w(px(320.))
            .items_center()
            .gap_3()
            .px_4()
            .py_3()
            .rounded_lg()
            .bg(semantic::elevated_surface(cx))
            .border_1()
            .border_color(semantic::border(cx))
            .shadow_level(Shadow::Lg)
            .child(
                Icon::new(self.variant.icon())
                    .size(IconSize::Small)
                    .color(Color::Custom(icon_color)),
            )
            .child(Label::new(self.message))
    }
}

#[derive(Clone)]
struct SonnerToastEntry {
    id: u64,
    message: SharedString,
    variant: SonnerToastVariant,
}

/// Sonner-style toast queue with auto-dismiss timers (newest appended last).
///
/// Create with `cx.new(|cx| SonnerStack::new(cx))`.
pub struct SonnerStack {
    toasts: Vec<SonnerToastEntry>,
    next_id: u64,
    max_visible: usize,
}

impl SonnerStack {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let mut stack = Self {
            toasts: Vec::new(),
            next_id: 1,
            max_visible: TOAST_STACK_MAX_VISIBLE,
        };
        stack.push_internal(
            cx,
            "Welcome back!".into(),
            SonnerToastVariant::Info,
            SONNER_DEFAULT_DISMISS,
        );
        stack
    }

    pub fn max_visible(mut self, max: usize) -> Self {
        self.max_visible = max.max(1);
        self
    }

    /// Enqueue a toast and schedule auto-dismiss.
    pub fn push(
        &mut self,
        cx: &mut Context<Self>,
        message: impl Into<SharedString>,
        variant: SonnerToastVariant,
    ) {
        self.push_internal(cx, message.into(), variant, SONNER_DEFAULT_DISMISS);
    }

    pub fn push_with_duration(
        &mut self,
        cx: &mut Context<Self>,
        message: impl Into<SharedString>,
        variant: SonnerToastVariant,
        dismiss_after: Duration,
    ) {
        self.push_internal(cx, message.into(), variant, dismiss_after);
    }

    fn push_internal(
        &mut self,
        cx: &mut Context<Self>,
        message: SharedString,
        variant: SonnerToastVariant,
        dismiss_after: Duration,
    ) {
        let id = self.next_id;
        self.next_id += 1;
        self.toasts.push(SonnerToastEntry {
            id,
            message,
            variant,
        });
        cx.notify();

        cx.spawn(async move |this, cx| {
            cx.background_executor().timer(dismiss_after).await;
            this.update(cx, |stack, cx| {
                stack.dismiss(id, cx);
            })
            .ok();
        })
        .detach();
    }

    pub fn dismiss(&mut self, id: u64, cx: &mut Context<Self>) {
        self.toasts.retain(|entry| entry.id != id);
        cx.notify();
    }
}

impl Render for SonnerStack {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let visible: Vec<_> = self
            .toasts
            .iter()
            .rev()
            .take(self.max_visible)
            .cloned()
            .collect();

        div().absolute().bottom_0().right_0().p_4().child(
            v_flex()
                .id("sonner-stack")
                .gap(DynamicSpacing::Base12.rems(cx))
                .items_end()
                .children(
                    visible
                        .into_iter()
                        .map(|entry| SonnerToast::new(entry.message).variant(entry.variant)),
                ),
        )
    }
}

/// Gallery catalog entry for [`SonnerStack`].
#[derive(IntoElement, RegisterComponent)]
pub struct SonnerStackPreview;

impl RenderOnce for SonnerStackPreview {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        div()
            .relative()
            .w(px(480.))
            .h(px(320.))
            .overflow_hidden()
            .child(cx.new(SonnerStack::new))
    }
}

impl Component for SonnerStackPreview {
    fn scope() -> ComponentScope {
        ComponentScope::Notification
    }

    fn description() -> Option<&'static str> {
        Some("Sonner-style toast queue with auto-dismiss timers; newest on top.")
    }

    fn preview(window: &mut Window, cx: &mut App) -> Option<AnyElement> {
        SonnerStackPreview
            .render(window, cx)
            .into_any_element()
            .into()
    }
}
