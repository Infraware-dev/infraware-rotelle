use serde_json::Value;
use std::collections::HashMap;

// ── Activation parameters ─────────────────────────────────────────────────────

/// Generic key-value parameters passed to a scenario on activation.
///
/// Backed by a JSON object so any scenario can accept any parameters without
/// changing this type. Read values with the typed helpers; unknown keys are ignored.
///
/// **Wire format** — supply parameters as flat JSON fields alongside `cmd`/`scenario`:
/// ```json
/// {"cmd": "set", "scenario": "oom-kill", "loop_time_secs": 10, "loop_amount_mb": 15}
/// ```
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ActivationParams(HashMap<String, Value>);

impl ActivationParams {
    /// Build params from a `serde_json::json!({...})` literal — the idiomatic way to
    /// declare `default_params` in a scenario implementation.
    ///
    /// ```rust
    /// fn default_params(&self) -> ActivationParams {
    ///     ActivationParams::from_json(serde_json::json!({
    ///         "loop_time_secs": 10,
    ///         "loop_amount_mb": 10
    ///     }))
    /// }
    /// ```
    pub fn from_json(v: Value) -> Self {
        if let Value::Object(map) = v {
            Self(map.into_iter().collect())
        } else {
            Self::default()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

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
}

impl From<HashMap<String, Value>> for ActivationParams {
    fn from(map: HashMap<String, Value>) -> Self {
        Self(map)
    }
}

// ── Scenario metadata (internal) ─────────────────────────────────────────────

/// Snapshot used by the control panel renderer. Built by `ScenarioRegistry::list()`
/// — not part of the public scenario-authoring API.
pub(crate) struct ScenarioMeta {
    pub(crate) name: &'static str,
    pub(crate) description: &'static str,
}

// ── Scenario trait ────────────────────────────────────────────────────────────

/// Every failure scenario implements this trait.
///
/// **To add a new scenario:** implement this trait in a new file, then add one
/// line to `catalog.rs`. See `CONTRIBUTING.md` for the full walkthrough.
pub trait Scenario: Send + Sync {
    /// API identifier — the value callers pass in the `scenario` field.
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

    /// Default parameter values for this scenario.
    ///
    /// Return an `ActivationParams` built from a JSON literal and the control panel
    /// renders the inputs automatically — no HTML or JS changes needed. The JSON value
    /// type determines the input type: numbers → `<input type="number">`, strings →
    /// `<input type="text">`. Default: no parameters.
    ///
    /// ```rust
    /// fn default_params(&self) -> ActivationParams {
    ///     ActivationParams::from_json(serde_json::json!({
    ///         "interval_secs": 10,
    ///         "label": "my-label"
    ///     }))
    /// }
    /// ```
    fn default_params(&self) -> ActivationParams {
        ActivationParams::default()
    }
}

// ── Index response ────────────────────────────────────────────────────────────

pub enum IndexEffect {
    /// Return HTTP 200 with this HTML body.
    Respond(String),
    /// Simulate a crash — sets the crash flag so GET /health returns 503,
    /// which triggers a Kubernetes pod restart while keeping the control plane alive.
    Exit,
    /// Hold the connection open indefinitely — simulates a Service with no reachable backends.
    /// The index route handler sleeps the async task; the thread pool stays unblocked.
    Hang,
    /// Return an HTTP response with a specific status code and HTML body.
    RespondWithStatus(u16, String),
    /// Sleep for the given number of milliseconds, then return HTTP 200 with this HTML body.
    /// Simulates a degraded-but-alive pod whose slow responses exceed readiness probe timeouts.
    /// The index route handler sleeps the async task; the thread pool stays unblocked.
    RespondAfterDelay(u64, String),
}

// ── Shared styles ─────────────────────────────────────────────────────────────

const SHARED_CSS: &str = r#"
    :root {
      color-scheme: light;
      --canvas: #f3eee4;
      --canvas-grid: rgba(71, 62, 50, 0.08);
      --surface: #fffaf1;
      --surface-raised: #fffdf8;
      --surface-muted: #ebe4d7;
      --ink: #20251f;
      --ink-soft: #5f625b;
      --ink-faint: #6b6c64;
      --line: #d6cdbc;
      --line-strong: #a89d8a;
      --accent: #c74422;
      --accent-hover: #a93418;
      --accent-soft: #f7d7c7;
      --healthy: #256b4a;
      --healthy-soft: #dcecdf;
      --warning: #92570d;
      --warning-soft: #f5e6c7;
      --danger: #a8332b;
      --danger-soft: #f3d9d5;
      --focus: #185f83;
      --shadow: 0 22px 55px rgba(66, 54, 37, 0.13);
      --mono: 'SFMono-Regular', Consolas, 'Liberation Mono', ui-monospace, monospace;
      --sans: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
    }

    [hidden] { display: none !important; }
    *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }

    body {
      min-height: 100vh;
      padding: clamp(1.25rem, 5vw, 4.5rem) clamp(1rem, 4vw, 2.5rem);
      color: var(--ink);
      background-color: var(--canvas);
      background-image:
        linear-gradient(var(--canvas-grid) 1px, transparent 1px),
        linear-gradient(90deg, var(--canvas-grid) 1px, transparent 1px),
        radial-gradient(circle at 85% 0%, rgba(228, 90, 50, 0.16), transparent 30rem);
      background-size: 32px 32px, 32px 32px, auto;
      font-family: var(--sans);
    }

