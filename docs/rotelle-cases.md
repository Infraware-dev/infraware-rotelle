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

### missing-env-var

**API case name:** `missing-env-var`  
**Source:** `src/scenario/missing_env_var.rs`

**What it simulates:** A deployment that exits on startup because a required
environment variable is absent, causing CrashLoopBackOff.

On first activation a 500 ms-delayed exit is spawned so the HTTP response and
state are persisted before the process dies. On every subsequent pod restart
(`on_resume`) the check runs immediately — if the variable is still absent, the
process exits before the server starts, reproducing the startup-failure loop.
Adding the variable to the deployment "fixes" the scenario without a reset.

**Parameters:**

| Field | Type | Default | Description |
|---|---|---|---|
| `required_var` | String | `REQUIRED_APP_SECRET` | Name of the environment variable to check |

**Activation example:**
```json
{ "cmd": "set", "case": "missing-env-var", "required_var": "DATABASE_URL" }
```

**`/rotectl/status` extras:**
```json
{ "required_var": "DATABASE_URL", "var_present": false }
```

**Diagnosis value:**  
Identifies missing configuration as the root cause of a CrashLoopBackOff.
Teaches operators to check `kubectl describe pod` for exit code 1 with no
visible error, then inspect env vars on the deployment.

**Hurl test:** `tests/hurl-case/missing-env-var.hurl`

---

### service-unreachable

**API case name:** `service-unreachable`  
**Source:** `src/scenario/service_unreachable.rs`

**What it simulates:** A Kubernetes Service whose selector does not match any
pod labels — the pod is healthy but receives no traffic.

Every `GET /` hangs the connection indefinitely (the async task sleeps; the
thread pool stays unblocked). The control plane (`/rotectl/*`) remains fully
responsive so the scenario can be reset while connections are hung.

**Parameters:** none

**Activation example:**
```json
{ "cmd": "set", "case": "service-unreachable" }
```

**Observe the hang:**
```sh
curl --max-time 5 http://localhost:8080/
```

**Diagnosis value:**  
Isolates traffic-routing failures from application failures. When `GET /`
times out but `/rotectl/status` succeeds, the pod is alive but unreachable
via the Service — pointing to a selector or label mismatch.

**Hurl test:** `tests/hurl-case/service-unreachable.hurl`

---

### ingress-conflict

**API case name:** `ingress-conflict`  
**Source:** `src/scenario/ingress_conflict.rs`

**What it simulates:** Routing conflicts from having both a LoadBalancer
Service and an Ingress controller configured simultaneously.

Returns HTTP 502 on every 3rd `GET /`; other requests return 200. The counter
resets on each activation.

**Parameters:** none

**Activation example:**
```json
{ "cmd": "set", "case": "ingress-conflict" }
```

**`/rotectl/status` extras:**
```json
{ "request_count": 4, "fail_every": 3 }
```

**Diagnosis value:**  
Reproduces the intermittent 502 errors that appear when both a LoadBalancer
Service and an Ingress rule compete to route traffic to the same workload.
Helps operators correlate gateway errors with duplicate routing configuration.

**Hurl test:** `tests/hurl-case/ingress-conflict.hurl`

---

## Proposed

No additional scenarios are currently planned. Contributions welcome — see
[CONTRIBUTING.md](../CONTRIBUTING.md).
