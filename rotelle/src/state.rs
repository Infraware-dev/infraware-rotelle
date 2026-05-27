use crate::core::registry::ScenarioRegistry;
use crate::core::{ActivationParams, Scenario};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Runtime mode — set via the `ROTELLE_MODE` environment variable.
/// - `full` (default): simulation surface + control plane in a single process on one port.
/// - `sim`: simulation surface only. Polls `state.json` for live scenario changes and
///   writes a heartbeat file so the control sidecar can check health.
/// - `control`: control plane only. Reads `state.json` as source of truth; checks the
///   heartbeat file to report simulation health.
#[derive(Clone, PartialEq, Debug)]
pub enum Mode {
    Full,
    Sim,
    Control,
}

/// Shared application state, cheaply cloneable via inner `Arc`s.
#[derive(Clone)]
pub struct AppState {
    pub active_scenario: Arc<Mutex<Arc<dyn Scenario>>>,
    pub data_dir: String,
    pub state_file: String,
    pub registry: Arc<ScenarioRegistry>,
    pub mode: Mode,
    /// Set when the active scenario triggers a simulated crash (`IndexEffect::Exit`).
    /// In `full` / `sim` mode: causes `GET /health` to return 503 so Kubernetes restarts
    /// the pod while the control plane stays accessible.
    /// In `control` mode: not used (crash state is read from the sim heartbeat file).
    pub crashed: Arc<AtomicBool>,
    /// Params that were last applied to the active scenario. Kept in sync by
    /// `apply_scenario_inner` so the control panel can display current values.
    pub active_params: Arc<Mutex<ActivationParams>>,
    /// URL used for the "Control" nav link on the simulation page.
    /// Set via `ROTELLE_CONTROL_URL`. Defaults to `/rotectl/control` (relative, works in
    /// full mode). In sidecar deployments, set to the absolute control service URL
    /// (e.g. `http://localhost:9090/rotectl/control`) so the browser reaches the correct port.
    pub control_url: String,
    /// URL used for the "Status" nav link on the control panel.
    /// Set via `ROTELLE_SIM_URL`. Defaults to `/` (relative, works in full mode).
    /// In sidecar deployments, set to the absolute sim service URL (e.g. `http://localhost:8080`)
    /// so the browser reaches the correct port.
    pub sim_url: String,
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

    /// Apply a scenario change that was loaded from `state.json` by the background poller.
    /// Updates in-memory state without re-writing the file (avoids a write→poll→write loop).
    pub fn apply_scenario(&self, new: Arc<dyn Scenario>, params: ActivationParams) {
        self.apply_scenario_inner(new, params, false);
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
        self.write_sim_health("ok");
        *self.active_params.lock().unwrap() = effective.clone();
        if persist {
            persist_state(&self.state_file, name, &effective);
        }
        name
    }

    /// Write the sim health file immediately (sim mode only).
    /// Called on crash and recovery so the control sidecar sees the change
    /// without waiting for the 2-second background writer interval.
    pub fn write_sim_health(&self, status: &str) {
        if self.mode == Mode::Sim {
            let _ = std::fs::write(format!("{}/sim_health", self.data_dir), status);
        }
    }

    /// Returns the current simulation health status as a string:
    /// - `"ok"` — simulation is running normally.
    /// - `"crashed"` — simulation hit `IndexEffect::Exit`; health probe is failing.
    /// - `"down"` — simulation process is not running (e.g. OOMKilled); heartbeat is stale.
    ///
    /// In `full` / `sim` mode: reads the in-process `crashed` flag.
    /// In `control` mode: reads the heartbeat file written by the sim container.
    pub fn sim_health_status(&self) -> String {
        match self.mode {
            Mode::Control => {
                let path = format!("{}/sim_health", self.data_dir);
                let meta = match std::fs::metadata(&path) {
                    Ok(m) => m,
                    Err(_) => return "down".to_string(),
                };
                let age_secs = meta
                    .modified()
                    .ok()
                    .and_then(|t| std::time::SystemTime::now().duration_since(t).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(u64::MAX);
                if age_secs > 30 {
                    return "down".to_string();
                }
                std::fs::read_to_string(&path)
                    .map(|s| match s.trim() {
                        "ok" | "crashed" => s.trim().to_string(),
                        _ => "down".to_string(),
                    })
                    .unwrap_or_else(|_| "down".to_string())
            }
            _ => {
                if self.crashed.load(Ordering::SeqCst) {
                    "crashed".to_string()
                } else {
                    "ok".to_string()
                }
            }
        }
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
