use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use gpui::{Entity, SharedString};

use crate::prelude::*;
use crate::score;
use crate::{ContextMenu, DropdownMenu, DropdownStyle, TextInput};

/// Model lists at or under this length render as a flat menu. Longer lists
/// gain a fuzzy-filter input above the entries so a specific model can be
/// typed to find rather than scrolled to.
const FUZZY_FILTER_THRESHOLD: usize = 8;

type OnSelectModel = Arc<dyn Fn(SharedString, &mut Window, &mut App) + 'static>;

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
    on_select: Option<OnSelectModel>,
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

        let menu = if available_model_ids.len() > FUZZY_FILTER_THRESHOLD {
            build_filterable_menu(window, cx, current, available_model_ids, on_select)
        } else {
            build_flat_menu(window, cx, current, available_model_ids, on_select)
        };

        DropdownMenu::new(self.id, label, menu).style(DropdownStyle::Subtle)
    }
}

fn build_flat_menu(
    window: &mut Window,
    cx: &mut App,
    current: Option<SharedString>,
    available_model_ids: Vec<SharedString>,
    on_select: Option<OnSelectModel>,
) -> Entity<ContextMenu> {
    ContextMenu::build(window, cx, move |mut menu, _window, _cx| {
        for model_id in &available_model_ids {
            menu = push_model_entry(menu, model_id.clone(), current.as_ref(), on_select.clone());
        }
        menu
    })
}

/// Builds a menu with a live fuzzy-filter input above the model entries. The
/// filter text lives in a [`TextInput`] entity created once (on first build)
/// and reused across rebuilds, so typing doesn't lose focus or content; each
/// keystroke rebuilds the menu (`ContextMenu::build_persistent`) with the
/// list re-scored via [`score`].
fn build_filterable_menu(
    window: &mut Window,
    cx: &mut App,
    current: Option<SharedString>,
    available_model_ids: Vec<SharedString>,
    on_select: Option<OnSelectModel>,
) -> Entity<ContextMenu> {
    let query_input: Rc<RefCell<Option<Entity<TextInput>>>> = Rc::default();

    ContextMenu::build_persistent(window, cx, move |mut menu, window, cx| {
        let input = query_input
            .borrow_mut()
            .get_or_insert_with(|| {
                let input = cx.new(|cx| TextInput::new(cx).placeholder("Filter models…"));
                cx.observe_in(&input, window, |menu, _, window, cx| {
                    menu.rebuild(window, cx);
                })
                .detach();
                input
            })
            .clone();

        let query = input.read(cx).text().to_string();
        let mut matched: Vec<(SharedString, i32)> = available_model_ids
            .iter()
            .filter_map(|id| score(&query, id.as_ref()).map(|s| (id.clone(), s)))
            .collect();
        matched.sort_by(|a, b| b.1.cmp(&a.1));

        menu = menu.custom_row(move |_window, _cx| {
            div()
                .w_full()
                .px_1()
                .child(input.clone())
                .into_any_element()
        });

        if matched.is_empty() {
            menu = menu.header("No matching models");
        }

        for (model_id, _score) in matched {
            menu = push_model_entry(menu, model_id, current.as_ref(), on_select.clone());
        }

        menu.keep_open_on_confirm(false)
    })
}

fn push_model_entry(
    menu: ContextMenu,
    model_id: SharedString,
    current: Option<&SharedString>,
    on_select: Option<OnSelectModel>,
) -> ContextMenu {
    let is_current = current == Some(&model_id);
    let entry_id = model_id.clone();
    menu.toggleable_entry(
        model_id,
        is_current,
        IconPosition::End,
        None,
        move |window, cx| {
            if let Some(on_select) = &on_select {
                on_select(entry_id.clone(), window, cx);
            }
        },
    )
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
             available model ids; no data fetching happens here. Lists \
             longer than a handful of entries gain a fuzzy-filter input.",
        )
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        let models = vec![
            "claude-opus-4.6".to_string(),
            "claude-sonnet-4.6".to_string(),
            "gpt-5".to_string(),
        ];

        let many_models: Vec<String> = [
            "claude-opus-4.6",
            "claude-sonnet-4.6",
            "claude-haiku-4.6",
            "gpt-5",
            "gpt-5-mini",
            "gpt-5-nano",
            "gemini-2.5-pro",
            "gemini-2.5-flash",
            "llama-4-scout",
            "llama-4-maverick",
        ]
        .into_iter()
        .map(String::from)
        .collect();

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
                .child(single_example(
                    "Long list (fuzzy-filterable)",
                    AgentModelSelector::new("model-selector-3", Some("gpt-5"), &many_models)
                        .into_any_element(),
                ))
                .into_any_element(),
        )
    }
}
