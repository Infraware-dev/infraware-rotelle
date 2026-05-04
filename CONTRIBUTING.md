# Contributing a New Failure Scenario

Adding a scenario requires touching **four places**:

1. A new file in `rotelle/src/scenario/`
2. One line in `rotelle/src/scenario/catalog.rs`
3. A hurl test in `tests/hurl-case/`
4. An entry to `docs/rotelle-cases.md`

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
    let n = params.get_u64("my_value").unwrap_or(10);
}
```

Available helpers: `get_u64`, `get_usize`. Adding a new parameter requires no
changes to any shared struct — `ActivationParams` is a plain JSON object.

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
fn status_extras(&self) -> serde_json::Value {
    serde_json::json!({ "my_key": 42 })
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

## Running the control-path test suite

```sh
just run        # terminal 1
just test-hurl  # terminal 2 — runs check.hurl
```

---

## Existing scenarios (reference implementations)

| File | Name | Complexity |
|---|---|---|
| `idle.rs` | `none, idle` | stateless — simplest possible |
| `check.rs` | `check` | stateless — simplest possible |
| `intermittent_01.rs` | `intermittent-01` | shared counter with Mutex |
| `intermittent_02.rs` | `intermittent-02` | background task + memory allocation |