    a, button, input { -webkit-tap-highlight-color: transparent; }
    :focus-visible { outline: 3px solid var(--focus); outline-offset: 3px; }

    .container { width: min(100%, 840px); margin: 0 auto; }
    .site-header {
      display: flex;
      align-items: center;
      gap: 1.5rem;
      margin-bottom: clamp(2rem, 6vw, 3.75rem);
    }
    .brand-lockup { display: flex; align-items: center; gap: 0.75rem; }
    .brand-mark-link { display: flex; flex: 0 0 auto; }
    .brand-mark { width: 48px; height: 48px; flex: 0 0 auto; filter: drop-shadow(0 5px 8px rgba(88, 55, 37, 0.2)); }
    .brand-copy { display: grid; gap: 0.2rem; }
    .brand-name { color: inherit; font-size: 1.6rem; font-weight: 800; letter-spacing: -0.055em; line-height: 1; text-decoration: none; }
    .brand-tagline, .footer-credit {
      color: var(--ink-faint);
      font-family: var(--mono);
      font-size: 0.7rem;
      letter-spacing: 0.02em;
      line-height: 1.4;
    }
    .brand-tagline a, .footer-credit a { color: inherit; text-decoration-color: var(--line-strong); text-underline-offset: 0.2em; }
    .brand-tagline a:hover, .footer-credit a:hover { color: var(--accent); }

    .header-nav { margin-left: auto; display: flex; gap: 0.4rem; align-items: center; }
    .nav-link {
      padding: 0.58em 1em;
      border: 1px solid transparent;
      border-radius: 9px;
      color: var(--ink-soft);
      font-family: var(--mono);
      font-size: 0.8rem;
      font-weight: 700;
      letter-spacing: 0.02em;
      text-decoration: none;
      transition: background 140ms ease, border-color 140ms ease, color 140ms ease;
    }
    .nav-link:hover { border-color: var(--line); background: rgba(255, 250, 241, 0.7); color: var(--ink); }
    .nav-link.nav-active { border-color: var(--ink); background: var(--ink); color: var(--surface); }

