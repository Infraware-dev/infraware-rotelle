# Rotelle Cases

Each case is a named failure scenario activated via `POST /rotectl/cmd`.

## Implemented

---

### none, idle

**API case name:** `none, idle`  
**Source:** `src/scenario/idle.rs`

No failure active. The service responds normally to all requests. This is the
default state and the target of the `reset` command.

---

### check

**API case name:** `check`  
**Source:** `src/scenario/check.rs`

Control checkpoint. Marks a known, stable state that test sequences can assert
against. Activated by the `check` command (or `set` with `case: check`).

---

### intermittent-01

**API case name:** `intermittent-01`  
**Source:** `src/scenario/intermittent_01.rs`

**What it simulates:** Intermittent pod crash-loops.

The process calls `std::process::exit(1)` on every 5th `GET /`. Kubernetes
detects the non-zero exit code and restarts the pod. The counter resets on each
restart.

**Parameters:** none

**Activation example:**
```json
{ "cmd": "set", "case": "intermittent-01" }
```

**`/rotectl/status` extras:**
```json
{ "access_count": 3, "crash_every": 5 }
```

**Diagnosis value:**  
Identifies whether a service failure is caused by the pod process itself
(crash-loop) versus infrastructure or configuration. If the pod restarts on a
predictable cycle, the cause is in-process.

**Hurl test:** `tests/hurl-case/intermittent-01.hurl`

---

### intermittent-02

**API case name:** `intermittent-02`  
**Source:** `src/scenario/intermittent_02.rs`

**What it simulates:** Pod memory exhaustion leading to OOMKill.

A background task wakes every `loop_time_secs` seconds and allocates
`loop_amount_mb` MB of memory that is never freed. Once the pod's memory limit
is exceeded, Kubernetes OOMKills and restarts it.

**Parameters:**

| Field | Type | Default | Description |
|---|---|---|---|
| `loop_time_secs` | u64 | 10 | Seconds between allocations |
| `loop_amount_mb` | usize | 10 | MB allocated per iteration |

**Activation example** (OOMKill a 64 Mi pod in ~30–40 s):
```json
{ "cmd": "set", "case": "intermittent-02", "loop_time_secs": 10, "loop_amount_mb": 15 }
```

**`/rotectl/status` extras:**
```json
{ "loop_time_secs": 10, "loop_amount_mb": 15 }
```

**Persistence:** Parameters are saved to `/data/state.json` and the leak task
resumes automatically after a pod restart.

**Diagnosis value:**  
Identifies resource-exhaustion failures. Helps operators confirm the root cause
is memory growth rather than a logic bug, and gives a baseline for tuning pod
memory limits.

**Hurl test:** `tests/hurl-case/intermittent-02.hurl`

---

## Proposed

The following cases are planned but not yet implemented. Contributions welcome
— see [CONTRIBUTING.md](../CONTRIBUTING.md).

- **Pod deployment failure via k8s config** — missing required environment
  variable causes the app to exit on startup.
- **Service unreachable** — k8s Service selector mismatch; pod is healthy but
  no traffic reaches it.
- **Overspecified ingress** — both a LoadBalancer Service and an Ingress
  controller are configured, causing routing conflicts.
