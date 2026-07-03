use crate::{ModalFooter, ModalHeader, prelude::*};
use gpui::{AnyElement, ScrollHandle};
use smallvec::SmallVec;

/// Default width for a [`Drawer`] (Tailwind `w-96` ~= 384px).
pub const DRAWER_WIDTH: Pixels = px(384.);

/// A side panel ("slide-over") anchored to the right edge of its container.
///
/// Structurally a sibling of [`crate::Modal`] (header/body/footer), but
/// positioned `absolute` + `right-0` and spanning the full height of its
/// containing element instead of being centered. The caller is responsible
/// for placing [`Drawer`] inside a `relative()` + `size_full()` overlay layer
/// (this component does not render its own backdrop/scrim, matching
/// [`crate::Modal`]'s existing convention).
#[derive(IntoElement, RegisterComponent)]
pub struct Drawer {
    id: ElementId,
    body_id: ElementId,
    header: ModalHeader,
    children: SmallVec<[AnyElement; 2]>,
    footer: Option<ModalFooter>,
    width: Pixels,
    container_scroll_handle: Option<ScrollHandle>,
    animate: bool,
}

impl Drawer {
    pub fn new(id: impl Into<SharedString>) -> Self {
        let id = id.into();
        let body_id = ElementId::Name(format!("{}_body", id).into());

        Self {
            id: ElementId::Name(id),
            body_id,
            header: ModalHeader::new(),
            children: SmallVec::new(),
            footer: None,
            width: DRAWER_WIDTH,
            container_scroll_handle: None,
            animate: true,
        }
    }

    pub fn header(mut self, header: ModalHeader) -> Self {
        self.header = header;
        self
    }

    pub fn footer(mut self, footer: ModalFooter) -> Self {
        self.footer = Some(footer);
        self
    }

    /// Overrides the default `w-96` (384px) panel width.
    pub fn width(mut self, width: Pixels) -> Self {
        self.width = width;
        self
    }

    pub fn show_dismiss(mut self, show: bool) -> Self {
        self.header = self.header.show_dismiss_button(show);
        self
    }

    pub fn scroll_handle(mut self, handle: ScrollHandle) -> Self {
        self.container_scroll_handle = Some(handle);
        self
    }

    /// Disables the slide-in animation, e.g. for static previews/snapshots.
    pub fn animate(mut self, animate: bool) -> Self {
        self.animate = animate;
        self
    }
}

impl ParentElement for Drawer {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements)
    }
}

impl RenderOnce for Drawer {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let panel = v_flex()
            .id(self.id.clone())
            .absolute()
            .top_0()
            .right_0()
            .h_full()
            .w(self.width)
            .bg(semantic::elevated_surface(cx))
            .border_l_1()
            .border_color(semantic::border(cx))
            .shadow_level(Shadow::Xl)
            .overflow_hidden()
            .child(self.header)
            .child(
                v_flex()
                    .id(self.body_id)
                    .flex_1()
                    .w_full()
                    .p(DynamicSpacing::Base24.rems(cx))
                    .gap(DynamicSpacing::Base08.rems(cx))
                    .when_some(self.container_scroll_handle, |this, handle| {
                        this.overflow_y_scroll().track_scroll(&handle)
                    })
                    .children(self.children),
            )
            .children(self.footer);

        if self.animate {
            panel.animate_in_from_right(false).into_any_element()
        } else {
            panel.into_any_element()
        }
    }
}

impl Component for Drawer {
    fn scope() -> ComponentScope {
        ComponentScope::Overlays
    }

    fn description() -> Option<&'static str> {
        Some(
            "A side panel (slide-over) anchored to the right edge, for detail views or forms that shouldn't fully replace the current context.",
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
                        Drawer::new("drawer-preview")
                            .animate(false)
                            .header(
                                ModalHeader::new()
                                    .headline("Drawer title")
                                    .show_dismiss_button(true),
                            )
                            .child(Label::new("Drawer body content goes here."))
                            .footer(
                                ModalFooter::new()
                                    .end_slot(Button::new("drawer-confirm", "Save").primary()),
                            ),
                    )
                    .into_any_element(),
            )])
            .into_any_element(),
        )
    }
}
