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

    /// Read a param as `String` — use for named configuration values (e.g. `required_var`).
    pub fn get_string(&self, key: &str) -> Option<String> {
        self.0.get(key)?.as_str().map(|s| s.to_string())
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
    /// Hold the connection open indefinitely — simulates a Service with no reachable backends.
    /// The index route handler sleeps the async task; the thread pool stays unblocked.
    Hang,
    /// Return an HTTP response with a specific status code and HTML body.
    RespondWithStatus(u16, String),
}

// ── HTML helper ───────────────────────────────────────────────────────────────

pub fn page_html(case: &str, body: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Rotelle</title>
  <style>
    *, *::before, *::after {{ box-sizing: border-box; }}
    body {{
      font-family: system-ui, sans-serif;
      background: #f5f5f5;
      color: #222;
      margin: 0;
      padding: 2rem 1rem;
    }}
    .card {{
      background: #fff;
      border: 1px solid #ddd;
      border-radius: 8px;
      max-width: 560px;
      margin: 0 auto;
      padding: 2rem;
    }}
    h1 {{ margin: 0 0 1rem; font-size: 1.4rem; color: #111; }}
    .case {{
      display: inline-block;
      background: #f0f0f0;
      border-radius: 4px;
      padding: 0.2em 0.5em;
      font-family: monospace;
      font-size: 0.95rem;
    }}
    p {{ margin: 0.75rem 0 0; color: #444; }}
  </style>
</head>
<body>
  <div class="card">
    <h1>Rotelle</h1>
    <p>Active case: <span class="case">{case}</span></p>
    {body}
  </div>
</body>
</html>"#
    )
}
