//! Gallery catalog entry for [`PaneGroup`]. Kept separate from the
//! interactive `ui_gallery` Layout-page demo (which owns a persistent
//! `Entity<PaneGroup>` on `GalleryApp`, created once in `new()`) — this is a
//! throwaway static example for the `Component` catalog, mirroring
//! `ResizablePreview`'s equivalent split from `GalleryApp::resizable`.

use gpui::AnyElement;

use super::PaneGroup;
use crate::{Pane, SplitDirection, TabContent, prelude::*};

struct DemoTab(&'static str);

impl TabContent for DemoTab {
    fn render(&self, _focused: bool, _window: &mut Window, _cx: &mut App) -> AnyElement {
        div()
            .p_4()
            .child(Label::new(self.0).color(Color::Muted))
            .into_any_element()
    }

    fn title(&self) -> SharedString {
        self.0.into()
    }
}

/// Gallery catalog entry for [`PaneGroup`]: a two-pane horizontal split,
/// each with one demo tab.
#[derive(IntoElement, RegisterComponent)]
pub struct PaneGroupPreview;

impl RenderOnce for PaneGroupPreview {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        div().h(px(280.)).child(cx.new(|cx| {
            let first = cx.new(|cx| {
                let mut pane = Pane::new().with_new_tab_factory(|| Box::new(DemoTab("New tab")));
                pane.add_tab(Box::new(DemoTab("main.rs")), cx);
                pane
            });
            let mut group = PaneGroup::new(cx, first).with_pane_factory(|cx| {
                let mut pane = Pane::new().with_new_tab_factory(|| Box::new(DemoTab("New tab")));
                pane.add_tab(Box::new(DemoTab("README.md")), cx);
                pane
            });
            group.split(SplitDirection::Right, cx);
            group
        }))
    }
}

impl Component for PaneGroupPreview {
    fn scope() -> ComponentScope {
        ComponentScope::Layout
    }

    fn description() -> Option<&'static str> {
        Some(
            "Recursive split-tree pane group: N-way horizontal/vertical splits, \
             per-pane tabs, and both-axis drag-resize.",
        )
    }

    fn preview(window: &mut Window, cx: &mut App) -> Option<AnyElement> {
        PaneGroupPreview
            .render(window, cx)
            .into_any_element()
            .into()
    }
}