    .card {
      position: relative;
      overflow: visible;
      border: 1px solid var(--line-strong);
      border-radius: 18px;
      background: var(--surface-raised);
      box-shadow: var(--shadow);
    }
    .card::before {
      position: absolute;
      z-index: -1;
      inset: 10px -10px -10px 10px;
      border: 1px solid rgba(199, 68, 34, 0.35);
      border-radius: 18px;
      content: '';
    }
    .card-header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 1.25rem clamp(1.25rem, 5vw, 2.5rem);
      border-bottom: 1px solid var(--line);
      border-radius: 18px 18px 0 0;
      background: var(--surface-muted);
    }
    .card-label, .scenario-label, .details-label, .switch-label {
      color: var(--ink-soft);
      font-family: var(--mono);
      font-size: 0.75rem;
      font-weight: 700;
      letter-spacing: 0.1em;
      text-transform: uppercase;
    }
    .status-pill {
      display: flex;
      align-items: center;
      gap: 8px;
      padding: 0.42em 0.85em;
      border: 1px solid #a7c9b0;
      border-radius: 8px;
      background: var(--healthy-soft);
    }
    .status-dot { width: 8px; height: 8px; border-radius: 50%; background: var(--healthy); box-shadow: 0 0 0 3px rgba(37, 107, 74, 0.13); }
    .status-text { color: var(--healthy); font-family: var(--mono); font-size: 0.78rem; font-weight: 700; letter-spacing: 0.03em; }

    .card-body { padding: clamp(1.5rem, 6vw, 3rem); }
    .scenario-label { margin-bottom: 0.7rem; }
    .scenario-name {
      margin-bottom: 0.7rem;
      color: var(--ink);
      font-family: var(--mono);
      font-size: clamp(1.65rem, 5vw, 2.35rem);
      font-weight: 750;
      letter-spacing: -0.055em;
      line-height: 1.1;
      overflow-wrap: anywhere;
    }
    .scenario-name::before { color: var(--accent); content: './'; font-weight: 500; }
    .scenario-description { max-width: 60ch; color: var(--ink-soft); font-size: 1rem; line-height: 1.65; }
    .details-section { margin-top: 2rem; padding-top: 1.5rem; border-top: 1px dashed var(--line-strong); }
    .details-section:has(.card-extra:empty) { display: none; }
    .details-label { margin-bottom: 0.75rem; }
    .card-extra { color: var(--ink-soft); font-size: 1.05rem; font-weight: 500; line-height: 1.75; }
    .card-extra p { margin: 0; }
    .card-extra strong { color: var(--ink); font-weight: 700; }
    .card-extra code {
      padding: 0.17em 0.45em;
      border: 1px solid #e1ae97;
      border-radius: 5px;
      background: var(--accent-soft);
      color: #7e2a14;
      font-family: var(--mono);
      font-size: 0.88em;
    }
    .card-footer { padding: 1rem clamp(1.25rem, 5vw, 2.5rem); border-top: 1px solid var(--line); border-radius: 0 0 18px 18px; background: #f7f0e4; }

    @media (max-width: 620px) {
      .site-header { align-items: flex-start; flex-wrap: wrap; gap: 1rem; }
      .brand-mark { width: 42px; height: 42px; }
      .header-nav { width: 100%; margin-left: 0; }
      .nav-link { flex: 1; text-align: center; }
      .card::before { inset: 6px -6px -6px 6px; }
      .card-header { align-items: flex-start; gap: 0.75rem; }
    }

    @media (prefers-reduced-motion: reduce) {
      *, *::before, *::after { scroll-behavior: auto !important; transition-duration: 0.01ms !important; }
    }
"#;

// ── Control-page styles ───────────────────────────────────────────────────────

const CONTROL_CSS: &str = r#"
    .section-divider {
      margin-top: 1.75rem;
      padding-top: 1.75rem;
      border-top: 1px solid #2A2A2D;
    }
    .switch-label {
      font-size: 0.875rem;
      text-transform: uppercase;
      letter-spacing: 0.12em;
      color: #74757B;
      font-weight: 700;
      margin-bottom: 1rem;
    }

    /* Active scenario row: info left, health badge right */
    .active-scenario-row {
      display: flex;
      align-items: flex-start;
      justify-content: space-between;
      gap: 1.5rem;
    }
    .active-scenario-info { flex: 1; min-width: 0; }

    /* Simulation health indicator */
    .sim-status {
      display: inline-flex;
      align-items: center;
      gap: 7px;
      flex-shrink: 0;
      align-self: center;
      font-size: 0.82rem;
      font-weight: 600;
      padding: 0.35em 0.9em;
      border-radius: 20px;
    }
    .sim-dot {
      width: 7px;
      height: 7px;
      border-radius: 50%;
      flex-shrink: 0;
    }
    .sim-status-ok {
      color: #15803d;
      background: rgba(22,163,74,0.08);
      border: 1px solid rgba(22,163,74,0.22);
    }
    .sim-status-ok .sim-dot {
      background: #16a34a;
      box-shadow: 0 0 5px rgba(22,163,74,0.55);
    }
    .sim-status-crashed {
      color: #b45309;
      background: rgba(245,158,11,0.07);
      border: 1px solid rgba(245,158,11,0.3);
    }
    .sim-status-crashed .sim-dot {
      background: #d97706;
      box-shadow: 0 0 5px rgba(245,158,11,0.5);
    }
    .sim-status-down {
      color: #b91c1c;
      background: rgba(220,38,38,0.06);
      border: 1px solid rgba(220,38,38,0.22);
    }
    .sim-status-down .sim-dot {
      background: #dc2626;
      box-shadow: 0 0 5px rgba(220,38,38,0.45);
    }

    /* Custom combobox */
    .combobox { position: relative; margin-bottom: 1.25rem; }
    .combobox-trigger {
      width: 100%;
      display: flex;
      align-items: center;
      justify-content: space-between;
      gap: 0.5rem;
      padding: 0.65em 1em;
      font-size: 0.9rem;
      font-family: 'SF Mono', 'Fira Code', 'Cascadia Code', ui-monospace, monospace;
      color: #C8C9CD;
      background: #0C0C0E;
      border: 1.5px solid #2A2A2D;
      border-radius: 8px;
      outline: none;
      cursor: pointer;
      text-align: left;
      transition: border-color 0.15s, box-shadow 0.15s, border-radius 0.1s;
    }
    .combobox-trigger:hover { border-color: #54555C; }
    .combobox-trigger.is-open {
      border-color: #54555C;
      box-shadow: 0 0 0 3px rgba(255,255,255,0.05);
      border-bottom-left-radius: 0;
      border-bottom-right-radius: 0;
    }
    .combobox-chevron { color: #5A5B61; transition: transform 0.15s; flex-shrink: 0; }
    .combobox-trigger.is-open .combobox-chevron { transform: rotate(180deg); }
    .combobox-panel {
      position: absolute;
      top: 100%;
      left: 0;
      right: 0;
      z-index: 20;
      background: #161618;
      border: 1.5px solid #54555C;
      border-top: none;
      border-bottom-left-radius: 8px;
      border-bottom-right-radius: 8px;
      box-shadow: 0 8px 28px rgba(0,0,0,0.5);
      overflow: hidden;
    }
    .combobox-search-wrap {
      padding: 0.5rem;
      border-bottom: 1px solid #2A2A2D;
      position: relative;
    }
    .combobox-search-icon {
      position: absolute;
      left: 1rem;
      top: 50%;
      transform: translateY(-50%);
      width: 14px;
      height: 14px;
      color: #5A5B61;
      pointer-events: none;
    }
    .combobox-search {
      width: 100%;
      padding: 0.45em 0.75em 0.45em 2.1em;
      font-size: 0.85rem;
      font-family: inherit;
      color: #C8C9CD;
      background: #0C0C0E;
      border: 1px solid #2A2A2D;
      border-radius: 6px;
      outline: none;
      transition: border-color 0.1s;
    }
    .combobox-search:focus { border-color: #54555C; }
    .combobox-list {
      max-height: 280px;
      overflow-y: auto;
      padding: 0.3rem;
    }
    .combobox-option {
      padding: 0.65rem 0.85rem;
      border-radius: 6px;
      cursor: pointer;
      transition: background 0.1s;
    }
    .combobox-option:hover { background: rgba(255,255,255,0.04); }
    .combobox-option.is-selected { background: rgba(255,255,255,0.07); }
    .combobox-option-name {
      font-family: 'SF Mono', 'Fira Code', 'Cascadia Code', ui-monospace, monospace;
      font-size: 0.875rem;
      font-weight: 700;
      color: #ECEDF0;
      margin-bottom: 0.2rem;
      display: flex;
      align-items: center;
      gap: 0.45rem;
    }
    .active-dot {
      display: inline-block;
      width: 6px;
      height: 6px;
      border-radius: 50%;
      background: #16a34a;
      flex-shrink: 0;
    }
    .combobox-option-desc { font-size: 0.78rem; color: #85868C; line-height: 1.35; }
    .combobox-no-results {
      padding: 1rem;
      text-align: center;
      color: #5A5B61;
      font-size: 0.85rem;
    }

    /* Param inputs */
    .params-for { font-weight: 500; text-transform: none; letter-spacing: 0; color: #85868C; font-family: 'SF Mono', ui-monospace, monospace; font-size: 0.8rem; }
    .params-section {
      margin-bottom: 1.25rem;
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
      padding: 1rem 1.25rem;
      background: #111113;
      border: 1px solid #2A2A2D;
      border-radius: 8px;
    }
    .param-row { display: flex; align-items: center; gap: 0.6rem; flex-wrap: wrap; }
    .param-label {
      font-size: 0.8rem;
      font-family: 'SF Mono', ui-monospace, monospace;
      color: #85868C;
      font-weight: 600;
      min-width: 130px;
    }
    .param-input {
      font-size: 0.875rem;
      font-family: 'SF Mono', ui-monospace, monospace;
      color: #C8C9CD;
      background: #0C0C0E;
      border: 1.5px solid #2A2A2D;
      border-radius: 6px;
      padding: 0.3em 0.6em;
      outline: none;
      transition: border-color 0.15s, box-shadow 0.15s;
    }
    .param-input:focus { border-color: #54555C; box-shadow: 0 0 0 3px rgba(255,255,255,0.05); }

    /* Submit & feedback */
    .form-footer { display: flex; align-items: center; gap: 1rem; flex-wrap: wrap; }
    .submit-btn {
      display: inline-flex;
      align-items: center;
      background: #ECEDF0;
      color: #0C0C0E;
      border: none;
      border-radius: 8px;
      padding: 0.65em 1.6em;
      font-size: 0.9rem;
      font-weight: 700;
      letter-spacing: 0.02em;
      cursor: pointer;
      font-family: inherit;
      transition: background 0.15s, transform 0.08s;
    }
    .submit-btn:hover { background: #F4F5F7; }
    .submit-btn:active { transform: scale(0.97); }
    .submit-btn:disabled { opacity: 0.4; cursor: default; transform: none; }
    .submit-btn-outline {
      display: inline-flex;
      align-items: center;
      background: transparent;
      color: #C8C9CD;
      border: 1.5px solid #2A2A2D;
      border-radius: 8px;
      padding: 0.62em 1.6em;
      font-size: 0.9rem;
      font-weight: 700;
      letter-spacing: 0.02em;
      cursor: pointer;
      font-family: inherit;
      transition: background 0.15s, border-color 0.15s, transform 0.08s;
    }
    .submit-btn-outline:hover { background: rgba(255,255,255,0.04); border-color: #54555C; }
    .submit-btn-outline:active { transform: scale(0.97); }
    .submit-btn-outline:disabled { opacity: 0.4; cursor: default; transform: none; }
    .feedback {
      font-size: 0.875rem;
      font-weight: 600;
      padding: 0.5em 1em;
      border-radius: 8px;
      display: none;
    }
    .feedback.feedback-ok {
      display: inline-block;
      color: #15803d;
      background: rgba(22,163,74,0.08);
      border: 1px solid rgba(22,163,74,0.22);
    }
    .feedback.feedback-err {
      display: inline-block;
      color: #b91c1c;
      background: rgba(220,38,38,0.06);
      border: 1px solid rgba(220,38,38,0.18);
    }

    /* Rotelle control surface */
    .section-divider { margin-top: 2rem; padding-top: 1.75rem; border-top: 1px dashed var(--line-strong); }
    .switch-label { margin-bottom: 1rem; }
    .active-scenario-row { gap: 2rem; }

    .sim-status {
      gap: 7px;
      align-self: center;
      padding: 0.48em 0.8em;
      border-radius: 8px;
      font-family: var(--mono);
      font-size: 0.72rem;
      font-weight: 700;
      line-height: 1.35;
    }
    .sim-dot { width: 7px; height: 7px; }
    .sim-status-ok { color: var(--healthy); background: var(--healthy-soft); border-color: #a7c9b0; }
    .sim-status-ok .sim-dot { background: var(--healthy); box-shadow: 0 0 0 3px rgba(37, 107, 74, 0.13); }
    .sim-status-crashed { color: var(--warning); background: var(--warning-soft); border-color: #d8bc7f; }
    .sim-status-crashed .sim-dot { background: var(--warning); box-shadow: 0 0 0 3px rgba(146, 87, 13, 0.12); }
    .sim-status-down { color: var(--danger); background: var(--danger-soft); border-color: #d9aaa4; }
    .sim-status-down .sim-dot { background: var(--danger); box-shadow: 0 0 0 3px rgba(168, 51, 43, 0.12); }

    .combobox { margin-bottom: 1.25rem; }
    .combobox-trigger {
      min-height: 48px;
      padding: 0.7em 1em;
      border-color: var(--line-strong);
      border-radius: 9px;
      background: var(--surface);
      color: var(--ink);
      font-family: var(--mono);
      font-size: 0.88rem;
    }
    .combobox-trigger:hover { border-color: var(--accent); }
    .combobox-trigger.is-open { border-color: var(--accent); box-shadow: 0 0 0 3px rgba(199, 68, 34, 0.14); }
    .combobox-chevron { color: var(--accent); }
    .combobox-panel {
      border-color: var(--accent);
      border-radius: 0 0 10px 10px;
      background: var(--surface-raised);
      box-shadow: 0 16px 30px rgba(66, 54, 37, 0.18);
    }
    .combobox-search-wrap { border-color: var(--line); background: #f8f1e5; }
    .combobox-search-icon { color: var(--ink-faint); }
    .combobox-search {
      min-height: 40px;
      border-color: var(--line);
      background: var(--surface-raised);
      color: var(--ink);
      font-family: var(--sans);
    }
    .combobox-search:focus { border-color: var(--focus); box-shadow: 0 0 0 2px rgba(24, 95, 131, 0.12); }
    .combobox-list { max-height: 300px; padding: 0.4rem; }
    .combobox-option { border: 1px solid transparent; border-radius: 7px; padding: 0.72rem 0.85rem; }
    .combobox-option:hover, .combobox-option.is-focused { border-color: #edc0ac; background: #fbebdf; }
    .combobox-option.is-selected { border-color: #e1ae97; background: var(--accent-soft); }
    .combobox-option-name { color: var(--ink); font-family: var(--mono); }
    .active-dot { background: var(--accent); }
    .combobox-option-desc { color: var(--ink-soft); }
    .combobox-no-results { color: var(--ink-faint); }

    .params-for { color: var(--ink-faint); font-family: var(--mono); }
    .params-section { gap: 0.75rem; padding: 1.15rem 1.25rem; border-color: var(--line); border-radius: 10px; background: #f7f0e4; }
    .param-row { gap: 0.75rem; }
    .param-label { color: var(--ink-soft); font-family: var(--mono); }
    .param-input {
      min-height: 38px;
      border-color: var(--line-strong);
      border-radius: 7px;
      background: var(--surface-raised);
      color: var(--ink);
      font-family: var(--mono);
    }
    .param-input:focus { border-color: var(--focus); box-shadow: 0 0 0 3px rgba(24, 95, 131, 0.12); }

    .submit-btn, .submit-btn-outline {
      justify-content: center;
      min-height: 44px;
      border-radius: 9px;
      font-family: var(--mono);
      font-size: 0.82rem;
    }
    .submit-btn { border: 1px solid var(--accent); background: var(--accent); color: #fffaf1; box-shadow: 0 5px 12px rgba(199, 68, 34, 0.18); }
    .submit-btn:hover { background: var(--accent-hover); }
    .submit-btn-outline { border-color: var(--ink); color: var(--ink); }
    .submit-btn-outline:hover { border-color: var(--accent); background: var(--accent-soft); color: #7e2a14; }
    .submit-btn:disabled, .submit-btn-outline:disabled { opacity: 0.55; }
    .feedback { border-radius: 8px; font-family: var(--mono); font-size: 0.78rem; }
    .feedback.feedback-ok { color: var(--healthy); background: var(--healthy-soft); border-color: #a7c9b0; }
    .feedback.feedback-err { color: var(--danger); background: var(--danger-soft); border-color: #d9aaa4; }

    @media (max-width: 620px) {
      .active-scenario-row { flex-direction: column; gap: 1rem; }
      .sim-status { align-self: flex-start; }
      .param-row { align-items: stretch; flex-direction: column; }
      .param-label { min-width: 0; }
      .param-input { width: 100% !important; }
      .form-footer { align-items: stretch; flex-direction: column; }
      .submit-btn, .submit-btn-outline { width: 100%; }
    }
"#;

// ── Control-page script ───────────────────────────────────────────────────────

const CONTROL_JS: &str = r#"
(function () {
  'use strict';

  var combobox      = document.getElementById('combobox');
  var trigger       = document.getElementById('combobox-trigger');
  var panel         = document.getElementById('combobox-panel');
  var searchInput   = document.getElementById('scenario-search');
  var comboList     = document.getElementById('combobox-list');
  var noResults     = document.getElementById('combobox-no-results');
  var hiddenInput   = document.getElementById('scenario-hidden');
  var currentLbl    = document.getElementById('combobox-current');
  var activateForm  = document.getElementById('activate-form');
  var activateBtn   = document.getElementById('activate-btn');
  var activateFb    = document.getElementById('activate-feedback');
  var paramsForm    = document.getElementById('params-form');
  var paramsBtn     = document.getElementById('params-btn');
  var paramsFb      = document.getElementById('params-feedback');
  var simBadge      = document.getElementById('sim-badge');
  var focusedIndex  = -1;

  /* ── Combobox ───────────────────────────────────────────────────────── */
  function visibleOptions() {
    return Array.prototype.filter.call(
      document.querySelectorAll('.combobox-option'),
      function (opt) { return !opt.hidden; }
    );
  }

  function focusOption(index) {
    var options = visibleOptions();
    document.querySelectorAll('.combobox-option').forEach(function (opt) {
      opt.classList.remove('is-focused');
    });
    if (!options.length) {
      focusedIndex = -1;
      searchInput.removeAttribute('aria-activedescendant');
      return;
    }
    focusedIndex = (index + options.length) % options.length;
    var option = options[focusedIndex];
    option.classList.add('is-focused');
    searchInput.setAttribute('aria-activedescendant', option.id);
    option.scrollIntoView({ block: 'nearest' });
  }

  function openPanel(preferredIndex) {
    panel.hidden = false;
    trigger.classList.add('is-open');
    trigger.setAttribute('aria-expanded', 'true');
    searchInput.value = '';
    filterOptions('');
    searchInput.focus();
    var options = visibleOptions();
    var selectedIndex = options.findIndex(function (opt) {
      return opt.dataset.value === hiddenInput.value;
    });
    focusOption(typeof preferredIndex === 'number' ? preferredIndex : Math.max(selectedIndex, 0));
  }

  function closePanel() {
    panel.hidden = true;
    trigger.classList.remove('is-open');
    trigger.setAttribute('aria-expanded', 'false');
    focusedIndex = -1;
    searchInput.removeAttribute('aria-activedescendant');
    document.querySelectorAll('.combobox-option').forEach(function (opt) {
      opt.classList.remove('is-focused');
    });
  }

  function selectScenario(value) {
    hiddenInput.value = value;
    currentLbl.textContent = value;
    document.querySelectorAll('.combobox-option').forEach(function (opt) {
      var selected = opt.dataset.value === value;
      opt.classList.toggle('is-selected', selected);
      opt.setAttribute('aria-selected', selected ? 'true' : 'false');
    });
    closePanel();
    trigger.focus();
  }

  function filterOptions(query) {
    var q = (query || '').toLowerCase().trim();
    var any = false;
    document.querySelectorAll('.combobox-option').forEach(function (opt) {
      var name = (opt.querySelector('.combobox-option-name') || {}).textContent || '';
      var desc = (opt.querySelector('.combobox-option-desc') || {}).textContent || '';
      var show = !q || name.toLowerCase().includes(q) || desc.toLowerCase().includes(q);
      opt.hidden = !show;
      if (show) any = true;
    });
    if (noResults) noResults.hidden = any;
    focusOption(0);
  }

  trigger.addEventListener('click', function () {
    if (panel.hidden) { openPanel(); } else { closePanel(); }
  });
  trigger.addEventListener('keydown', function (e) {
    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      e.preventDefault();
      if (panel.hidden) openPanel(e.key === 'ArrowUp' ? -1 : 0);
    }
  });
  searchInput.addEventListener('input', function () { filterOptions(this.value); });
  searchInput.addEventListener('keydown', function (e) {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      focusOption(focusedIndex + 1);
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      focusOption(focusedIndex - 1);
    } else if (e.key === 'Enter') {
      var options = visibleOptions();
      if (focusedIndex >= 0 && options[focusedIndex]) {
        e.preventDefault();
        selectScenario(options[focusedIndex].dataset.value);
      }
    } else if (e.key === 'Home') {
      e.preventDefault();
      focusOption(0);
    } else if (e.key === 'End') {
      e.preventDefault();
      focusOption(-1);
    } else if (e.key === 'Escape') {
      e.preventDefault();
      closePanel();
      trigger.focus();
    } else if (e.key === 'Tab') {
      closePanel();
    }
  });
  comboList.addEventListener('click', function (e) {
    var opt = e.target.closest('.combobox-option');
    if (opt && !opt.hidden) { selectScenario(opt.dataset.value); }
  });
  comboList.addEventListener('mousemove', function (e) {
    var opt = e.target.closest('.combobox-option');
    if (!opt || opt.hidden) return;
    focusOption(visibleOptions().indexOf(opt));
  });
  document.addEventListener('click', function (e) {
    if (combobox && !combobox.contains(e.target)) { closePanel(); }
  });

  /* ── Shared POST helper ─────────────────────────────────────────────── */
  function postCmd(body, btn, fb) {
    btn.disabled = true;
    fb.className = 'feedback';
    fb.textContent = '';
    fetch('/rotectl/cmd', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body)
    })
      .then(function (r) { return r.json(); })
      .then(function (d) {
        if (d.ok) {
          fb.className = 'feedback feedback-ok';
          fb.textContent = 'Active: ' + d.scenario;
          setTimeout(function () { window.location.reload(); }, 600);
        } else {
          fb.className = 'feedback feedback-err';
          fb.textContent = d.error || 'Unknown error.';
          btn.disabled = false;
        }
      })
      .catch(function (err) {
        fb.className = 'feedback feedback-err';
        fb.textContent = 'Network error: ' + err.message;
        btn.disabled = false;
      });
  }

  /* ── Activate Scenario ──────────────────────────────────────────────── */
  if (activateForm) {
    activateForm.addEventListener('submit', function (e) {
      e.preventDefault();
      var scenario = hiddenInput.value;
      if (!scenario) return;
      postCmd({ cmd: 'set', scenario: scenario }, activateBtn, activateFb);
    });
  }

  /* ── Update Parameters ──────────────────────────────────────────────── */
  if (paramsForm) {
    paramsForm.addEventListener('submit', function (e) {
      e.preventDefault();
      var scenario = document.getElementById('params-scenario-hidden').value;
      var body = { cmd: 'set', scenario: scenario };
      paramsForm.querySelectorAll('[data-param-name]').forEach(function (input) {
        var key  = input.dataset.paramName;
        var type = input.dataset.paramType;
        var val  = input.value.trim();
        if (!key) return;
        if (type === 'number') {
          var n = parseFloat(val);
          if (!isNaN(n)) body[key] = n;
        } else {
          body[key] = val;
        }
      });
      postCmd(body, paramsBtn, paramsFb);
    });
  }

  /* ── Simulation health polling ──────────────────────────────────────── */
  if (simBadge) {
    function refreshSimHealth() {
      fetch('/rotectl/sim-health')
        .then(function (r) { return r.json(); })
        .then(function (d) {
          var status = d.status || 'down';
          simBadge.className = 'sim-status sim-status-' + status;
          var txt = simBadge.querySelector('.sim-text');
          if (txt) {
            if (status === 'ok') txt.textContent = 'Simulation reachable';
            else if (status === 'crashed') txt.textContent = 'Crash active — pod restarting';
            else txt.textContent = 'Container down — pod restarting';
          }
        })
        .catch(function () {});
    }
    refreshSimHealth();
    setInterval(refreshSimHealth, 2000);
  }
}());
"#;

// ── HTML helpers ──────────────────────────────────────────────────────────────

fn logo_nav_html(active_href: &str, sim_url: &str, control_url: &str) -> String {
    let status_active = if active_href == "/" {
        " nav-active"
    } else {
        ""
    };
    let control_link = if control_url.is_empty() {
        String::new()
    } else {
        let active = if active_href != "/" {
            " nav-active"
        } else {
            ""
        };
        let current = if active_href != "/" {
            r#" aria-current="page""#
        } else {
            ""
        };
        format!(r#"<a href="{control_url}" class="nav-link{active}"{current}>Control</a>"#)
    };
    let status_current = if active_href == "/" {
        r#" aria-current="page""#
    } else {
        ""
    };
    format!(
        r#"<header class="site-header">
      <div class="brand-lockup">
        <a class="brand-mark-link" href="{sim_url}" aria-label="Rotelle status">
          <img class="brand-mark" src="/rotelle-mark.png" alt="">
        </a>
        <span class="brand-copy">
          <a class="brand-name" href="{sim_url}">rotelle</a>
          <span class="brand-tagline">open-source chaos lab by <a href="https://infraware.dev">infraware.dev</a></span>
        </span>
      </div>
      <nav class="header-nav" aria-label="Primary navigation">
        <a href="{sim_url}" class="nav-link{status_active}"{status_current}>Status</a>
        {control_link}
      </nav>
    </header>"#
    )
}

fn footer_credit_html() -> &'static str {
    r#"<span class="footer-credit">Rotelle is an open-source chaos lab by <a href="https://infraware.dev">infraware.dev</a></span>"#
}

/// Renders param input rows for the currently active scenario (always visible, no hidden wrapper).
fn build_active_params_html(params: &ActivationParams) -> String {
    if params.0.is_empty() {
        return String::new();
    }
    let mut keys: Vec<&String> = params.0.keys().collect();
    keys.sort();
    let mut rows = String::new();
    for (index, key) in keys.into_iter().enumerate() {
        let val = &params.0[key];
        let (input_type, width, value_str) = match val {
            Value::Number(n) => ("number", "width:110px", n.to_string()),
            Value::String(s) => ("text", "width:210px", s.clone()),
            other => ("text", "width:210px", other.to_string()),
        };
        rows.push_str(&format!(
            r#"<div class="param-row">
            <label class="param-label" for="scenario-param-{index}">{key}</label>
            <input class="param-input" id="scenario-param-{index}" type="{input_type}" value="{value_str}"
                   data-param-name="{key}" data-param-type="{input_type}"
                   style="{width}">
          </div>"#
        ));
    }
    rows
}

// ── Page builders ─────────────────────────────────────────────────────────────

pub fn page_html(scenario: &str, body: &str, control_url: &str, sim_url: &str) -> String {
    let nav = logo_nav_html("/", sim_url, control_url);
    let footer_credit = footer_credit_html();
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Rotelle &mdash; Status</title>
  <link rel="icon" type="image/png" href="/rotelle-mark.png">
  <style>{SHARED_CSS}</style>
</head>
<body>
  <div class="container">
    {nav}
    <main class="card">
      <div class="card-header">
        <span class="card-label">Service Status</span>
        <div class="status-pill" role="status">
          <div class="status-dot"></div>
          <span class="status-text">running</span>
        </div>
      </div>
      <div class="card-body">
        <div class="scenario-label">Active scenario</div>
        <div class="scenario-name">{scenario}</div>
        {body}
      </div>
      <footer class="card-footer">{footer_credit}</footer>
    </main>
  </div>
</body>
</html>"##
    )
}

pub fn control_html(
    active_scenario: &str,
    active_description: &str,
    active_params: &ActivationParams,
    scenarios: &[ScenarioMeta],
    sim_status: &str,
    control_url: &str,
    sim_url: &str,
) -> String {
    let nav = logo_nav_html("/rotectl/control", sim_url, control_url);
    let footer_credit = footer_credit_html();

    let sim_badge_class = format!("sim-status sim-status-{sim_status}");
    let sim_badge_text = match sim_status {
        "ok" => "Simulation reachable",
        "crashed" => "Crash active \u{2014} pod restarting",
        _ => "Container down \u{2014} pod restarting",
    };

    let mut combobox_options = String::new();
    for (index, meta) in scenarios.iter().enumerate() {
        let is_selected = meta.name == active_scenario;
        let selected_class = if is_selected { " is-selected" } else { "" };
        let active_dot = if is_selected {
            r#"<span class="active-dot"></span>"#
        } else {
            ""
        };
        combobox_options.push_str(&format!(
            r#"<div class="combobox-option{selected_class}" id="scenario-option-{index}" role="option" aria-selected="{is_selected}" data-value="{name}">
              <div class="combobox-option-name">{active_dot}{name}</div>
              <div class="combobox-option-desc">{desc}</div>
            </div>"#,
            name = meta.name,
            desc = meta.description,
        ));
    }

    // Parameters section — only shown when the active scenario has configurable params.
    let params_section = {
        let param_rows = build_active_params_html(active_params);
        if param_rows.is_empty() {
            String::new()
        } else {
            format!(
                r#"<div class="section-divider">
          <div class="switch-label">Parameters <span class="params-for">— {active_scenario}</span></div>
          <form id="params-form">
            <input type="hidden" id="params-scenario-hidden" value="{active_scenario}">
            <div class="params-section">
              {param_rows}
            </div>
            <div class="form-footer">
              <button type="submit" class="submit-btn-outline" id="params-btn">Update Parameters</button>
              <div class="feedback" id="params-feedback" role="status" aria-live="polite"></div>
            </div>
          </form>
        </div>"#
            )
        }
    };

    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Rotelle &mdash; Control</title>
  <link rel="icon" type="image/png" href="/rotelle-mark.png">
  <style>{SHARED_CSS}{CONTROL_CSS}</style>
</head>
<body>
  <div class="container">
    {nav}
    <main class="card">
      <div class="card-header">
        <span class="card-label">Scenario Control</span>
      </div>
      <div class="card-body">
        <div class="active-scenario-row">
          <div class="active-scenario-info">
            <div class="scenario-label">Active Scenario</div>
            <div class="scenario-name">{active_scenario}</div>
            <div class="scenario-description">{active_description}</div>
          </div>
          <div id="sim-badge" class="{sim_badge_class}" role="status" aria-live="polite">
            <div class="sim-dot"></div>
            <span class="sim-text">{sim_badge_text}</span>
          </div>
        </div>
        <div class="section-divider">
          <div class="switch-label" id="scenario-picker-label">Switch Scenario</div>
          <form id="activate-form">
            <div class="combobox" id="combobox">
              <button type="button" class="combobox-trigger" id="combobox-trigger"
                      aria-haspopup="listbox" aria-expanded="false" aria-controls="combobox-list"
                      aria-labelledby="scenario-picker-label combobox-current">
                <span id="combobox-current">{active_scenario}</span>
                <svg class="combobox-chevron" viewBox="0 0 12 8" fill="none" xmlns="http://www.w3.org/2000/svg" width="12" height="8">
                  <path d="M1 1l5 5 5-5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/>
                </svg>
              </button>
              <div class="combobox-panel" id="combobox-panel" hidden>
                <div class="combobox-search-wrap">
                  <svg class="combobox-search-icon" viewBox="0 0 16 16" fill="none" xmlns="http://www.w3.org/2000/svg">
                    <circle cx="6.5" cy="6.5" r="4.25" stroke="currentColor" stroke-width="1.5"/>
                    <path d="M10 10l3 3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round"/>
                  </svg>
                  <input type="search" id="scenario-search" class="combobox-search"
                         placeholder="Search scenarios&hellip;" autocomplete="off" spellcheck="false"
                         role="combobox" aria-autocomplete="list" aria-expanded="true"
                         aria-controls="combobox-list" aria-label="Search scenarios">
                </div>
                <div class="combobox-list" id="combobox-list" role="listbox" aria-label="Scenarios">
                  {combobox_options}
                  <div id="combobox-no-results" class="combobox-no-results" role="status" hidden>No scenarios match.</div>
                </div>
              </div>
              <input type="hidden" id="scenario-hidden" value="{active_scenario}">
            </div>
            <div class="form-footer">
              <button type="submit" class="submit-btn" id="activate-btn">Activate Scenario</button>
              <div class="feedback" id="activate-feedback" role="status" aria-live="polite"></div>
            </div>
          </form>
        </div>
        {params_section}
      </div>
      <footer class="card-footer">{footer_credit}</footer>
    </main>
  </div>
  <script>{CONTROL_JS}</script>
</body>
</html>"##
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_page_uses_rotelle_identity() {
        let html = page_html(
            "none, idle",
            "<p>Ready for a scenario.</p>",
            "/rotectl/control",
            "/",
        );

        assert!(html.contains("<title>Rotelle &mdash; Status</title>"));
        assert!(html.contains("href=\"/rotelle-mark.png\""));
        assert!(html.contains("src=\"/rotelle-mark.png\""));
        assert!(html.contains("<a class=\"brand-name\" href=\"/\">rotelle</a>"));
        assert!(html.contains(
            "open-source chaos lab by <a href=\"https://infraware.dev\">infraware.dev</a>"
        ));
        assert!(html.contains("class=\"nav-link nav-active\" aria-current=\"page\">Status"));
        assert!(!html.contains("class=\"logo-img\""));
    }

    #[test]
    fn control_page_exposes_accessible_scenario_picker() {
        let active_params = ActivationParams::from_json(serde_json::json!({
            "interval_secs": 10
        }));
        let scenarios = vec![
            ScenarioMeta {
                name: "none, idle",
                description: "No failure active.",
            },
            ScenarioMeta {
                name: "slow-response",
                description: "Responds after a delay.",
            },
        ];
        let html = control_html(
            "slow-response",
            "Responds after a delay.",
            &active_params,
            &scenarios,
            "ok",
            "/rotectl/control",
            "/",
        );

        assert!(html.contains("<title>Rotelle &mdash; Control</title>"));
        assert!(html.contains("aria-haspopup=\"listbox\" aria-expanded=\"false\""));
        assert!(html.contains("role=\"listbox\" aria-label=\"Scenarios\""));
        assert!(
            html.contains("role=\"option\" aria-selected=\"true\" data-value=\"slow-response\"")
        );
        assert!(html.contains("data-param-name=\"interval_secs\""));
        assert!(html.contains("role=\"status\" aria-live=\"polite\""));
        assert!(html.contains("class=\"nav-link nav-active\" aria-current=\"page\">Control"));
    }
}
