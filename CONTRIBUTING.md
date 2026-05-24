# Contributing to Rotelle

Welcome, and thanks for considering a contribution! Every improvement matters — whether that's a new failure scenario, a bug fix, a documentation correction or a better test.

By participating you agree to abide by the [Code of Conduct](CODE_OF_CONDUCT.md). For security issues, contact the maintainers directly instead of opening a public issue.

**Not sure where to start?** Check out already existing issues in the GitHub repo.

---

## Ways to contribute

| Type | Process |
|---|---|
| **New failure scenario** | Open an [issue](https://github.com/infraware-dev/infraware-rotelle/issues/new?template=feature_request.yml) first so we can confirm scope, then send a PR. |
| **Bug fix** | For obvious bugs, send a PR directly. For anything non-trivial, open a [bug report](https://github.com/infraware-dev/infraware-rotelle/issues/new?template=bug_report.yml) first. |
| **Documentation** | Send a PR directly — no issue needed. |
| **Tooling / CI** | Discuss significant changes in an issue first; small improvements can go straight to a PR. |

---

## Prerequisites

| Tool | Purpose | Install |
|---|---|---|
| Rust (stable) | Build and run locally | see Rust docs |
| [just](https://github.com/casey/just) | Run project tasks | `cargo install just` |
| [hurl](https://hurl.dev/docs/installation.html) | Run HTTP test sequences | see hurl docs |
| [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) | Dependency audit (CI check) | `cargo install cargo-deny` |

**For Docker builds only:** add the musl target — `rustup target add x86_64-unknown-linux-musl` (or `aarch64-unknown-linux-musl` on Apple Silicon).

`kubectl` and a local cluster (e.g. kind) are only needed to test against a real Kubernetes deployment. `just run` runs the binary directly without any cluster.

---

## Development setup

**1. Fork and clone.**

Click **Fork** on the [repository page](https://github.com/infraware-dev/infraware-rotelle), then:

```sh
git clone https://github.com/<your-username>/infraware-rotelle.git
cd infraware-rotelle
git remote add upstream https://github.com/infraware-dev/infraware-rotelle.git
```

The `upstream` remote lets you pull future changes from the main repo into your fork.

**2. Verify the build.**

```sh
cd rotelle && cargo build
```

**3. Run the server locally.**

```sh
just run
# → Listening on 0.0.0.0:8080
```

Verify it responds:

```sh
curl http://localhost:8080/rotectl/status
# {"failure_case":"none","params":{}}
```

**4. Run the tests.**

Keep `just run` running in terminal 1, then in terminal 2:

```sh
just test-hurl   # should pass with no errors
```

**5. Create a branch.**

Always branch off `main`. Branch names follow the format `<type>/<short-description>`:

```sh
git checkout -b feat/my-scenario
git checkout -b fix/short-description
git checkout -b docs/short-description
```

Common types: `feat`, `fix`, `docs`, `test`, `chore`, `refactor`.

---

## Adding a failure scenario

Adding a scenario is the primary contribution path. It touches four steps.

### Step 1 — Write the scenario

Create `rotelle/src/scenario/my_scenario.rs`. Choose a reference file based on complexity:

| Reference file | Use when |
|---|---|
| `idle.rs` or `check.rs` | Stateless — no mutable state, no background tasks |
| `intermittent_01.rs` | Needs a counter (`std::sync::Mutex`) |
| `intermittent_02.rs` | Needs a background task (`tokio::spawn` + `AbortHandle`) |
| `missing_env_var.rs` | Needs `on_resume` override (behavior differs on pod restart) |

Copy the closest reference and adapt it:

```rust
use super::{ActivationParams, IndexEffect, Scenario};

pub struct MyScenario;

impl MyScenario {
    pub fn new() -> Self { Self }
}

impl Scenario for MyScenario {
    fn name(&self) -> &'static str { "my-scenario" }

    fn description(&self) -> &'static str {
        "One sentence: what Kubernetes failure does this simulate."
    }

    fn activate(&self, _params: &ActivationParams) {}

    fn deactivate(&self) {}

    fn on_index_request(&self) -> IndexEffect {
        IndexEffect::Respond("<p>my scenario is active</p>".to_string())
        // or: IndexEffect::Exit(1)                       — crashes the pod
        // or: IndexEffect::Hang                          — holds connection open
        // or: IndexEffect::RespondWithStatus(503, html)  — returns an error status
    }
}
```

**Reading activation parameters** — extra JSON fields from the API request:

```json
{"cmd": "set", "case": "my-scenario", "my_value": 42}
```

```rust
fn activate(&self, params: &ActivationParams) {
    let n   = params.get_u64("my_value").unwrap_or(10);
    let sz  = params.get_usize("my_bytes").unwrap_or(64);
    let key = params.get_string("my_key").unwrap_or_else(|| "DEFAULT".to_string());
}
```

**Storing state between requests** — use `std::sync::Mutex` (not `tokio::sync::Mutex` — scenario methods are synchronous):

```rust
pub struct MyScenario {
    counter: std::sync::Mutex<u32>,
}
```

**Optional trait methods:**

```rust
// Override if pod-restart behavior must differ from initial activation.
fn on_resume(&self, params: &ActivationParams) { self.activate(params); }

// Extra fields merged into GET /rotectl/status.
fn status_extras(&self) -> serde_json::Value {
    serde_json::json!({ "my_counter": 3 })
}
```

### Step 2 — Register in the catalog

In `rotelle/src/scenario/catalog.rs`, add one line:

```rust
Box::new(|| Arc::new(MyScenario::new())),
```

In `rotelle/src/scenario/mod.rs`, expose the module:

```rust
pub mod my_scenario;
```

### Step 3 — Write a hurl test

Create `tests/hurl-case/my-scenario.hurl`:

```hurl
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

POST {{rotelle_service_host}}/rotectl/cmd
Content-Type: application/json
{"cmd": "reset"}

HTTP 200
[Asserts]
jsonpath "$.ok" == true
jsonpath "$.failure_case" == "none, idle"
```

The only variable passed automatically is `rotelle_service_host`. Any other variables your test needs must be hardcoded directly in the hurl file — CI runs all hurl files automatically and passes no extra variables.

Run it locally (keep `just run` open in terminal 1):

```sh
just run-case my-scenario   # your scenario
just test-hurl              # regression — must still pass
```

### Step 4 — Document in rotelle-cases.md

Add an entry to `docs/rotelle-cases.md` under `## Implemented`. Use any existing entry as a template. Include:

- **API case name** and **Source** file path
- **What it simulates** — one paragraph
- **Parameters** table (or "none")
- **Activation example** — a JSON snippet
- **`/rotectl/status` extras** — if you override `status_extras`
- **Hurl test** path

---

## Project structure

See [docs/architecture.md](docs/architecture.md) for the full module layout. Read `core/scenario_template.rs` first — it defines the `Scenario` trait contract everything else follows from.

---

## Contribution workflow

### Commit message format

Rotelle uses **[Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/)**:

```
<type>: <short description in imperative mood>
```

| Type | When to use |
|---|---|
| `feat` | New scenario or control API feature |
| `fix` | Bug fix |
| `docs` | Documentation only |
| `test` | Adding or fixing hurl tests |
| `ci` | Changes to `.github/workflows/` |
| `chore` | Build scripts, `justfile`, dependencies |
| `refactor` | Code restructure with no behavior change |

Examples:

```
feat: add readiness probe failure simulation
fix: reset counter on deactivate
docs: add ingress-conflict diagnosis notes
ci: add cargo deny dependency audit step
```

Rules: imperative mood, subject under **72 characters**, no trailing period.

### Branch protection

Direct pushes to `main` are blocked. The merge button requires:

- All CI jobs pass (`lint`, `deny`, `test`, `build`, `hurl`)
- At least 1 maintainer approval
- All review comments resolved

---

## Opening a pull request

Run these from the **repo root** before opening the PR:

```sh
# code quality — must all pass
cargo fmt --all --manifest-path rotelle/Cargo.toml
cargo clippy --manifest-path rotelle/Cargo.toml --all-targets -- -D warnings
cargo deny --manifest-path rotelle/Cargo.toml check
cargo test --manifest-path rotelle/Cargo.toml

# integration tests — terminal 1: start server, terminal 2: run tests
just run
just test-hurl
just run-case <your-scenario>
```

Then:

1. Push your branch: `git push origin feat/my-scenario`
2. Open the PR on GitHub — the description template will guide you.
3. Title must follow Conventional Commits: `feat: add readiness probe failure simulation`
4. Target `main` as the base branch.
5. One scenario or fix per PR.

If `main` moved while you were working, rebase before opening:

```sh
git fetch upstream
git rebase upstream/main
```

---

## The review process

1. CI runs automatically on PR open — fix failures before requesting review.
2. A maintainer will review within a few days. If a week passes, leave a comment to bump it.
3. Address feedback with new commits — do not force-push during an active review.
4. Once approved and CI is green, a maintainer squash-merges the PR.

---

## For maintainers

The release process (versioning, tagging, publishing to GHCR) is documented in [RELEASING.md](RELEASING.md).

---

## Reference: existing scenarios

| File | Scenario name | Complexity |
|---|---|---|
| `idle.rs` | `none, idle` | stateless |
| `check.rs` | `check` | stateless |
| `intermittent_01.rs` | `intermittent-01` | counter with `std::sync::Mutex` |
| `intermittent_02.rs` | `intermittent-02` | background task + `AbortHandle` |
| `missing_env_var.rs` | `missing-env-var` | delayed exit + `on_resume` override |
| `service_unreachable.rs` | `service-unreachable` | `IndexEffect::Hang` |
| `ingress_conflict.rs` | `ingress-conflict` | counter + `IndexEffect::RespondWithStatus` |
