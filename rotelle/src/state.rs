use crate::core::registry::ScenarioRegistry;
use crate::core::{ActivationParams, Scenario};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use serde_json::Value;

/// Runtime mode — set via the `ROTELLE_MODE` environment variable.
/// - `full` (default): simulation surface + control plane in a single process on one port.
///   Use for local development and for the sim pod in K8s deployments. Exposes both
///   `GET /` (sim surface) and all `/rotectl/*` (control API), so the separate control
///   pod can probe it over HTTP.
/// - `control`: standalone control pod in an independent namespace. All `/rotectl/*`
///   handlers proxy to the sim pod via `ROTELLE_SIM_API_URL`. No shared volume required.
#[derive(Clone, PartialEq, Debug)]
pub enum Mode {
    Full,
    Control,
}

/// Shared application state, cheaply cloneable via inner `Arc`s.
#[derive(Clone)]
pub struct AppState {
    pub active_scenario: Arc<Mutex<Arc<dyn Scenario>>>,
    pub state_file: String,
    pub registry: Arc<ScenarioRegistry>,
    pub mode: Mode,
    /// Set when the active scenario triggers a simulated crash (`IndexEffect::Exit`).
    /// In `full` / `sim` mode: causes `GET /health` to return 503 so Kubernetes restarts
    /// the pod while the control plane stays accessible.
    /// In `control` mode: not used (crash state is probed from the sim pod via HTTP).
    pub crashed: Arc<AtomicBool>,
    /// Params that were last applied to the active scenario. Kept in sync by
    /// `apply_scenario_inner` so the control panel can display current values.
    pub active_params: Arc<Mutex<ActivationParams>>,
    /// URL used for the "Control" nav link on the simulation page.
    /// Set via `ROTELLE_CONTROL_URL`. Defaults to `/rotectl/control` (relative, works in
    /// full mode). In separate-pod deployments, set to the absolute control service URL.
    pub control_url: String,
    /// URL used for the "Status" nav link on the control panel.
    /// Set via `ROTELLE_SIM_URL`. Defaults to `/` (relative, works in full mode).
    /// In separate-pod deployments, set to the absolute sim service URL.
    pub sim_url: String,
    /// Base URL of the sim pod's API (e.g. `http://rotelle.rotelle.svc.cluster.local:8080`).
    /// Set via `ROTELLE_SIM_API_URL`. Only used in `control` mode; empty string otherwise.
    pub sim_api_url: String,
    /// HTTP client for proxying requests to the sim pod in `control` mode.
    pub http_client: reqwest::Client,
    /// Last successful `/rotectl/status` response from the sim pod.
    /// Used as a fallback in `control` mode so the panel keeps showing the
    /// active scenario instead of reverting to idle while the sim is restarting.
    pub last_known_sim_status: Arc<Mutex<Value>>,
}

impl AppState {
    /// Switch scenario, update in-memory state, persist to disk, and reset the crash flag.
    /// Use this when the change originates from the control API (authoritative write).
    pub fn switch_scenario(
        &self,
        new: Arc<dyn Scenario>,
        params: ActivationParams,
    ) -> &'static str {
        self.apply_scenario_inner(new, params, true)
    }

    /// Probe the sim pod's `/health` endpoint and return the health status string.
    /// Only meaningful in `control` mode; returns `"ok"` or `"crashed"` based on
    /// the response code, and `"down"` on connection error or timeout.
    pub async fn probe_sim_health(&self) -> String {
        match self
            .http_client
            .get(format!("{}/health", self.sim_api_url))
            .timeout(Duration::from_secs(3))
            .send()
            .await
        {
            Ok(r) if r.status().as_u16() == 200 => "ok".to_string(),
            Ok(_) => "crashed".to_string(),
            Err(_) => "down".to_string(),
        }
    }

    /// Fetch `/rotectl/status` from the sim pod and return the JSON response.
    /// On success the result is cached; on failure the last cached value is returned
    /// so the control panel keeps showing the active scenario while the sim restarts.
    pub async fn fetch_sim_status(&self) -> Value {
        let result = async {
            self.http_client
                .get(format!("{}/rotectl/status", self.sim_api_url))
                .timeout(Duration::from_secs(3))
                .send()
                .await?
                .json::<Value>()
                .await
        }
        .await
        .ok();

        if let Some(fresh) = result {
            *self.last_known_sim_status.lock().unwrap() = fresh.clone();
            fresh
        } else {
            self.last_known_sim_status.lock().unwrap().clone()
        }
    }

    fn apply_scenario_inner(
        &self,
        new: Arc<dyn Scenario>,
        params: ActivationParams,
        persist: bool,
    ) -> &'static str {
        // When no params are provided (e.g. plain "Activate" click), seed with
        // the scenario's declared defaults so the Parameters panel shows correct
        // initial values rather than empty inputs.
        let effective = if params.is_empty() {
            new.default_params()
        } else {
            params
        };
        let name;
        {
            let mut active = self.active_scenario.lock().unwrap();
            active.deactivate();
            new.activate(&effective);
            name = new.name();
            *active = new;
        }
        self.crashed.store(false, Ordering::SeqCst);
        *self.active_params.lock().unwrap() = effective.clone();
        if persist {
            persist_state(&self.state_file, name, &effective);
        }
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
