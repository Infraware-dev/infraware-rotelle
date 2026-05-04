use std::sync::{Arc, Mutex};
use crate::scenario::{ActivationParams, Scenario};
use crate::scenario::registry::ScenarioRegistry;

/// Shared application state, cheaply cloneable via inner `Arc`s.
#[derive(Clone)]
pub struct AppState {
    pub active_scenario: Arc<Mutex<Arc<dyn Scenario>>>,
    pub state_file: String,
    pub registry: Arc<ScenarioRegistry>,
}

impl AppState {
    /// Deactivate the current scenario, activate `new`, and persist the change.
    pub fn switch_scenario(&self, new: Arc<dyn Scenario>, params: ActivationParams) -> &'static str {
        let name;
        {
            let mut active = self.active_scenario.lock().unwrap();
            active.deactivate();
            new.activate(&params);
            name = new.name();
            *active = new;
        }
        persist_state(&self.state_file, name, &params);
        name
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
struct PersistedState {
    #[serde(default)]
    failure_case: String,
    #[serde(default)]
    params: ActivationParams,
}

pub fn load_state(path: &str) -> (String, ActivationParams) {
    let persisted: PersistedState = std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    let case = if persisted.failure_case.is_empty() {
        "none, idle".to_string()
    } else {
        tracing::info!(path, failure_case = %persisted.failure_case, "loaded persisted state");
        persisted.failure_case
    };

    (case, persisted.params)
}

pub fn persist_state(path: &str, case: &str, params: &ActivationParams) {
    let persisted = PersistedState {
        failure_case: case.to_string(),
        params: params.clone(),
    };
    if let Ok(json) = serde_json::to_string(&persisted) {
        let _ = std::fs::write(path, json);
    }
}
