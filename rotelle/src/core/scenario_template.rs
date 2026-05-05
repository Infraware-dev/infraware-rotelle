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

pub fn page_html(case: &str, description: &str, body: &str) -> String {
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>infraware.dev &middot; rotelle</title>
  <link rel="icon" type="image/png" href="/favicon.png">
  <style>
    *, *::before, *::after {{ box-sizing: border-box; margin: 0; padding: 0; }}

    body {{
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', system-ui, sans-serif;
      background: #f0f2fe;
      color: #0f0f1e;
      min-height: 100vh;
      padding: 4.5rem 2.5rem;
      background-image: radial-gradient(ellipse 60% 50% at 75% -5%, rgba(99,102,241,0.15) 0%, transparent 60%);
    }}

    .container {{
      max-width: 760px;
      margin: 0 auto;
    }}

    .logo {{
      margin-bottom: 3.5rem;
      display: flex;
      align-items: center;
      gap: 1.1rem;
    }}
    .logo-img {{
      height: 64px;
      width: auto;
    }}
    .logo-divider {{
      width: 1px;
      height: 36px;
      background: rgba(55,48,163,0.2);
      flex-shrink: 0;
    }}
    .logo-subtitle {{
      font-size: 1.1rem;
      text-transform: uppercase;
      letter-spacing: 0.2em;
      color: #9090c0;
      font-weight: 700;
    }}

    .card {{
      background: #ffffff;
      border: 1px solid rgba(0,0,0,0.08);
      border-radius: 20px;
      overflow: hidden;
      box-shadow:
        0 1px 3px rgba(0,0,0,0.06),
        0 12px 40px rgba(99,102,241,0.1),
        0 40px 80px rgba(0,0,0,0.07);
    }}

    .card-header {{
      padding: 1.5rem 2.5rem;
      border-bottom: 1px solid rgba(0,0,0,0.06);
      display: flex;
      align-items: center;
      justify-content: space-between;
      background: #fafbff;
    }}
    .card-label {{
      font-size: 0.9rem;
      text-transform: uppercase;
      letter-spacing: 0.12em;
      color: #7070a0;
      font-weight: 700;
    }}
    .status-pill {{
      display: flex;
      align-items: center;
      gap: 8px;
      background: rgba(22,163,74,0.08);
      border: 1px solid rgba(22,163,74,0.22);
      border-radius: 20px;
      padding: 0.4em 1.1em;
    }}
    .status-dot {{
      width: 8px;
      height: 8px;
      border-radius: 50%;
      background: #16a34a;
      box-shadow: 0 0 6px rgba(22,163,74,0.6);
    }}
    .status-text {{
      font-size: 0.875rem;
      color: #15803d;
      font-weight: 600;
      letter-spacing: 0.04em;
    }}

    .card-body {{ padding: 2.5rem; }}

    .scenario-label {{
      font-size: 0.875rem;
      text-transform: uppercase;
      letter-spacing: 0.12em;
      color: #8888b8;
      font-weight: 700;
      margin-bottom: 0.65rem;
    }}

    .case-name {{
      font-size: 1.85rem;
      font-weight: 700;
      letter-spacing: -0.02em;
      color: #3730a3;
      line-height: 1.15;
      font-family: 'SF Mono', 'Fira Code', 'Cascadia Code', ui-monospace, monospace;
      margin-bottom: 0.6rem;
    }}

    .case-description {{
      font-size: 1rem;
      color: #6868a0;
      line-height: 1.5;
    }}

    .details-section {{
      margin-top: 1.75rem;
      padding-top: 1.75rem;
      border-top: 1px solid rgba(0,0,0,0.06);
    }}
    .details-section:has(.card-extra:empty) {{ display: none; }}
    .details-label {{
      font-size: 0.875rem;
      text-transform: uppercase;
      letter-spacing: 0.12em;
      color: #8888b8;
      font-weight: 700;
      margin-bottom: 0.65rem;
    }}
    .card-extra {{
      color: #2e2e6a;
      font-size: 1.2rem;
      font-weight: 500;
      line-height: 1.75;
    }}
    .card-extra p {{ margin: 0; }}
    .card-extra strong {{ color: #0f0f1e; font-weight: 600; }}
    .card-extra code {{
      font-family: 'SF Mono', ui-monospace, monospace;
      font-size: 0.9em;
      background: rgba(99,102,241,0.07);
      border: 1px solid rgba(99,102,241,0.15);
      border-radius: 5px;
      padding: 0.15em 0.45em;
      color: #4338ca;
    }}

    .card-footer {{
      padding: 1.25rem 2.5rem;
      border-top: 1px solid rgba(0,0,0,0.06);
      background: #fafbff;
    }}
    .footer-text {{
      font-size: 0.9rem;
      color: #5c5c88;
      letter-spacing: 0.02em;
    }}
  </style>
</head>
<body>
  <div class="container">
    <div class="logo">
      <img class="logo-img" src="/logo.png" alt="infraware.dev">
      <div class="logo-divider"></div>
      <span class="logo-subtitle">rotelle</span>
    </div>
    <div class="card">
      <div class="card-header">
        <span class="card-label">Scenario Status</span>
        <div class="status-pill">
          <div class="status-dot"></div>
          <span class="status-text">active</span>
        </div>
      </div>
      <div class="card-body">
        <div class="scenario-label">Active Failure Case</div>
        <div class="case-name">{case}</div>
        <div class="case-description">{description}</div>
        <div class="details-section">
          <div class="details-label">Details</div>
          <div class="card-extra">{body}</div>
        </div>
      </div>
      <div class="card-footer">
        <span class="footer-text">rotelle &middot; infraware.dev</span>
      </div>
    </div>
  </div>
</body>
</html>"##
    )
}
