use gpui::{AnyElement, FontWeight, IntoElement, ParentElement, Styled};
use std::rc::Rc;

use crate::prelude::*;

/// A bordered, `semantic::elevated_surface`-backed section with a fieldset
/// area (arbitrary child fields, e.g. `FormField`s) followed by a `border-t`
/// footer with right-aligned Save/Cancel `Button`s.
#[derive(IntoElement, RegisterComponent)]
pub struct ActionPanel {
    title: Option<SharedString>,
    description: Option<SharedString>,
    fields: Vec<AnyElement>,
    save_label: SharedString,
    cancel_label: SharedString,
    on_save: Option<Rc<dyn Fn(&mut Window, &mut App) + 'static>>,
    on_cancel: Option<Rc<dyn Fn(&mut Window, &mut App) + 'static>>,
}

impl ActionPanel {
    pub fn new() -> Self {
        Self {
            title: None,
            description: None,
            fields: Vec::new(),
            save_label: "Save".into(),
            cancel_label: "Cancel".into(),
            on_save: None,
            on_cancel: None,
        }
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Adds a field element to the fieldset area (e.g. a `FormField`).
    pub fn field(mut self, field: impl IntoElement) -> Self {
        self.fields.push(field.into_any_element());
        self
    }

    pub fn save_label(mut self, label: impl Into<SharedString>) -> Self {
        self.save_label = label.into();
        self
    }

    pub fn cancel_label(mut self, label: impl Into<SharedString>) -> Self {
        self.cancel_label = label.into();
        self
    }

    pub fn on_save(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_save = Some(Rc::new(handler));
        self
    }

    pub fn on_cancel(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_cancel = Some(Rc::new(handler));
        self
    }
}

impl Default for ActionPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for ActionPanel {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let title = self.title;
        let description = self.description;
        let fields = self.fields;
        let save_label = self.save_label;
        let cancel_label = self.cancel_label;
        let on_save = self.on_save;
        let on_cancel = self.on_cancel;

        let mut fieldset = v_flex().gap_4().p_4();
        for field in fields {
            fieldset = fieldset.child(field);
        }

        v_flex()
            .w_full()
            .rounded_lg()
            .border_1()
            .border_color(semantic::border(cx))
            .bg(semantic::elevated_surface(cx))
            .overflow_hidden()
            .when(title.is_some() || description.is_some(), |this| {
                this.child(
                    v_flex()
                        .gap_1()
                        .px_4()
                        .pt_4()
                        .when_some(title, |this, title| {
                            this.child(
                                Label::new(title)
                                    .size(LabelSize::Default)
                                    .weight(FontWeight::MEDIUM),
                            )
                        })
                        .when_some(description, |this, description| {
                            this.child(
                                Label::new(description)
                                    .size(LabelSize::Small)
                                    .color(Color::Muted),
                            )
                        }),
                )
            })
            .child(fieldset)
            .child(
                h_flex()
                    .justify_end()
                    .gap_2()
                    .px_4()
                    .py_3()
                    .border_t_1()
                    .border_color(semantic::border(cx))
                    .when_some(on_cancel, |this, handler| {
                        this.child(
                            Button::new("action-panel-cancel", cancel_label)
                                .on_click(move |_, window, cx| handler(window, cx)),
                        )
                    })
                    .when_some(on_save, |this, handler| {
                        this.child(
                            Button::new("action-panel-save", save_label)
                                .primary()
                                .on_click(move |_, window, cx| handler(window, cx)),
                        )
                    }),
            )
    }
}

impl Component for ActionPanel {
    fn scope() -> ComponentScope {
        ComponentScope::Input
    }

    fn description() -> Option<&'static str> {
        Some("A fieldset section with a footer of right-aligned Save/Cancel buttons.")
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        Some(
            ActionPanel::new()
                .title("Profile")
                .description("Update your personal details.")
                .field(
                    FormField::new(Label::new("Ada Lovelace").color(Color::Default)).label("Name"),
                )
                .field(
                    FormField::new(Label::new("ada@example.com").color(Color::Default))
                        .label("Email"),
                )
                .on_save(|_, _| {})
                .on_cancel(|_, _| {})
                .into_any_element(),
        )
    }
}
