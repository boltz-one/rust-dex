//! Lightweight static table primitives (shadcn `Table` anatomy).
//!
//! For sortable/filterable tables use [`crate::Table`] in `data_table.rs`.

use gpui::AnyElement;
use smallvec::SmallVec;

use crate::prelude::*;

/// Root wrapper for a static HTML-style table layout.
///
/// For sortable/filterable data grids use [`crate::Table`] (`data_table.rs`).
#[derive(IntoElement, RegisterComponent)]
pub struct Table {
    children: SmallVec<[AnyElement; 3]>,
    caption: Option<AnyElement>,
}

impl Table {
    pub fn new() -> Self {
        Self {
            children: SmallVec::new(),
            caption: None,
        }
    }

    pub fn caption(mut self, caption: impl IntoElement) -> Self {
        self.caption = Some(caption.into_any_element());
        self
    }
}

impl Default for Table {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for Table {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Table {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        v_flex()
            .w_full()
            .when_some(self.caption, |this, caption| this.child(caption))
            .child(
                v_flex()
                    .w_full()
                    .overflow_hidden()
                    .rounded_md()
                    .border_1()
                    .border_color(semantic::border(cx))
                    .children(self.children),
            )
    }
}

/// Table header section (`<thead>`).
#[derive(IntoElement)]
pub struct TableHeader {
    children: SmallVec<[AnyElement; 1]>,
}

impl TableHeader {
    pub fn new() -> Self {
        Self {
            children: SmallVec::new(),
        }
    }
}

impl Default for TableHeader {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for TableHeader {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for TableHeader {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        v_flex()
            .w_full()
            .bg(semantic::elevated_surface(cx))
            .border_b_1()
            .border_color(semantic::border_muted(cx))
            .children(self.children)
    }
}

/// Table body section (`<tbody>`).
#[derive(IntoElement)]
pub struct TableBody {
    children: SmallVec<[AnyElement; 4]>,
}

impl TableBody {
    pub fn new() -> Self {
        Self {
            children: SmallVec::new(),
        }
    }
}

impl Default for TableBody {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for TableBody {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for TableBody {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        v_flex().w_full().children(self.children)
    }
}

/// Table footer section (`<tfoot>`).
#[derive(IntoElement)]
pub struct TableFooter {
    children: SmallVec<[AnyElement; 1]>,
}

impl TableFooter {
    pub fn new() -> Self {
        Self {
            children: SmallVec::new(),
        }
    }
}

impl Default for TableFooter {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for TableFooter {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for TableFooter {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        v_flex()
            .w_full()
            .bg(semantic::elevated_surface(cx))
            .border_t_1()
            .border_color(semantic::border_muted(cx))
            .children(self.children)
    }
}

/// A single table row.
#[derive(IntoElement)]
pub struct TableRow {
    children: SmallVec<[AnyElement; 4]>,
    hover: bool,
}

impl TableRow {
    pub fn new() -> Self {
        Self {
            children: SmallVec::new(),
            hover: true,
        }
    }

    pub fn hover(mut self, hover: bool) -> Self {
        self.hover = hover;
        self
    }
}

impl Default for TableRow {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for TableRow {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for TableRow {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let hover_bg = semantic::hover_bg(cx);
        h_flex()
            .w_full()
            .border_b_1()
            .border_color(semantic::border_muted(cx))
            .when(self.hover, |this| this.hover(move |s| s.bg(hover_bg)))
            .children(self.children)
    }
}

/// Header cell (`<th>`).
#[derive(IntoElement)]
pub struct TableHead {
    children: SmallVec<[AnyElement; 1]>,
    align_end: bool,
}

impl TableHead {
    pub fn new() -> Self {
        Self {
            children: SmallVec::new(),
            align_end: false,
        }
    }

    pub fn align_end(mut self, align_end: bool) -> Self {
        self.align_end = align_end;
        self
    }
}

impl Default for TableHead {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for TableHead {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for TableHead {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        h_flex()
            .flex_1()
            .px_4()
            .py_3()
            .when(self.align_end, |this| this.justify_end())
            .children(self.children)
    }
}

/// Body cell (`<td>`).
#[derive(IntoElement)]
pub struct TableCell {
    children: SmallVec<[AnyElement; 1]>,
    align_end: bool,
}

impl TableCell {
    pub fn new() -> Self {
        Self {
            children: SmallVec::new(),
            align_end: false,
        }
    }

    pub fn align_end(mut self, align_end: bool) -> Self {
        self.align_end = align_end;
        self
    }
}

impl Default for TableCell {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for TableCell {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for TableCell {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        h_flex()
            .flex_1()
            .px_4()
            .py_3()
            .when(self.align_end, |this| this.justify_end())
            .children(self.children)
    }
}

/// Optional table caption shown above the bordered table.
#[derive(IntoElement)]
pub struct TableCaption {
    label: SharedString,
}

impl TableCaption {
    pub fn new(label: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
        }
    }
}

impl RenderOnce for TableCaption {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        div().mb_2().child(
            Label::new(self.label)
                .size(LabelSize::Small)
                .color(Color::Muted),
        )
    }
}

impl Component for Table {
    fn scope() -> ComponentScope {
        ComponentScope::DataDisplay
    }

    fn description() -> Option<&'static str> {
        Some("Lightweight static table primitives (Root/Header/Body/Row/Head/Cell/Caption).")
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        Some(
            Table::new()
                .caption(TableCaption::new("A list of recent invoices."))
                .child(
                    TableHeader::new().child(
                        TableRow::new().hover(false).children([
                            TableHead::new()
                                .child(Label::new("Invoice").weight(gpui::FontWeight::SEMIBOLD)),
                            TableHead::new()
                                .align_end(true)
                                .child(Label::new("Amount").weight(gpui::FontWeight::SEMIBOLD)),
                        ]),
                    ),
                )
                .child(
                    TableBody::new()
                        .child(
                            TableRow::new().children([
                                TableCell::new().child(Label::new("INV001")),
                                TableCell::new()
                                    .align_end(true)
                                    .child(Label::new("$250.00")),
                            ]),
                        )
                        .child(
                            TableRow::new().children([
                                TableCell::new().child(Label::new("INV002")),
                                TableCell::new()
                                    .align_end(true)
                                    .child(Label::new("$150.00")),
                            ]),
                        ),
                )
                .into_any_element(),
        )
    }
}
