use super::{ActivationParams, IndexEffect, Scenario};
use std::sync::Mutex;

const DEFAULT_VERSION: &str = "v1";

/// Simulates a pod serving stale configuration after a ConfigMap update.
///
/// Kubernetes never refreshes a ConfigMap consumed as environment variables —
/// the values are snapshotted into the pod at start and only change on a pod
/// restart. This scenario models that silent config drift: it captures a
/// `version` string on activation and serves it on every `GET /`, unchanged,
/// until re-activated with a new `version` (the manual `kubectl rollout restart`
/// a real fix requires).
///
/// Unlike every other scenario the failure is silent and data-level: no crash,
/// no error, no latency — just subtly wrong data. Diagnosis requires comparing
/// the value the pod serves against the current ConfigMap.
pub struct ConfigStaleScenario {
    version: Mutex<String>,
}

impl ConfigStaleScenario {
    pub fn new() -> Self {
        Self {
            version: Mutex::new(DEFAULT_VERSION.to_string()),
        }
    }
}

impl Scenario for ConfigStaleScenario {
    fn name(&self) -> &'static str {
        "config-stale"
    }

    fn description(&self) -> &'static str {
        "Serves a stale config version on every request — models a pod ignoring a ConfigMap update. Set `version` to change what is served."
    }

    fn activate(&self, params: &ActivationParams) {
        let version = params
            .get_string("version")
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| DEFAULT_VERSION.to_string());
        *self.version.lock().unwrap() = version.clone();
        tracing::info!(version = %version, "config-stale: activated");
    }

    fn deactivate(&self) {}

    fn on_index_request(&self) -> IndexEffect {
        let version = self.version.lock().unwrap().clone();
        IndexEffect::Respond(format!(
            "<p>Serving configuration <code>version={version}</code>.</p>\
             <p>This value was snapshotted at pod start and will not change until \
             the pod is restarted — even if the ConfigMap has since been updated.</p>"
        ))
    }

    fn status_extras(&self) -> serde_json::Value {
        let version = self.version.lock().unwrap().clone();
        serde_json::json!({ "served_version": version })
    }

    fn default_params(&self) -> ActivationParams {
        ActivationParams::from_json(serde_json::json!({
            "version": DEFAULT_VERSION
        }))
    }
}
