use std::sync::Arc;

use gpui::SharedString;

use crate::prelude::*;
use crate::{ContextMenu, DropdownMenu, DropdownStyle};

/// The lifecycle an agent thread's session runs under. Plain, protocol-agnostic
/// data: the caller maps this to whatever session-mode concept its own
/// runtime uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionModeChoice {
    /// The session stays alive across multiple requests.
    Persistent,
    /// The session is torn down after a single request.
    Oneshot,
}

impl SessionModeChoice {
    const ALL: [SessionModeChoice; 2] = [SessionModeChoice::Persistent, SessionModeChoice::Oneshot];

    fn label(self) -> &'static str {
        match self {
            SessionModeChoice::Persistent => "Persistent",
            SessionModeChoice::Oneshot => "One-shot",
        }
    }
}

/// A dropdown for picking whether an agent thread's session persists across
/// requests or runs as a single one-shot exchange.
///
/// Pure builder: the caller supplies the current choice and is notified via
/// `on_select` when the user picks a different one. No data fetching happens
/// here.
#[derive(IntoElement, RegisterComponent)]
pub struct ModeSelector {
    id: ElementId,
    current: Option<SessionModeChoice>,
    on_select: Option<Arc<dyn Fn(SessionModeChoice, &mut Window, &mut App) + 'static>>,
}

impl ModeSelector {
    pub fn new(id: impl Into<ElementId>, current: Option<SessionModeChoice>) -> Self {
        Self {
            id: id.into(),
            current,
            on_select: None,
        }
    }

    pub fn on_select(
        mut self,
        handler: impl Fn(SessionModeChoice, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Arc::new(handler));
        self
    }
}

impl RenderOnce for ModeSelector {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let label: SharedString = self
            .current
            .map(SessionModeChoice::label)
            .unwrap_or("Select mode")
            .into();
        let current = self.current;
        let on_select = self.on_select.clone();

        let menu = ContextMenu::build(window, cx, move |mut menu, _window, _cx| {
            for mode in SessionModeChoice::ALL {
                let is_current = current == Some(mode);
                let on_select = on_select.clone();
                menu = menu.toggleable_entry(
                    mode.label(),
                    is_current,
                    IconPosition::End,
                    None,
                    move |window, cx| {
                        if let Some(on_select) = &on_select {
                            on_select(mode, window, cx);
                        }
                    },
                );
            }
            menu
        });

        DropdownMenu::new(self.id, label, menu).style(DropdownStyle::Subtle)
    }
}

impl Component for ModeSelector {
    fn scope() -> ComponentScope {
        ComponentScope::Agent
    }

    fn name() -> &'static str {
        "ModeSelector"
    }

    fn description() -> Option<&'static str> {
        Some(
            "A dropdown for picking whether an agent thread's session \
             persists across requests or runs as a single one-shot exchange.",
        )
    }

    fn preview(_window: &mut Window, _cx: &mut App) -> Option<AnyElement> {
        Some(
            v_flex()
                .gap_4()
                .child(single_example(
                    "Persistent selected",
                    ModeSelector::new("mode-selector-1", Some(SessionModeChoice::Persistent))
                        .into_any_element(),
                ))
                .child(single_example(
                    "One-shot selected",
                    ModeSelector::new("mode-selector-2", Some(SessionModeChoice::Oneshot))
                        .into_any_element(),
                ))
                .child(single_example(
                    "No selection",
                    ModeSelector::new("mode-selector-3", None).into_any_element(),
                ))
                .into_any_element(),
        )
    }
}
