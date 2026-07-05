//! Session-level "advertised model" bookkeeping.
//!
//! Ports `others/acpx/src/session/model-state.ts`. Note this is a different
//! concept from [`crate::agent_command::model_support::SessionModelState`]
//! (which this module wraps): that type is the *ephemeral* model state an
//! agent just advertised over the wire; this module folds it into the
//! *persisted* [`SessionAcpxState`].

use agent_client_protocol::schema::v1::{SessionConfigOption, SessionConfigOptionCategory};

use crate::agent_command::model_support::{SessionModelState, model_state_from_config_options};
use crate::session::acpx_state::{ModelControl, SessionAcpxState};

fn config_options_are_authoritative(state: &SessionAcpxState) -> bool {
    state.model_control == Some(ModelControl::ConfigOption)
}

fn legacy_model_state(state: &SessionAcpxState) -> Option<SessionModelState> {
    let available_models = state.available_models.as_ref()?;
    Some(SessionModelState {
        config_id: None,
        current_model_id: state.current_model_id.clone().unwrap_or_default(),
        available_models: available_models
            .iter()
            .map(
                |model_id| crate::agent_command::model_support::AvailableModel {
                    model_id: model_id.clone(),
                    name: model_id.clone(),
                },
            )
            .collect(),
    })
}

/// Ports `advertisedModelState`.
pub fn advertised_model_state(state: Option<&SessionAcpxState>) -> Option<SessionModelState> {
    let state = state?;
    let config_options = state.config_options.as_deref().unwrap_or(&[]);
    if let Some(models) = model_state_from_config_options(config_options) {
        return Some(models);
    }
    if config_options_are_authoritative(state) {
        return None;
    }
    legacy_model_state(state)
}

/// Ports `applyAdvertisedModelState`.
pub fn apply_advertised_model_state(state: &mut SessionAcpxState, models: &SessionModelState) {
    state.current_model_id = Some(models.current_model_id.clone());
    state.available_models = Some(
        models
            .available_models
            .iter()
            .map(|m| m.model_id.clone())
            .collect(),
    );
    state.model_control = Some(if models.config_id.is_some() {
        ModelControl::ConfigOption
    } else {
        ModelControl::LegacySetModel
    });
}

/// Ports `clearAdvertisedModelState`.
pub fn clear_advertised_model_state(state: &mut SessionAcpxState) {
    state.current_model_id = None;
    state.available_models = None;
    state.model_control = None;
}

fn is_model_config_option(option: &SessionConfigOption) -> bool {
    option.category == Some(SessionConfigOptionCategory::Model) || option.id.0.as_ref() == "model"
}

/// Ports `removeModelConfigOptions`.
pub fn remove_model_config_options(state: &mut SessionAcpxState) {
    if let Some(config_options) = &mut state.config_options {
        config_options.retain(|option| !is_model_config_option(option));
    }
}

/// Ports `applyConfigOptionsModelState`.
pub fn apply_config_options_model_state(
    state: &mut SessionAcpxState,
    config_options: Vec<SessionConfigOption>,
) {
    let previous_config_models =
        model_state_from_config_options(state.config_options.as_deref().unwrap_or(&[]));
    let preserves_legacy_control = state.model_control == Some(ModelControl::LegacySetModel)
        || (state.model_control.is_none()
            && previous_config_models.is_none()
            && legacy_model_state(state).is_some());

    let models = model_state_from_config_options(&config_options);
    state.config_options = Some(config_options);

    if let Some(models) = &models {
        apply_advertised_model_state(state, models);
    } else if preserves_legacy_control {
        state.model_control = Some(ModelControl::LegacySetModel);
    } else {
        clear_advertised_model_state(state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use agent_client_protocol::schema::v1::{
        SessionConfigId, SessionConfigKind, SessionConfigSelect, SessionConfigSelectOption,
    };

    fn model_option(current: &'static str) -> SessionConfigOption {
        SessionConfigOption::new(
            SessionConfigId::new("model"),
            "Model",
            SessionConfigKind::Select(SessionConfigSelect::new(
                current,
                vec![SessionConfigSelectOption::new(
                    "default-model",
                    "default-model",
                )],
            )),
        )
    }

    #[test]
    fn applying_config_options_sets_advertised_model_state() {
        let mut state = SessionAcpxState::default();
        apply_config_options_model_state(&mut state, vec![model_option("default-model")]);
        assert_eq!(state.current_model_id.as_deref(), Some("default-model"));
        assert_eq!(state.model_control, Some(ModelControl::ConfigOption));
    }

    #[test]
    fn legacy_state_used_when_no_config_option_models_advertised() {
        let mut state = SessionAcpxState {
            current_model_id: Some("legacy-model".into()),
            available_models: Some(vec!["legacy-model".into()]),
            ..Default::default()
        };
        let models = advertised_model_state(Some(&state)).unwrap();
        assert_eq!(models.current_model_id, "legacy-model");

        clear_advertised_model_state(&mut state);
        assert!(state.current_model_id.is_none());
    }
}
