use crate::core::registry::ScenarioRegistry;
use crate::core::{ActivationParams, Scenario};
use std::sync::{Arc, Mutex};

/// Shared application state, cheaply cloneable via inner `Arc`s.
#[derive(Clone)]
pub struct AppState {
    pub active_scenario: Arc<Mutex<Arc<dyn Scenario>>>,
    pub state_file: String,
    pub registry: Arc<ScenarioRegistry>,
}

impl AppState {
    pub fn switch_scenario(
        &self,
        new: Arc<dyn Scenario>,
        params: ActivationParams,
    ) -> &'static str {
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
    scenario: String,
    #[serde(default)]
    params: ActivationParams,
}

pub fn load_state(path: &str) -> (String, ActivationParams) {
    let persisted: PersistedState = std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    let scenario = if persisted.scenario.is_empty() {
        "none, idle".to_string()
    } else {
        tracing::info!(path, scenario = %persisted.scenario, "loaded persisted state");
        persisted.scenario
    };

    (scenario, persisted.params)
}

pub fn persist_state(path: &str, scenario: &str, params: &ActivationParams) {
    let persisted = PersistedState {
        scenario: scenario.to_string(),
        params: params.clone(),
    };
    if let Ok(json) = serde_json::to_string(&persisted) {
        let _ = std::fs::write(path, json); // best-effort: a failed write only means state won't survive a restart
    }
}
