# Contributing to Rotelle

Thank you for contributing! This guide covers everything from forking the repo
to opening a pull request.

---

## Prerequisites

Make sure the following are installed before you start:

| Tool | Purpose | Install |
|---|---|---|
| Rust (stable) | Build the binary | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| musl target | Cross-compile for the container | `rustup target add x86_64-unknown-linux-musl` (or `aarch64-unknown-linux-musl` on Apple Silicon) |
| [just](https://github.com/casey/just) | Run project tasks | `cargo install just` |
| [hurl](https://hurl.dev/docs/installation.html) | Run HTTP test sequences | see hurl docs |
| Docker | Build the container image | see docker docs |

kubectl and a local cluster (e.g. kind) are only needed if you want to test
against a real Kubernetes deployment; `just run` runs the binary directly
without a cluster.

---

## Getting started

**1. Fork the repository** on GitHub, then clone your fork:

```sh
git clone https://github.com/<your-username>/infraware-rotelle.git
cd infraware-rotelle
```

**2. Add the upstream remote** so you can pull future changes:

```sh
git remote add upstream https://github.com/infraware-dev/infraware-rotelle.git
```

**3. Create a branch** off `main`:

```sh
git checkout -b scenario/my-scenario   # new scenario
# or: fix/short-description            # bug fix
# or: docs/short-description           # documentation only
```

---

## Contributing a New Failure Scenario

Adding a scenario requires touching **five places**:

1. A new file in `rotelle/src/scenario/`
2. One line in `rotelle/src/scenario/catalog.rs` + one line in `rotelle/src/scenario/mod.rs`
3. A hurl test in `tests/hurl-case/`
4. An entry in `docs/rotelle-cases.md`

---

## Step 1 — Write the scenario

Create `rotelle/src/scenario/<name>.rs`. Copy `idle.rs` or `check.rs` as a
starting point for stateless scenarios, or `intermittent_01.rs` for scenarios
that need a counter, or `intermittent_02.rs` for scenarios with a background task.

```rust
// rotelle/src/scenario/my_scenario.rs

use super::{ActivationParams, IndexEffect, Scenario, page_html};

pub struct MyScenario;

impl MyScenario {
    pub fn new() -> Self { Self }
}

impl Scenario for MyScenario {
    fn name(&self) -> &'static str { "my-scenario" }

    fn description(&self) -> &'static str {
        "One sentence: what failure does this simulate."
    }

    fn activate(&self, _params: &ActivationParams) {
        // start background tasks, reset counters, etc.
    }

    fn deactivate(&self) {
        // abort tasks, release resources
    }

    fn on_index_request(&self) -> IndexEffect {
        IndexEffect::Respond(page_html("my-scenario", "<p>extra content here</p>"))
        // or: IndexEffect::Exit(1)  ← crashes the process; k8s restarts the pod
    }
}
```

### Reading activation parameters

Parameters arrive as flat JSON fields in the API request:

```json
{"cmd": "set", "case": "my-scenario", "my_value": 42}
```

Read them in `activate` — always supply a default:

```rust
fn activate(&self, params: &ActivationParams) {
    let n   = params.get_u64("my_value").unwrap_or(10);       // u64  — durations, counters
    let sz  = params.get_usize("my_bytes").unwrap_or(64);     // usize — buffer/size values
    let key = params.get_string("my_key")
                    .unwrap_or_else(|| "DEFAULT".to_string()); // String — names, identifiers
}
```

Available helpers:

| Helper | Return type | Use for |
|---|---|---|
| `get_u64(key)` | `Option<u64>` | durations, counters |
| `get_usize(key)` | `Option<usize>` | buffer sizes, byte counts |
| `get_string(key)` | `Option<String>` | named config values, env var names |

Adding a new parameter requires no changes to any shared struct — `ActivationParams` is a plain JSON object.

### Storing state between requests

Use `std::sync::Mutex` for any mutable state the scenario holds across requests.
Do **not** use `tokio::sync::Mutex` — scenario methods are synchronous.

```rust
pub struct MyScenario {
    counter: std::sync::Mutex<u32>,
}
```

For background tasks, spawn with `tokio::spawn` inside `activate` and store
the `AbortHandle` so `deactivate` can cancel it. See `intermittent_02.rs`.

### Optional overrides

```rust
// Called on pod restart if this scenario was previously persisted.
// Default: calls activate(params). Override only if resume must differ.
fn on_resume(&self, params: &ActivationParams) { self.activate(params); }

// Extra fields merged into GET /rotectl/status. Default: nothing.
// Use params.to_json() to echo all active params back into the status response.
fn status_extras(&self) -> serde_json::Value {
    serde_json::json!({ "my_key": 42 })
    // or: params.to_json()  ← echoes the full ActivationParams map
}
```

---

## Step 2 — Register in the catalog

Open `rotelle/src/scenario/catalog.rs` and add one line:

```rust
Box::new(|| Arc::new(MyScenario::new())),
```

Also expose the module in `rotelle/src/scenario/mod.rs`:

```rust
pub mod my_scenario;
```

The scenario name is read automatically from `Scenario::name()`.

---

## Step 3 — Write a hurl test

Create `tests/hurl-case/my-scenario.hurl`:

```hurl
# my-scenario.hurl
#
# Usage:
#   hurl --variable rotelle_service_host=<URL> tests/hurl-case/my-scenario.hurl
#   just run-case my-scenario

POST {{rotelle_service_host}}/rotectl/cmd
Content-Type: application/json
{"cmd": "set", "case": "my-scenario"}

HTTP 200
[Asserts]
jsonpath "$.ok" == true
jsonpath "$.failure_case" == "my-scenario"

GET {{rotelle_service_host}}/rotectl/status
HTTP 200
[Asserts]
jsonpath "$.failure_case" == "my-scenario"

GET {{rotelle_service_host}}/
HTTP 200
[Asserts]
body contains "my-scenario"
```

Run it locally:

```sh
just run                      # terminal 1 — start server on :8080
just run-case my-scenario     # terminal 2
```

---

## Step 4 — Document in rotelle-cases.md

Add an entry to `docs/rotelle-cases.md` under `## Implemented`. Use an existing
entry as a template. Include:

- **API case name** and **Source** path
- **What it simulates** — one paragraph describing the real K8s failure
- **Parameters** table (or "none")
- **Activation example** JSON snippet
- **`/rotectl/status` extras** JSON snippet (if `status_extras` is overridden)
- **Diagnosis value** — what an operator learns from observing this scenario
- **Hurl test** path

---

## Running the control-path test suite

```sh
just run        # terminal 1
just test-hurl  # terminal 2 — runs check.hurl
```

---

## Opening a pull request

Before opening the PR, verify locally:

- [ ] `cargo build --release` compiles without errors or warnings
- [ ] `just run` starts the server cleanly
- [ ] `just run-case <your-scenario>` passes
- [ ] `just run-case check` still passes (regression check)
- [ ] `docs/rotelle-cases.md` has your entry

When you open the PR:

1. **Target `main`** as the base branch.
2. **Title** — use the scenario name, e.g. `Add scenario: my-scenario`.
3. **Description** — include:
   - What Kubernetes failure the scenario simulates and why it's useful
   - How to manually verify it (`just run` + `just run-case …`)
   - Any design decisions worth noting (e.g. why you chose a particular `IndexEffect`)
4. Keep the PR focused — one scenario per PR.

All changes — including from maintainers — go through a pull request. Direct
pushes to `main` are blocked. A maintainer will review your scenario
implementation, hurl test, and `rotelle-cases.md` entry before merging.

---

## Existing scenarios (reference implementations)

| File | Name | Complexity |
|---|---|---|
| `idle.rs` | `none, idle` | stateless — simplest possible |
| `check.rs` | `check` | stateless — simplest possible |
| `intermittent_01.rs` | `intermittent-01` | shared counter with Mutex |
| `intermittent_02.rs` | `intermittent-02` | background task + memory allocation |
| `missing_env_var.rs` | `missing-env-var` | delayed exit task + `on_resume` override |
| `service_unreachable.rs` | `service-unreachable` | `IndexEffect::Hang` — async connection hold |
| `ingress_conflict.rs` | `ingress-conflict` | counter + `IndexEffect::RespondWithStatus` |
