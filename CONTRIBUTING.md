# Contributing to Rotelle

Welcome, and thanks for considering a contribution! Rotelle is a small project and every improvement matters — whether that's a new failure scenario, a bug fix, a documentation correction, or a better test.

By participating you agree to abide by the [Code of Conduct](CODE_OF_CONDUCT.md). For security issues, use the [responsible disclosure process](SECURITY.md) instead of opening a public issue.

**Not sure where to start?** See [docs/good-first-issues.md](docs/good-first-issues.md) for a curated list of well-scoped scenarios ready to implement, each with a clear definition of done.

---

## Table of Contents

1. [Ways to contribute](#ways-to-contribute)
2. [Prerequisites](#prerequisites)
3. [Development setup](#development-setup)
4. [Project structure](#project-structure)
5. [Example: adding a failure scenario](#example-adding-a-failure-scenario)
6. [Contribution workflow](#contribution-workflow)
7. [Opening a pull request](#opening-a-pull-request)
8. [The review process](#the-review-process)
9. [Reference: existing scenarios](#reference-existing-scenarios)

---

## Ways to contribute

| Type | How |
|---|---|
| **New failure scenario** | The primary contribution path. See the [example below](#example-adding-a-failure-scenario). |
| **Bug fix** | Open a [bug report](https://github.com/infraware-dev/infraware-rotelle/issues/new?template=bug_report.yml) first so we can discuss it, then send a PR. |
| **Documentation** | Fix anything unclear in `README.md`, `CONTRIBUTING.md`, `docs/rotelle-cases.md`, etc. directly via PR — no issue needed. |
| **Tooling / CI** | Improvements to `justfile`, scripts, or `.github/workflows/` are welcome. Discuss significant changes in an issue first. |
| **New scenario ideas** | Open a [feature request](https://github.com/infraware-dev/infraware-rotelle/issues/new?template=feature_request.yml) — we'll give feedback before you write any code. |

---

## Prerequisites

Install these before starting. Everything else can be learned as you go.

| Tool | Purpose | Install |
|---|---|---|
| Rust (stable) | Build the binary | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| musl target | Cross-compile for the container | `rustup target add x86_64-unknown-linux-musl` (or `aarch64-unknown-linux-musl` on Apple Silicon) |
| [just](https://github.com/casey/just) | Run project tasks | `cargo install just` |
| [hurl](https://hurl.dev/docs/installation.html) | Run HTTP test sequences | see hurl docs |
| Docker | Build the container image | see docker docs |

`kubectl` and a local cluster (e.g. kind) are only needed to test against a real Kubernetes deployment. `just run` runs the binary directly without any cluster.

---

## Development setup

Follow these steps exactly once when you set up a new machine or fork.

**Step 1 — Fork the repository on GitHub.**

Click **Fork** on the [repository page](https://github.com/infraware-dev/infraware-rotelle). This creates your own copy under your GitHub account.

**Step 2 — Clone your fork locally.**

```sh
git clone https://github.com/<your-username>/infraware-rotelle.git
cd infraware-rotelle
```

**Step 3 — Add the upstream remote.**

This lets you pull future changes from the main repo into your fork.

```sh
git remote add upstream https://github.com/infraware-dev/infraware-rotelle.git
```

Verify both remotes are set up:

```sh
git remote -v
# origin    https://github.com/<your-username>/infraware-rotelle.git (fetch)
# origin    https://github.com/<your-username>/infraware-rotelle.git (push)
# upstream  https://github.com/infraware-dev/infraware-rotelle.git (fetch)
# upstream  https://github.com/infraware-dev/infraware-rotelle.git (push)
```

**Step 4 — Verify the build.**

```sh
cd rotelle
cargo build
```

The first build downloads dependencies and takes a minute or two. Subsequent builds are fast.

**Step 5 — Run the server locally.**

```sh
just run
```

You should see output like `Listening on 0.0.0.0:8080`. In a second terminal, verify it responds:

```sh
curl http://localhost:8080/rotectl/status
# {"failure_case":"none","params":{}}
```

**Step 6 — Run the test suite.**

Keep `just run` running in terminal 1 and run the following in terminal 2:

```sh
just test-hurl
# Runs tests/hurl-case/check.hurl — should pass with no errors
```

If everything passes, your setup is complete.

**Step 7 — Create a branch for your work.**

Always branch off `main`. Never work directly on `main`.

```sh
git checkout -b scenario/my-scenario   # new scenario
# or
git checkout -b fix/short-description  # bug fix
# or
git checkout -b docs/short-description # docs only
```

---

## Project structure

```
rotelle/src/
├── main.rs                        — startup: registry, state load, server boot
├── state.rs                       — AppState, load/persist to /data/state.json
├── core/
│   ├── scenario_template.rs       — Scenario trait, ActivationParams, IndexEffect  ← read this first
│   └── registry.rs                — name → scenario factory lookup
├── scenario/
│   ├── catalog.rs                 — list of all scenario factories  ← add new scenarios here
│   ├── mod.rs                     — pub mod declarations            ← and here
│   └── *.rs                       — individual scenario implementations
└── routes/
    ├── index.rs                   — GET /  → active scenario's on_index_request()
    └── rotectl/                   — control API handlers
```

Read `core/scenario_template.rs` first — it defines the full `Scenario` trait contract. Everything else follows from it.

Full architecture reference: [docs/architecture.md](docs/architecture.md)

---

## Example: adding a failure scenario

Adding a scenario is the main contribution path. It touches **four steps** (five files total).

### Step 1 — Write the scenario

Create `rotelle/src/scenario/my_scenario.rs`. Choose a starting point based on complexity:

| Reference file | Use when |
|---|---|
| `idle.rs` or `check.rs` | Stateless — no mutable state, no background tasks |
| `intermittent_01.rs` | Needs a counter (shared via `std::sync::Mutex`) |
| `intermittent_02.rs` | Needs a background task (tokio::spawn + AbortHandle) |
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

    fn activate(&self, _params: &ActivationParams) {
        // start background tasks, reset counters, etc.
    }

    fn deactivate(&self) {
        // cancel background tasks, release resources
    }

    fn on_index_request(&self) -> IndexEffect {
        IndexEffect::Respond("<p>my scenario is active</p>".to_string())
        // or: IndexEffect::Exit(1)               — crashes the pod; k8s restarts it
        // or: IndexEffect::Hang                  — holds connection open indefinitely
        // or: IndexEffect::RespondWithStatus(503, html) — returns an error status
    }
}
```

**Reading activation parameters** — parameters arrive as extra JSON fields in the API request:

```json
{"cmd": "set", "case": "my-scenario", "my_value": 42}
```

Read them in `activate`, always with a default:

```rust
fn activate(&self, params: &ActivationParams) {
    let n   = params.get_u64("my_value").unwrap_or(10);
    let sz  = params.get_usize("my_bytes").unwrap_or(64);
    let key = params.get_string("my_key").unwrap_or_else(|| "DEFAULT".to_string());
}
```

| Helper | Return type | Use for |
|---|---|---|
| `get_u64(key)` | `Option<u64>` | durations, counters |
| `get_usize(key)` | `Option<usize>` | buffer sizes, byte counts |
| `get_string(key)` | `Option<String>` | names, identifiers, paths |

**Storing state between requests** — use `std::sync::Mutex` (not `tokio::sync::Mutex` — scenario methods are synchronous):

```rust
pub struct MyScenario {
    counter: std::sync::Mutex<u32>,
}
```

**Optional trait methods:**

```rust
// Override if pod-restart behavior must differ from initial activation.
// Default: calls activate(params).
fn on_resume(&self, params: &ActivationParams) { self.activate(params); }

// Extra fields merged into GET /rotectl/status.
fn status_extras(&self) -> serde_json::Value {
    serde_json::json!({ "my_counter": 3 })
}
```

---

### Step 2 — Register in the catalog

Open `rotelle/src/scenario/catalog.rs` and add one line to the factory list:

```rust
Box::new(|| Arc::new(MyScenario::new())),
```

Open `rotelle/src/scenario/mod.rs` and expose the module:

```rust
pub mod my_scenario;
```

The scenario name is discovered automatically from `Scenario::name()` — no other wiring needed.

---

### Step 3 — Write a hurl test

Create `tests/hurl-case/my-scenario.hurl`:

```hurl
# Activate the scenario
POST {{rotelle_service_host}}/rotectl/cmd
Content-Type: application/json
{"cmd": "set", "case": "my-scenario"}

HTTP 200
[Asserts]
jsonpath "$.ok" == true
jsonpath "$.failure_case" == "my-scenario"

# Verify status reflects the active scenario
GET {{rotelle_service_host}}/rotectl/status
HTTP 200
[Asserts]
jsonpath "$.failure_case" == "my-scenario"

# Verify the simulation endpoint behaves as expected
GET {{rotelle_service_host}}/
HTTP 200
[Asserts]
body contains "my-scenario"
```

Run it locally:

```sh
# Terminal 1
just run

# Terminal 2
just run-case my-scenario
```

Also run the regression check to make sure you haven't broken anything:

```sh
just run-case check
```

---

### Step 4 — Document in rotelle-cases.md

Add an entry to `docs/rotelle-cases.md` under `## Implemented`. Use any existing entry as a template. Include:

- **API case name** and **Source** file path
- **What it simulates** — one paragraph describing the real Kubernetes failure
- **Parameters** table (or "none")
- **Activation example** — a JSON snippet
- **`/rotectl/status` extras** — JSON snippet (if you override `status_extras`)
- **Diagnosis value** — what an SRE or AI system learns by observing this scenario
- **Hurl test** path

---

## Contribution workflow

These rules are enforced by CI and branch protection — not just guidelines.

### Commit message format

Rotelle uses **[Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/)**. Every commit subject must follow:

```
<type>(<scope>): <short description in imperative mood>
```

| Type | When to use |
|---|---|
| `feat` | New scenario or control API feature |
| `fix` | Bug fix in an existing scenario or route |
| `docs` | Documentation only |
| `test` | Adding or fixing hurl tests |
| `ci` | Changes to `.github/workflows/` |
| `chore` | Build scripts, `justfile`, dependencies |
| `refactor` | Code restructure with no behavior change |

Real examples:

```
feat(scenario/readiness-fail): add readiness probe failure simulation
fix(scenario/intermittent-01): reset counter on deactivate
docs(rotelle-cases): add ingress-conflict diagnosis notes
ci: add cargo deny dependency audit step
chore: upgrade poem to 3.1
```

Rules:
- Imperative mood: "add crash-loop scenario" not "added crash-loop scenario"
- Subject line under **72 characters**, no trailing period
- Separate subject from body with a blank line if you need more detail

### DCO sign-off

All commits must include a **Developer Certificate of Origin** sign-off — a lightweight statement that you have the right to submit the code under this project's license (Apache 2.0).

Add it with the `-s` flag:

```sh
git commit -s -m "feat(scenario/readiness-fail): add readiness probe failure simulation"
```

This appends `Signed-off-by: Your Name <your@email.com>` to the commit. Your name and email must match your Git config:

```sh
git config user.name   # must be set
git config user.email  # must be set
```

**CI will reject commits without a sign-off.** If you forgot it on past commits in your branch, fix with:

```sh
git rebase --signoff HEAD~<number-of-commits>
```

### Branch protection

**Direct pushes to `main` are blocked** — for everyone, including maintainers. The merge button is locked until:

- All CI jobs pass (lint, test, build, DCO check)
- At least **1 maintainer approval**
- All review comments are resolved

Stale approvals are dismissed automatically when you push new commits.

---

## Opening a pull request

**Before you open the PR**, run this sequence locally:

```sh
cargo fmt --all                              # auto-fix formatting
cargo clippy --all-targets -- -D warnings   # must be clean
cargo test                                  # must pass
just run        # terminal 1
just run-case <your-scenario>               # terminal 2 — must pass
just run-case check                         # regression — must pass
```

**Opening the PR:**

1. Push your branch to your fork: `git push origin scenario/my-scenario`
2. Go to the repository on GitHub — you'll see a prompt to open a PR.
3. GitHub will pre-fill the description from the PR template. Fill in all sections — the template explains what each field is for.
4. **Title** must follow Conventional Commits: `feat(scenario/my-scenario): add readiness probe failure simulation`
5. **Target `main`** as the base branch.
6. Keep the PR focused — one scenario or one fix per PR.

**Keeping your branch up to date:**

If `main` moved while you were working, rebase before opening (or if asked during review):

```sh
git fetch upstream
git rebase upstream/main
```

Avoid `git merge upstream/main` — rebasing keeps history linear.

---

## The review process

1. **CI runs automatically** as soon as you open the PR. Fix any failures before asking for a review — a failing CI is a signal the PR isn't ready.
2. **A maintainer will review within a few days.** If a week passes with no response, leave a comment to bump it.
3. **Address feedback with new commits** — do not force-push during an active review, as it makes it hard to see what changed between rounds.
4. Once approved and CI is green, a maintainer **squash-merges** your PR. The final commit message will follow the Conventional Commits format based on your PR title.

That's it — you're a contributor.

---

## Reference: existing scenarios

| File | Scenario name | Complexity |
|---|---|---|
| `idle.rs` | `none, idle` | stateless — simplest possible |
| `check.rs` | `check` | stateless — simplest possible |
| `intermittent_01.rs` | `intermittent-01` | counter with `std::sync::Mutex` |
| `intermittent_02.rs` | `intermittent-02` | background task + `AbortHandle` |
| `missing_env_var.rs` | `missing-env-var` | delayed exit + `on_resume` override |
| `service_unreachable.rs` | `service-unreachable` | `IndexEffect::Hang` |
| `ingress_conflict.rs` | `ingress-conflict` | counter + `IndexEffect::RespondWithStatus` |
