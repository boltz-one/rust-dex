use gpui::AnyElement;
use smallvec::SmallVec;

use crate::prelude::*;

/// Visual style of a [`Card`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CardVariant {
    /// Surface with a 1px border and no shadow.
    #[default]
    Bordered,
    /// Surface with a medium drop shadow and no border.
    Elevated,
    /// Plain surface, no border or shadow.
    Flat,
}

/// A surface container for grouping related content, with optional header and
/// footer regions. Neutrals are theme-driven (dark/light aware).
#[derive(IntoElement, RegisterComponent)]
pub struct Card {
    variant: CardVariant,
    header: Option<AnyElement>,
    footer: Option<AnyElement>,
    children: SmallVec<[AnyElement; 2]>,
}

impl Card {
    pub fn new() -> Self {
        Self {
            variant: CardVariant::default(),
            header: None,
            footer: None,
            children: SmallVec::new(),
        }
    }

    pub fn variant(mut self, variant: CardVariant) -> Self {
        self.variant = variant;
        self
    }

    pub fn header(mut self, header: impl IntoElement) -> Self {
        self.header = Some(header.into_any_element());
        self
    }

    pub fn footer(mut self, footer: impl IntoElement) -> Self {
        self.footer = Some(footer.into_any_element());
        self
    }
}

impl Default for Card {
    fn default() -> Self {
        Self::new()
    }
}

impl ParentElement for Card {
    fn extend(&mut self, elements: impl IntoIterator<Item = AnyElement>) {
        self.children.extend(elements);
    }
}

impl RenderOnce for Card {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let mut base = v_flex()
            .bg(semantic::surface(cx))
            .rounded_lg()
            .text_color(semantic::text(cx));

        base = match self.variant {
            CardVariant::Bordered => base.border_1().border_color(semantic::border(cx)),
            CardVariant::Elevated => base.shadow_level(Shadow::Md),
            CardVariant::Flat => base,
        };

        base.when_some(self.header, |this, header| {
            this.child(
                div()
                    .px_6()
                    .py_4()
                    .border_b_1()
                    .border_color(semantic::border(cx))
                    .child(header),
            )
        })
        .child(v_flex().p_6().gap_4().children(self.children))
        .when_some(self.footer, |this, footer| {
            this.child(
                div()
                    .px_6()
                    .py_4()
                    .border_t_1()
                    .border_color(semantic::border(cx))
                    .child(footer),
            )
        })
    }
}

impl Component for Card {
    fn scope() -> ComponentScope {
        ComponentScope::Layout
    }

    fn description() -> Option<&'static str> {
        Some("A surface container for grouping content, with optional header and footer.")
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        Some(
            h_flex()
                .gap_4()
                .child(
                    Card::new()
                        .header(Label::new("Bordered"))
                        .child(Label::new("Body content"))
                        .footer(Label::new("Footer")),
                )
                .child(
                    Card::new()
                        .variant(CardVariant::Elevated)
                        .header(Label::new("Elevated"))
                        .child(Label::new("With shadow")),
                )
                .child(
                    Card::new()
                        .variant(CardVariant::Flat)
                        .child(Label::new("Flat card")),
                )
                .into_any_element(),
        )
    }
}
