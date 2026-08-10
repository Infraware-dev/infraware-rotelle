# Scenarios

Each scenario is a named failure condition activated via `POST /rotectl/cmd`.

## Quick reference

| Name | What it simulates |
|---|---|
| `none, idle` | Default state — no failure active |
| `check` | Control checkpoint for test sequences |
| `crash-loop` | Pod exits on every 5th request → CrashLoopBackOff |
| `oom-kill` | Background memory leak → OOMKill |
| `missing-env-var` | Pod exits on startup if a required env var is absent → CrashLoopBackOff |
| `service-unreachable` | Every `GET /` hangs — simulates a Service with no matching pods |
| `ingress-conflict` | Every 3rd request returns 502 — simulates a routing conflict |
| `config-stale` | Serves a stale config version — models a pod ignoring a ConfigMap update |
| `graceful-shutdown-failure` | Ignores SIGTERM while still serving → dropped connections on rollout |

---

## none, idle

**Source:** `rotelle/src/scenario/idle.rs`

Default state. The service responds normally to all requests. This is the target of the `reset` command.

---

## check

**Source:** `rotelle/src/scenario/check.rs`

Control checkpoint. Marks a known, stable state for test sequences to assert against. Activated by the `check` command or `{"cmd": "set", "scenario": "check"}`.

---

## crash-loop

**Source:** `rotelle/src/scenario/crash_loop.rs`

**What it simulates:** Intermittent pod crash-loops.

The process calls `exit(1)` on every 5th `GET /`. Kubernetes detects the non-zero exit and restarts the pod. The counter resets on each restart.

**Parameters:** none

**Activate:**
```json
{ "cmd": "set", "scenario": "crash-loop" }
```

**`/rotectl/status` extras:**
```json
{ "access_count": 3, "crash_every": 5 }
```

**Diagnosis value:** Identifies whether a failure is caused by the pod process itself (predictable crash cycle) versus infrastructure or configuration.

**Hurl test:** `tests/hurl-scenario/crash-loop.hurl`

---

## oom-kill

**Source:** `rotelle/src/scenario/oom_kill.rs`

**What it simulates:** Pod memory exhaustion leading to OOMKill.

A background task wakes every `loop_time_secs` seconds and allocates `loop_amount_mb` MB of memory that is never freed. Once the pod's memory limit is exceeded, Kubernetes OOMKills and restarts it.

**Parameters:**

| Field | Type | Default | Description |
|---|---|---|---|
| `loop_time_secs` | u64 | 10 | Seconds between allocations |
| `loop_amount_mb` | usize | 10 | MB allocated per iteration |

**Activate** (OOMKill a 64 Mi pod in ~30–40 s):
```json
{ "cmd": "set", "scenario": "oom-kill", "loop_time_secs": 10, "loop_amount_mb": 15 }
```

**`/rotectl/status` extras:**
```json
{ "loop_time_secs": 10, "loop_amount_mb": 15 }
```

**Persistence:** Parameters are saved to `/data/state.json` and the leak task resumes automatically after a pod restart.

**Diagnosis value:** Identifies resource-exhaustion failures. Helps operators confirm memory growth as the root cause and gives a baseline for tuning pod memory limits.

**Note:** Requires a memory limit to be enforced. The default manifest sets `limits.memory: 64Mi`. If your cluster's admission controller overrides this, the OOMKill will not trigger — verify with `kubectl describe pod -n rotelle <pod> | grep -A5 Limits`.

**Hurl test:** `tests/hurl-scenario/oom-kill.hurl`

---

## missing-env-var

**Source:** `rotelle/src/scenario/missing_env_var.rs`

**What it simulates:** A deployment that exits on startup because a required environment variable is absent, causing CrashLoopBackOff.

Activation stores the required variable name. The crash triggers on `GET /` when `var_value` is empty — the pod exits and Kubernetes applies CrashLoopBackOff. The `/rotectl/*` endpoints remain accessible between crash cycles, so `var_value` can be set via the control pod while the pod is up.

**Parameters:**

| Field | Type | Default | Description |
|---|---|---|---|
| `required_var` | String | `REQUIRED_APP_SECRET` | Name of the environment variable to check |
| `var_value` | String | `""` (empty) | Leave empty to trigger the crash; set to any value to simulate the var being present |

**Activate** (triggers crash loop):
```json
{ "cmd": "set", "scenario": "missing-env-var", "required_var": "DATABASE_URL" }
```

**Fix** (set var_value to stop crashing — send via the control pod while the sim is up):
```json
{ "cmd": "set", "scenario": "missing-env-var", "required_var": "DATABASE_URL", "var_value": "postgres://..." }
```

**`/rotectl/status` extras:**
```json
{ "required_var": "DATABASE_URL", "var_present": false }
```

**Diagnosis value:** Identifies missing configuration as the root cause of a CrashLoopBackOff. Teaches operators to look for exit code 1 with no error logs in `kubectl describe pod`, then inspect env vars on the deployment.

**Note:** After reset, the pod may still be in crash-loop if it restarted while the scenario was active. Force a clean restart: `kubectl rollout restart deployment/rotelle -n rotelle`.

**Hurl test:** `tests/hurl-scenario/missing-env-var.hurl`

---

## service-unreachable

**Source:** `rotelle/src/scenario/service_unreachable.rs`

**What it simulates:** A Kubernetes Service whose selector does not match any pod labels — the pod is healthy but receives no traffic.

Every `GET /` hangs the connection indefinitely (the async task sleeps; the thread pool stays unblocked). The control plane (`/rotectl/*`) remains fully responsive so the scenario can be reset while connections are hung.

