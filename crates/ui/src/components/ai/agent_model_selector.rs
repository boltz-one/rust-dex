use std::sync::Arc;

use gpui::SharedString;

use crate::prelude::*;
use crate::{ContextMenu, DropdownMenu, DropdownStyle};

/// A dropdown for picking the active model for an agent thread.
///
/// This is a pure builder: the caller supplies the current selection and the
/// full list of selectable model ids (no fetching happens here), and is
/// notified via `on_select` when the user picks a different one.
#[derive(IntoElement, RegisterComponent)]
pub struct AgentModelSelector {
    id: ElementId,
    current_model_id: Option<SharedString>,
    available_model_ids: Vec<SharedString>,
    on_select: Option<Arc<dyn Fn(SharedString, &mut Window, &mut App) + 'static>>,
}

impl AgentModelSelector {
    pub fn new(
        id: impl Into<ElementId>,
        current_model_id: Option<&str>,
        available_model_ids: &[String],
    ) -> Self {
        Self {
            id: id.into(),
            current_model_id: current_model_id.map(SharedString::from),
            available_model_ids: available_model_ids
                .iter()
                .map(|id| SharedString::from(id.clone()))
                .collect(),
            on_select: None,
        }
    }

    pub fn on_select(
        mut self,
        handler: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for AgentModelSelector {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let label = self
            .current_model_id
            .clone()
            .unwrap_or_else(|| "Select model".into());
        let current = self.current_model_id.clone();
        let on_select = self.on_select.clone();
        let available_model_ids = self.available_model_ids.clone();

        let menu = ContextMenu::build(window, cx, move |mut menu, _window, _cx| {
            for model_id in &available_model_ids {
                let is_current = current.as_ref() == Some(model_id);
                let model_id = model_id.clone();
                let on_select = on_select.clone();
                menu = menu.toggleable_entry(
                    model_id.clone(),
                    is_current,
                    IconPosition::End,
                    None,
                    move |window, cx| {
                        if let Some(on_select) = &on_select {
                            on_select(model_id.clone(), window, cx);
                        }
                    },
                );
            }
            menu
        });

        DropdownMenu::new(self.id, label, menu).style(DropdownStyle::Subtle)
    }
}

impl Component for AgentModelSelector {
    fn scope() -> ComponentScope {
        ComponentScope::Agent
    }

    fn name() -> &'static str {
        "AgentModelSelector"
    }

    fn description() -> Option<&'static str> {
        Some(
            "A dropdown for picking the active model for an agent thread. \
             The caller supplies the current model id and the full list of \
             available model ids; no data fetching happens here.",
        )
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        let models = vec![
            "claude-opus-4.6".to_string(),
            "claude-sonnet-4.6".to_string(),
            "gpt-5".to_string(),
        ];

        Some(
            v_flex()
                .gap_4()
                .child(single_example(
                    "With selection",
                    AgentModelSelector::new("model-selector-1", Some("claude-sonnet-4.6"), &models)
                        .into_any_element(),
                ))
                .child(single_example(
                    "No selection",
                    AgentModelSelector::new("model-selector-2", None, &models).into_any_element(),
                ))
                .into_any_element(),
        )
    }
}
