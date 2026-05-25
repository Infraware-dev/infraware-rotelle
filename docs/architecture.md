# Architecture

## Overview

Rotelle is a single Rust binary that runs as a Kubernetes deployment. It exposes
an HTTP server (via [Poem](https://github.com/poem-web/poem)) with two surfaces:

- **Simulation endpoints** (`/`, `/health`) — behave as the failing application.
- **Control endpoints** (`/rotectl/*`) — activate, inspect, and reset scenarios.

## Module layout

```
src/
├── main.rs                    — startup: build registry, load state, start server
├── state.rs                   — AppState, load/persist state to disk
├── core/
│   ├── mod.rs                 — module index + re-exports
│   ├── scenario_template.rs   — Scenario trait, ActivationParams, IndexEffect, page_html
│   └── registry.rs            — ScenarioRegistry (name → factory lookup)
├── scenario/
│   ├── mod.rs                 — pub mod declarations + re-exports from core
│   ├── catalog.rs             — list of all scenario factories  ← add new scenarios here
│   └── <name>.rs              — one file per scenario; see docs/scenarios.md for the full list
└── routes/
    ├── mod.rs
    ├── index.rs               — GET /  → active scenario's on_index_request()
    ├── health.rs              — GET /health
    ├── logo.rs                — GET /logo.png, GET /favicon.png — compiled-in static assets
    └── rotectl/
        ├── cmd.rs             — POST /rotectl/cmd — activates via registry
        ├── status.rs          — GET /rotectl/status
        └── health.rs          — GET /rotectl/health
```

## How it works

**Adding a scenario** requires touching three source locations — a new file, one
line in `catalog.rs`, and one line in `mod.rs` — plus a hurl test and a
`scenarios.md` entry. Everything else (routing, persistence, status) is
wired up automatically.

### `core/scenario_template.rs` — the full contract

One file defines everything a scenario works with:

- `ActivationParams` — generic `HashMap<String, Value>` wrapper; scenarios read
  keys they care about, extra keys are ignored.
- `Scenario` trait — five required methods (`name`, `description`, `activate`,
  `deactivate`, `on_index_request`) plus two optional ones with defaults.
- `IndexEffect` — what `on_index_request` returns: `Respond(html)`, `Exit(code)`, `Hang` (hold connection open), or `RespondWithStatus(status, html)`.
- `page_html(scenario, description, body)` — helper that renders the standard index page; `description` comes from `Scenario::description()` and is shown below the scenario name.

### `catalog.rs` — the scenario list

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
                   → IndexEffect::Respond(html)  or  IndexEffect::Exit(code)
```

### State persistence

After every scenario switch, `AppState` writes to `/data/state.json`:

```json
{"scenario": "oom-kill", "params": {"loop_time_secs": 10, "loop_amount_mb": 15}}
```

On pod restart, `load_state` reads the file, the registry recreates the
scenario, and `on_resume` restarts any background tasks.

## Adding a new scenario

See [CONTRIBUTING.md](../CONTRIBUTING.md).