**Parameters:** none

**Activate:**
```json
{ "cmd": "set", "scenario": "service-unreachable" }
```

**Diagnosis value:** Isolates traffic-routing failures from application failures. When `GET /` times out but `/rotectl/status` succeeds, the pod is alive but unreachable via the Service — pointing to a selector or label mismatch.

**Hurl test:** `tests/hurl-scenario/service-unreachable.hurl`

---

## ingress-conflict

**Source:** `rotelle/src/scenario/ingress_conflict.rs`

**What it simulates:** Routing conflicts from having both a LoadBalancer Service and an Ingress controller configured simultaneously.

Returns HTTP 502 on every 3rd `GET /`; other requests return 200. The counter resets on each activation.

**Parameters:** none

**Activate:**
```json
{ "cmd": "set", "scenario": "ingress-conflict" }
```

**`/rotectl/status` extras:**
```json
{ "request_count": 4, "fail_every": 3 }
```

**Diagnosis value:** Reproduces the intermittent 502 errors that appear when both a LoadBalancer Service and an Ingress rule compete to route traffic to the same workload.

**Hurl test:** `tests/hurl-scenario/ingress-conflict.hurl`

---

## config-stale

**Source:** `rotelle/src/scenario/config_stale.rs`

**What it simulates:** A pod serving stale configuration after a ConfigMap update.

A ConfigMap consumed as environment variables is snapshotted into the pod at start and never refreshes when the ConfigMap changes — the pod keeps serving the old value until it is restarted. Activation captures a `version` string; every `GET /` renders that version, unchanged, until the scenario is re-activated with a new `version` (the manual intervention a real fix requires).

Unlike every other scenario, the failure is silent and data-level: no crash, no error, no latency — just subtly wrong data.

**Parameters:**

| Field | Type | Default | Description |
|---|---|---|---|
| `version` | String | `v1` | The config version the pod serves on every request |

**Activate:**
```json
{ "cmd": "set", "scenario": "config-stale", "version": "v2" }
```

**`/rotectl/status` extras:**
```json
{ "served_version": "v2" }
```

**Persistence:** The version is saved to `/data/state.json` and resumes automatically after a pod restart — modelling a pod that keeps serving stale config even across restarts. An explicit `reset` returns the service to idle.

**Diagnosis value:** The only silent, data-level scenario. There is nothing in logs, events, or pod status to find. Diagnosis requires comparing the value the pod actually serves against the current ConfigMap — e.g. `kubectl exec <pod> -- printenv VERSION` versus `kubectl get configmap <name> -o jsonpath='{.data.VERSION}'`. Trains operators to spot config drift between running pods and the declared ConfigMap.

**Hurl test:** `tests/hurl-scenario/config-stale.hurl`

---

## graceful-shutdown-failure

**Source:** `rotelle/src/scenario/graceful_shutdown_failure.rs`

**What it simulates:** A pod that ignores SIGTERM during a rolling update.

Kubernetes sends SIGTERM and waits `terminationGracePeriodSeconds`. While this scenario is active the process swallows the signal for `ignore_secs`, keeps answering `GET /` with 200, and only then exits — so in-flight requests are severed when the kubelet's SIGKILL lands. When the scenario is inactive, rotelle exits promptly on SIGTERM.

**Parameters:**

| Field | Type | Default | Description |
|---|---|---|---|
| `ignore_secs` | u64 | `30` | How long SIGTERM is ignored while the pod keeps serving |

**Activate:**
```json
{ "cmd": "set", "scenario": "graceful-shutdown-failure", "ignore_secs": 90 }
```

**`/rotectl/status` extras:**
```json
{ "ignore_secs": 90, "armed": true, "requests_served": 12 }
```

**Triggering it:** the failure only appears on a pod lifecycle event.

```bash
kubectl -n rotelle get pod -w &
kubectl -n rotelle delete pod -l app=rotelle    # or: kubectl exec <pod> -- kill -TERM 1
```

With `terminationGracePeriodSeconds: 60` in the manifests, `ignore_secs=30` shows a delete blocked for 30 s that then exits cleanly; `ignore_secs=90` shows the pod SIGKILLed with requests in flight. Keep traffic flowing during the delete (`while true; do curl -s -o /dev/null -w '%{http_code}\n' <url>/; done`) to see the resets.

Locally, Ctrl-C sends SIGINT, which is unhandled — use `kill -TERM <pid>` instead.

**Persistence:** `ignore_secs` is saved to `/data/state.json` and re-arms on restart, so the next rollout fails the same way until an explicit `reset`.

**Diagnosis value:** Trains operators to spot a missing `preStop` hook and improper shutdown handling. Symptoms appear only during pod lifecycle events — a blocking `kubectl delete pod` waits out the entire window, pods sit in `Terminating` far longer than expected while still `Ready` and serving traffic, and clients see connection resets when the kubelet's SIGKILL lands. Every probe stays green throughout, so nothing in pod status or events points at the cause.

Note that `kubectl rollout status` may report success while the doomed pod is still `Terminating`: recent Kubernetes versions account for terminating pods separately (`ReplicaSet.status.terminatingReplicas`) and no longer hold the rollout open for them. `kubectl get pod -w` is the reliable signal.

**Hurl test:** `tests/hurl-scenario/graceful-shutdown-failure.hurl`

---

## Proposed

No additional scenarios are currently planned. Contributions welcome — see [CONTRIBUTING.md](../CONTRIBUTING.md).
