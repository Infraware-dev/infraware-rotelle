# Architecture

## Overview

Rotelle is a single Rust binary that runs as a Kubernetes deployment. The same binary
is started in two different modes via the `ROTELLE_MODE` environment variable:

| Mode | Port | Handles | Purpose |
|---|---|---|---|
| `sim` | 8080 | `GET /`, `GET /health` | Simulation surface — intentionally crashable |
| `control` | 9090 | `GET /rotectl/*`, `POST /rotectl/cmd` | Control plane — always accessible |
| `full` (default) | 8080 | everything | Single-process mode for local development |

The two-container sidecar pattern in `k8s/` runs `sim` and `control` as separate containers
in the same pod, sharing a PVC. When the `oom-kill` scenario causes the sim container to be
OOMKilled, the control sidecar continues running — its memory limit is set high enough that
it is never killed by the OOM scenario. See [Control plane crash isolation](#control-plane-crash-isolation).

## Module layout

```
src/
├── main.rs                    — startup: build registry, load state, start server
├── catalog.rs                 — list of all scenario factories  ← add new scenarios here
├── state.rs                   — AppState, load/persist state to disk
├── core/
│   ├── mod.rs                 — module index + re-exports
│   ├── scenario_template.rs   — Scenario trait, ActivationParams, IndexEffect,
│   │                            page_html, control_html, shared CSS/JS consts
│   └── registry.rs            — ScenarioRegistry (name → factory lookup)
├── scenario/
│   ├── mod.rs                 — pub mod declarations + re-exports from core
│   └── <name>.rs              — one file per scenario; see docs/scenarios.md for the full list
└── routes/
    ├── mod.rs
    ├── index.rs               — GET /  → active scenario's on_index_request()
    ├── health.rs              — GET /health (sim), GET /rotectl/health (control), GET /rotectl/sim-health (JSON status)
    ├── logo.rs                — GET /logo.png, GET /favicon.png — compiled-in static assets
    └── rotectl/
        ├── cmd.rs             — POST /rotectl/cmd — activates via registry
        ├── control.rs         — GET /rotectl/control — browser control panel
        └── status.rs          — GET /rotectl/status
```

## How it works

**Adding a scenario** requires touching three source locations — a new file in
`scenario/`, one line in `catalog.rs`, and one line in `scenario/mod.rs` — plus
a hurl test and a `scenarios.md` entry. Everything else (routing, persistence,
status) is wired up automatically.

### `core/scenario_template.rs` — the full contract

One file defines everything a scenario works with:

- `ActivationParams` — generic `HashMap<String, Value>` wrapper; scenarios read
  keys they care about, extra keys are ignored. `ActivationParams::from_json(json!({...}))` is
  the idiomatic way to declare default params in a scenario.
- `Scenario` trait — five required methods (`name`, `description`, `activate`,
  `deactivate`, `on_index_request`) plus three optional ones with defaults
  (`on_resume`, `status_extras`, `default_params`).
- `default_params()` — the single addition needed to make a scenario's parameters
  appear in the control panel. Return an `ActivationParams` built from a JSON literal;
  number values → `<input type="number">`, string values → `<input type="text">`. No HTML
  or JS changes required. Default: no parameters.
- `IndexEffect` — what `on_index_request` returns: `Respond(html)`, `Exit` (in `sim` mode: calls `process::exit(1)` so Kubernetes applies real CrashLoopBackOff; in `full` mode: sets crash flag → health probe fails), `Hang` (hold connection open), or `RespondWithStatus(status, html)`.
- `page_html(scenario, body, control_url, sim_url)` — renders the service status page served at `GET /`. Control link is omitted when `control_url` is empty (i.e. in `sim` mode).
- `control_html(active, active_desc, active_params, scenarios, sim_status, control_url, sim_url)` — renders the browser control panel at `GET /rotectl/control`. Scenario list is a searchable combobox. Parameter inputs are generated from `active_params` automatically.
- `SHARED_CSS` / `CONTROL_CSS` / `CONTROL_JS` — CSS and JS shared across both pages, defined as `const` strings here. `CONTROL_JS` reads `data-param-name` / `data-param-type` attributes off inputs so it collects parameters generically without knowing scenario names.
- `ScenarioMeta` — internal snapshot `{name, description}` used only by `registry.list()` and `control_html`; not part of the scenario-authoring API.

### `catalog.rs` — the scenario registry

A single `Vec` of factory closures. The registry calls each factory once at
startup to read `Scenario::name()`; later calls produce fresh instances.

### Request flow

```
POST /rotectl/cmd  {"cmd":"set","scenario":"oom-kill","loop_time_secs":10}
  → cmd.rs         parses extra fields into ActivationParams
  → registry       create("oom-kill") → fresh OomKillScenario
  → AppState       old.deactivate(); new.activate(&params); persist to disk

GET /
  → index.rs       active_scenario.on_index_request()
                   → IndexEffect::Respond(html)  or  IndexEffect::Exit
```

### State persistence

After every scenario switch, `AppState` writes to `/data/state.json`:

```json
{"scenario": "oom-kill", "loop_time_secs": 10, "loop_amount_mb": 15}
```

On pod restart, `load_state` reads the file, the registry recreates the
scenario, and `on_resume` restarts any background tasks.

### Control plane crash isolation

The control plane is isolated from the simulation surface in two complementary ways:

**`IndexEffect::Exit` (crash-loop, missing-env-var):**
The process does **not** die. Instead a `crashed` flag is set in `AppState`. `GET /health`
returns 503, which Kubernetes treats as a probe failure and restarts the pod. The control
plane stays accessible throughout. The control panel shows an amber "Simulation crashed"
badge; the JS polls `GET /rotectl/sim-health` every 2 seconds to keep the badge current.

**`OOMKill` (oom-kill scenario):**
The OS sends `SIGKILL` to the sim container — no code can survive this. The sidecar
architecture handles it: the `rotelle-control` container has a 256 Mi memory limit so
the oom-kill scenario (which targets the 64 Mi sim container) never affects it. When the
sim container is restarted by Kubernetes, it reads `state.json` from the shared PVC and
resumes in under two seconds. The control panel shows a red "Simulation down — restarting"
badge while the sim is not running.

**State synchronisation:**
The control sidecar is authoritative for `state.json`. When a scenario is switched via
`POST /rotectl/cmd`, the control container writes `state.json` immediately and returns
success. The sim container polls `state.json` every 2 seconds and applies any change
to its in-memory state without restarting.

**Sim health file:**
The sim container writes `/data/sim_health` (on the shared PVC) every 2 seconds with
the value `ok` or `crashed`. `GET /rotectl/sim-health` reads this file; a stale or
missing file (> 30 s old) means the sim is `down` (OOMKilled or otherwise not running).

## Adding a new scenario

See [CONTRIBUTING.md](../CONTRIBUTING.md).