use std::sync::Arc;

use gpui::SharedString;

use crate::prelude::*;
use crate::{ContextMenu, DropdownMenu, DropdownStyle};

/// A single selectable profile/config option: a stable `key` (passed to
/// `on_select`) and a human-readable `label` shown in the menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileOption {
    pub key: SharedString,
    pub label: SharedString,
}

impl ProfileOption {
    pub fn new(key: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
        }
    }
}

impl From<(String, String)> for ProfileOption {
    fn from((key, label): (String, String)) -> Self {
        Self::new(key, label)
    }
}

impl From<String> for ProfileOption {
    fn from(key: String) -> Self {
        let label = key.clone();
        Self::new(key, label)
    }
}

impl From<&str> for ProfileOption {
    fn from(key: &str) -> Self {
        Self::new(key, key)
    }
}

/// A dropdown for picking an agent's active profile/config option out of a
/// caller-supplied list of keys (optionally paired with display labels).
///
/// Pure builder: the caller supplies the current key and the full option
/// list (no fetching happens here), and is notified via `on_select` with the
/// chosen option's `key` when the user picks a different one.
#[derive(IntoElement, RegisterComponent)]
pub struct ProfileSelector {
    id: ElementId,
    current_key: Option<SharedString>,
    options: Vec<ProfileOption>,
    on_select: Option<Arc<dyn Fn(SharedString, &mut Window, &mut App) + 'static>>,
}

impl ProfileSelector {
    pub fn new(
        id: impl Into<ElementId>,
        current_key: Option<&str>,
        options: impl IntoIterator<Item = impl Into<ProfileOption>>,
    ) -> Self {
        Self {
            id: id.into(),
            current_key: current_key.map(SharedString::from),
            options: options.into_iter().map(Into::into).collect(),
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

impl RenderOnce for ProfileSelector {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let current_label = self
            .current_key
            .as_ref()
            .and_then(|key| self.options.iter().find(|option| &option.key == key))
            .map(|option| option.label.clone());
        let label = current_label.unwrap_or_else(|| "Select profile".into());
        let current_key = self.current_key.clone();
        let on_select = self.on_select.clone();
        let options = self.options.clone();

        let menu = ContextMenu::build(window, cx, move |mut menu, _window, _cx| {
            for option in &options {
                let is_current = current_key.as_ref() == Some(&option.key);
                let key = option.key.clone();
                let on_select = on_select.clone();
                menu = menu.toggleable_entry(
                    option.label.clone(),
                    is_current,
                    IconPosition::End,
                    None,
                    move |window, cx| {
                        if let Some(on_select) = &on_select {
                            on_select(key.clone(), window, cx);
                        }
                    },
                );
            }
            menu
        });

        DropdownMenu::new(self.id, label, menu).style(DropdownStyle::Subtle)
    }
}

impl Component for ProfileSelector {
    fn scope() -> ComponentScope {
        ComponentScope::Agent
    }

    fn name() -> &'static str {
        "ProfileSelector"
    }

    fn description() -> Option<&'static str> {
        Some(
            "A dropdown for picking an agent's active profile/config option \
             out of a caller-supplied list of keys, optionally paired with \
             display labels. No data fetching happens here.",
        )
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        let plain_keys = vec!["default", "careful", "yolo"];
        let labeled_keys = vec![
            ("default".to_string(), "Default".to_string()),
            (
                "careful".to_string(),
                "Careful (read-only tools)".to_string(),
            ),
            ("yolo".to_string(), "Yolo (all tools)".to_string()),
        ];

        Some(
            v_flex()
                .gap_4()
                .child(single_example(
                    "Plain keys",
                    ProfileSelector::new("profile-selector-1", Some("default"), plain_keys)
                        .into_any_element(),
                ))
                .child(single_example(
                    "Key + label pairs",
                    ProfileSelector::new("profile-selector-2", Some("careful"), labeled_keys)
                        .into_any_element(),
                ))
                .child(single_example(
                    "No selection",
                    ProfileSelector::new(
                        "profile-selector-3",
                        None,
                        vec!["default", "careful", "yolo"],
                    )
                    .into_any_element(),
                ))
                .into_any_element(),
        )
    }
}
