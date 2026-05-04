use std::collections::HashMap;
use serde_json::Value;

// ── Activation parameters ─────────────────────────────────────────────────────

/// Generic key-value parameters passed to a scenario on activation.
///
/// Backed by a JSON object so any scenario can accept any parameters without
/// changing this type. Read values with the typed helpers; unknown keys are ignored.
///
/// **Wire format** — supply parameters as flat JSON fields alongside `cmd`/`case`:
/// ```json
/// {"cmd": "set", "case": "intermittent-02", "loop_time_secs": 10, "loop_amount_mb": 15}
/// ```
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ActivationParams(HashMap<String, Value>);

impl ActivationParams {
    /// Read a param as `u64` — use for durations and counters (e.g. `loop_time_secs`).
    pub fn get_u64(&self, key: &str) -> Option<u64> {
        self.0.get(key)?.as_u64()
    }

    /// Read a param as `usize` — use for buffer sizes where Rust requires `usize` (e.g. `loop_amount_mb`).
    pub fn get_usize(&self, key: &str) -> Option<usize> {
        self.0.get(key)?.as_u64().map(|v| v as usize)
    }

    /// Serialize the whole param map to JSON — use in `status_extras` to echo active params into `/rotectl/status`.
    pub fn to_json(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}

impl From<HashMap<String, Value>> for ActivationParams {
    fn from(map: HashMap<String, Value>) -> Self {
        Self(map)
    }
}

// ── Scenario trait ────────────────────────────────────────────────────────────

/// Every failure scenario implements this trait.
///
/// **To add a new scenario:** implement this trait in a new file, then add one
/// line to `catalog.rs`. See `CONTRIBUTING.md` for the full walkthrough.
pub trait Scenario: Send + Sync {
    /// API identifier — the value callers pass in the `case` field.
    fn name(&self) -> &'static str;

    /// Short description shown in `/rotectl/status`.
    fn description(&self) -> &'static str;

    /// Start the scenario's side-effects (background tasks, counters, …).
    fn activate(&self, params: &ActivationParams);

    /// Stop all side-effects and release resources.
    fn deactivate(&self);

    /// Called on every `GET /` while this scenario is active.
    fn on_index_request(&self) -> IndexEffect;

    /// Called on pod restart if this scenario was previously active.
    /// Default: calls `activate` — override only when resume must differ.
    fn on_resume(&self, params: &ActivationParams) {
        self.activate(params);
    }

    /// Extra fields merged into `/rotectl/status`. Default: nothing added.
    fn status_extras(&self) -> Value {
        Value::Null
    }
}

// ── Index response ────────────────────────────────────────────────────────────

pub enum IndexEffect {
    /// Return HTTP 200 with this HTML body.
    Respond(String),
    /// Exit the process — Kubernetes restarts the pod.
    Exit(i32),
}

// ── HTML helper ───────────────────────────────────────────────────────────────

pub fn page_html(case: &str, body: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html>
<head><title>Rotelle</title></head>
<body>
<h1>Rotelle</h1>
<p>Current failure case: <strong>{case}</strong></p>
{body}</body>
</html>"#
    )
}
