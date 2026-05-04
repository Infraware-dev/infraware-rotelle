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
├── scenario/
│   ├── mod.rs                 — pub mod declarations + re-exports
│   ├── core.rs                — Scenario trait, ActivationParams, IndexEffect, page_html
│   ├── catalog.rs             — list of all scenario factories  ← add new scenarios here
│   ├── registry.rs            — ScenarioRegistry (name → factory lookup)
│   ├── idle.rs                — "none, idle"
│   ├── check.rs               — "check"
│   ├── intermittent_01.rs     — "intermittent-01"
│   └── intermittent_02.rs     — "intermittent-02"
└── routes/
    ├── mod.rs
    ├── index.rs               — GET /  → active scenario's on_index_request()
    ├── health.rs              — GET /health
    └── rotectl/
        ├── cmd.rs             — POST /rotectl/cmd — activates via registry
        ├── status.rs          — GET /rotectl/status
        └── health.rs          — GET /rotectl/health
```

## How it works

**Adding a scenario** requires touching three source locations — a new file, one
line in `catalog.rs`, and one line in `mod.rs` — plus a hurl test and a
`rotelle-cases.md` entry. Everything else (routing, persistence, status) is
wired up automatically.

### `core.rs` — the full contract

One file defines everything a scenario works with:

- `ActivationParams` — generic `HashMap<String, Value>` wrapper; scenarios read
  keys they care about, extra keys are ignored.
- `Scenario` trait — five required methods (`name`, `description`, `activate`,
  `deactivate`, `on_index_request`) plus two optional ones with defaults.
- `IndexEffect` — what `on_index_request` returns: `Respond(html)` or `Exit(code)`.
- `page_html` — helper that renders the standard index page.

### `catalog.rs` — the scenario list

A single `Vec` of factory closures. The registry calls each factory once at
startup to read `Scenario::name()`; later calls produce fresh instances.

### Request flow

```
POST /rotectl/cmd  {"cmd":"set","case":"intermittent-02","loop_time_secs":10}
  → cmd.rs         parses extra fields into ActivationParams
  → registry       create("intermittent-02") → fresh Intermittent02Scenario
  → AppState       old.deactivate(); new.activate(&params); persist to disk

GET /
  → index.rs       active_scenario.on_index_request()
                   → IndexEffect::Respond(html)  or  IndexEffect::Exit(code)
```

### State persistence

After every scenario switch, `AppState` writes to `/data/state.json`:

```json
{"failure_case": "intermittent-02", "params": {"loop_time_secs": 10, "loop_amount_mb": 15}}
```

On pod restart, `load_state` reads the file, the registry recreates the
scenario, and `on_resume` restarts any background tasks.

## Adding a new scenario

See [CONTRIBUTING.md](../CONTRIBUTING.md). Short version:

1. Implement `Scenario` in `src/scenario/<name>.rs`.
2. Add `Box::new(|| Arc::new(MyScenario::new()))` to `catalog::all()`.
3. Expose the module in `mod.rs`.
4. Write a hurl test and a `docs/rotelle-cases.md` entry.
