use gpui::ElementId;
use std::rc::Rc;

use crate::prelude::*;

/// Selection mode for a [`ToggleGroup`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ToggleGroupMode {
    /// Exactly one item may be selected at a time.
    #[default]
    Single,
    /// Multiple items may be selected simultaneously.
    Multiple,
}

/// Configuration for one segment in a [`ToggleGroup`].
#[derive(Clone)]
pub struct ToggleGroupItem {
    label: SharedString,
    icon: Option<IconName>,
}

impl ToggleGroupItem {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            icon: None,
        }
    }

    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }
}

/// A connected row of toggle buttons supporting single or multiple selection.
///
/// Selection state is caller-managed: pass the active indices via
/// [`.selected()`](ToggleGroup::selected) and update them in
/// [`.on_change()`](ToggleGroup::on_change).
#[derive(IntoElement, RegisterComponent)]
pub struct ToggleGroup {
    id: ElementId,
    items: Vec<ToggleGroupItem>,
    mode: ToggleGroupMode,
    selected: Vec<usize>,
    disabled: bool,
    on_change: Option<Rc<dyn Fn(Vec<usize>, &mut Window, &mut App) + 'static>>,
}

impl ToggleGroup {
    pub fn new(id: impl Into<ElementId>, items: impl IntoIterator<Item = ToggleGroupItem>) -> Self {
        Self {
            id: id.into(),
            items: items.into_iter().collect(),
            mode: ToggleGroupMode::default(),
            selected: Vec::new(),
            disabled: false,
            on_change: None,
        }
    }

    pub fn mode(mut self, mode: ToggleGroupMode) -> Self {
        self.mode = mode;
        self
    }

    /// Sets the indices of currently selected items.
    pub fn selected(mut self, indices: impl IntoIterator<Item = usize>) -> Self {
        self.selected = indices.into_iter().collect();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Binds a handler called with the group's *next* selection after a
    /// click, computed according to [`ToggleGroupMode`]: in `Single` mode
    /// the clicked index replaces the selection (`[index]`); in `Multiple`
    /// mode the clicked index is toggled within the current selection. The
    /// group is controlled — the caller must feed the reported selection
    /// back in via [`.selected()`](ToggleGroup::selected) for it to render.
    pub fn on_change(
        mut self,
        handler: impl Fn(Vec<usize>, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for ToggleGroup {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let selected = self.selected;
        let disabled = self.disabled;
        let mode = self.mode;
        let group_id = self.id.clone();
        let on_change = self.on_change;

        let mut row = h_flex()
            .id(self.id)
            .gap_1()
            .p(px(2.))
            .rounded_md()
            .bg(semantic::muted_bg(cx));

        for (i, item) in self.items.into_iter().enumerate() {
            let is_selected = selected.contains(&i);
            let on_change = on_change.clone();
            let current_selected = selected.clone();

            let mut cell = h_flex()
                .id((group_id.clone(), i.to_string()))
                .flex_1()
                .justify_center()
                .items_center()
                .gap_1()
                .px_3()
                .py_1p5()
                .rounded_sm()
                .when(is_selected, |this| {
                    this.bg(semantic::accent_bg(cx))
                        .text_color(semantic::accent_fg(cx))
                })
                .when(!is_selected && !disabled, |this| {
                    this.hover(|style| style.bg(semantic::hover_bg(cx)))
                })
                .when(disabled, |this| this.opacity(0.5))
                .children(item.icon.map(|icon| {
                    Icon::new(icon)
                        .size(IconSize::Small)
                        .color(if is_selected {
                            Color::Accent
                        } else {
                            Color::Muted
                        })
                        .into_any_element()
                }))
                .child(
                    Label::new(item.label)
                        .size(LabelSize::Small)
                        .color(if is_selected {
                            Color::Accent
                        } else {
                            Color::Default
                        }),
                );

            if !disabled {
                cell = cell.cursor_pointer().on_click(move |_, window, cx| {
                    if let Some(handler) = &on_change {
                        let next = match mode {
                            ToggleGroupMode::Single => vec![i],
                            ToggleGroupMode::Multiple => {
                                let mut next = current_selected.clone();
                                if let Some(pos) = next.iter().position(|&x| x == i) {
                                    next.remove(pos);
                                } else {
                                    next.push(i);
                                }
                                next
                            }
                        };
                        handler(next, window, cx);
                    }
                });
            }

            row = row.child(cell);
        }

        row
    }
}

impl Component for ToggleGroup {
    fn scope() -> ComponentScope {
        ComponentScope::Input
    }

    fn description() -> Option<&'static str> {
        Some("A row of toggle buttons supporting single or multiple selection.")
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        Some(
            v_flex()
                .gap_4()
                .child(
                    ToggleGroup::new(
                        "toggle-group-single",
                        [
                            ToggleGroupItem::new("List"),
                            ToggleGroupItem::new("Grid"),
                            ToggleGroupItem::new("Board"),
                        ],
                    )
                    .selected([1])
                    .into_any_element(),
                )
                .child(
                    ToggleGroup::new(
                        "toggle-group-multiple",
                        [
                            ToggleGroupItem::new("Bold"),
                            ToggleGroupItem::new("Italic"),
                            ToggleGroupItem::new("Underline"),
                        ],
                    )
                    .mode(ToggleGroupMode::Multiple)
                    .selected([0, 2])
                    .into_any_element(),
                )
                .child(
                    ToggleGroup::new(
                        "toggle-group-icons",
                        [
                            ToggleGroupItem::new("Left").icon(IconName::ArrowLeft),
                            ToggleGroupItem::new("Center").icon(IconName::Dash),
                            ToggleGroupItem::new("Right").icon(IconName::ArrowRight),
                        ],
                    )
                    .selected([0])
                    .disabled(true)
                    .into_any_element(),
                )
                .into_any_element(),
        )
    }
}
