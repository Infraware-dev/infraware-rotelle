# Architecture

## Overview

Rotelle is a single Rust binary that runs as a Kubernetes deployment. The same binary
is started in different modes via the `ROTELLE_MODE` environment variable:

| Mode | Port | Handles | Purpose |
|---|---|---|---|
| `full` (default) | 8080 | everything | Single-process mode for local development |
| `sim` | 8080 | `GET /`, `GET /health` | Simulation surface only (legacy; K8s deployments use `full`) |
| `control` | 9090 | `GET /rotectl/*`, `POST /rotectl/cmd` | Standalone control pod — proxies to sim over HTTP |

In Kubernetes, the recommended setup is two separate pods in two separate namespaces:

- **`rotelle` namespace** — sim pod running in `full` mode (serves both the simulation surface
  and its own `/rotectl/*` API). Intentionally crashable.
- **`rotelle-system` namespace** — control pod running in `control` mode. Probes the sim pod's
  API over HTTP. Stays alive regardless of what happens to the sim pod.

See `k8s/rotelle.yaml` (sim) and `k8s/rotelle-control.yaml` (control pod).

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
    ├── health.rs              — GET /health (sim), GET /rotectl/health (control), GET /rotectl/sim-health (JSON)
    ├── logo.rs                — GET /logo.png, GET /favicon.png — compiled-in static assets
    └── rotectl/
        ├── cmd.rs             — POST /rotectl/cmd — activates locally (full/sim) or proxies to sim (control)
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
- `page_html(scenario, body, control_url, sim_url)` — renders the service status page served at `GET /`.
- `control_html(active, active_desc, active_params, scenarios, sim_status, control_url, sim_url)` — renders the browser control panel at `GET /rotectl/control`. Scenario list is a searchable combobox. Parameter inputs are generated from `active_params` automatically.
- `SHARED_CSS` / `CONTROL_CSS` / `CONTROL_JS` — CSS and JS shared across both pages, defined as `const` strings here. `CONTROL_JS` reads `data-param-name` / `data-param-type` attributes off inputs so it collects parameters generically without knowing scenario names.
- `ScenarioMeta` — internal snapshot `{name, description}` used only by `registry.list()` and `control_html`; not part of the scenario-authoring API.

### `catalog.rs` — the scenario registry

A single `Vec` of factory closures. The registry calls each factory once at
startup to read `Scenario::name()`; later calls produce fresh instances.

### Request flow — full/sim mode

```
POST /rotectl/cmd  {"cmd":"set","scenario":"oom-kill","loop_time_secs":10}
  → cmd.rs         parses JSON into ActivationParams
  → registry       create("oom-kill") → fresh OomKillScenario
  → AppState       old.deactivate(); new.activate(&params); persist to state.json

GET /
  → index.rs       active_scenario.on_index_request()
                   → IndexEffect::Respond(html)  or  IndexEffect::Exit
```

### Request flow — control mode

```
POST /rotectl/cmd  → proxied via HTTP to sim pod's POST /rotectl/cmd
GET  /rotectl/status  → proxied via HTTP to sim pod's GET /rotectl/status
GET  /rotectl/sim-health  → probes sim pod's GET /health
                             200 → {"status":"ok"}
                             503 → {"status":"crashed"}
                             error/timeout → {"status":"down"}
GET  /rotectl/control  → fetches sim's /rotectl/status, renders HTML with local registry list
```

The control pod is configured via `ROTELLE_SIM_API_URL` (e.g.
`http://rotelle.rotelle.svc.cluster.local:8080`). All communication is plain HTTP
within the cluster — no shared volume required.

### State persistence

After every scenario switch, `AppState` writes to `state.json` in `DATA_DIR`:

```json
{"scenario": "oom-kill", "params": {"loop_time_secs": 10, "loop_amount_mb": 15}}
```

On pod restart, `load_state` reads the file, the registry recreates the
scenario, and `on_resume` restarts any background tasks.

### Control plane crash isolation

The sim pod and control pod are separate Kubernetes pods in separate namespaces.
The control pod is unaffected by anything that happens to the sim pod — including
OOMKill, CrashLoopBackOff or any other crash. It detects sim health by probing
`GET /health` on the sim pod every 2 seconds via `GET /rotectl/sim-health` on
the control pod. The control panel badge shows `ok`, `crashed` (503) or `down`
(connection refused / timeout).

## Adding a new scenario

See [CONTRIBUTING.md](../CONTRIBUTING.md).
