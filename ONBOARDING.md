# Getting Started with Rotelle

> **Goal:** Rotelle deployed and your first failure scenario firing - in under 10 minutes. Works with any Kubernetes cluster; Option B includes a local Kind setup if you don't have one yet.

Rotelle is a lightweight Kubernetes-native tool for simulating realistic pod failure conditions on demand: crash-loops, OOMKills, missing environment variables, hung connections, ingress conflicts and more. It exposes a clean control API so you can trigger, observe and reset scenarios programmatically - making it ideal for SRE training, AI evaluation pipelines and chaos engineering foundations.

---

## Choose your path

| I want to… | Go to |
|---|---|
| **Deploy to an existing test cluster** (kind, minikube…) | [Option A — Existing cluster](#option-a--existing-cluster) |
| **Try it locally with no cluster yet** | [Option B — Kind quickstart](#option-b--kind-quickstart) |
| **Add scenarios or fix bugs** (open-source contributor) | [Contribute to Rotelle](#contribute-to-rotelle) |

---

## Run Rotelle

### Option A — Existing cluster

Deploy to any cluster reachable via `kubectl cluster-info`. No local build required — Kubernetes pulls the published image from GHCR automatically.

**Prerequisites:** [`kubectl`](https://kubernetes.io/docs/tasks/tools/) pointed at your cluster, `curl`

> **⚠️ Deploy to a test or dedicated staging cluster only — not production.** Rotelle crashes pods, leaks memory, and hangs connections on command. The control API has no authentication: any client that can reach the service can activate failure scenarios.
>
> The manifest uses `type: LoadBalancer`, which creates a public IP on cloud providers (EKS, GKE, AKS). If you're deploying to a shared cluster, change the Service type to `ClusterIP` in `k8s/rotelle.yaml` and use `kubectl port-forward` for access, or restrict the endpoint with a network policy.

```sh
kubectl apply -f https://raw.githubusercontent.com/infraware-dev/infraware-rotelle/main/k8s/rotelle.yaml
kubectl rollout status deployment/rotelle -n rotelle --timeout=60s
kubectl port-forward -n rotelle svc/rotelle 8080:8080
```

Tear down when done:

```sh
kubectl delete -f https://raw.githubusercontent.com/infraware-dev/infraware-rotelle/main/k8s/rotelle.yaml
```

---

### Option B — Kind quickstart

For first-time evaluation with no cluster. Creates a local Kind cluster, deploys Rotelle, and runs a live demo.

**Prerequisites:**

| Tool | Notes |
|---|---|
| [Docker](https://docs.docker.com/get-docker/) | Daemon must be running |
| [kind](https://kind.sigs.k8s.io/docs/user/quick-start/#installation) | v0.27+ |
| [kubectl](https://kubernetes.io/docs/tasks/tools/) | v1.29+ |
| `curl` | For driving the control API |

```sh
git clone https://github.com/infraware-dev/infraware-rotelle.git
cd infraware-rotelle
./scripts/quickstart.sh
```

The script:
1. Checks prerequisites and gives you install hints if anything is missing
2. Creates a Kind cluster called `rotelle-test` (1 control-plane + 2 workers)
3. Deploys the last published release from GHCR (not your local source)
4. Opens a port-forward to `http://localhost:8080` — exits with a clear error if port 8080 is already in use
5. Activates your first failure scenario so you can see it live

**Expected duration:** ~3–5 minutes (dominated by image pull on first run).

> **If the image pull fails** (e.g. the image is not yet published), build from source instead — requires Rust:
> ```sh
> ./scripts/quickstart.sh --build-from-source
> ```

---

### Step 2 — Verify Rotelle is running

```sh
curl http://localhost:8080/rotectl/status
# {"scenario":"none, idle","description":"No failure active — service responds normally."}
```

`scenario: "none, idle"` means Rotelle is idle and healthy — ready to simulate failures.

---

### Step 3 — Run your first scenario

#### Crash-loop (`crash-loop`)

**What it simulates:** The pod calls `exit(1)` on every 5th request. Kubernetes detects the non-zero exit and restarts the pod — the classic CrashLoopBackOff pattern.

**Activate:**

```sh
curl -X POST http://localhost:8080/rotectl/cmd \
  -H 'Content-Type: application/json' \
  -d '{"cmd": "set", "scenario": "crash-loop"}'
```

**Observe — open two terminals:**

```sh
# Terminal 1: watch pod lifecycle
kubectl get pods -n rotelle -w

# Terminal 2: drive traffic (the 5th request crashes the pod)
for i in $(seq 1 7); do
  echo "Request $i:"
  curl -s --max-time 5 http://localhost:8080/ | head -c 80 || echo "(pod crashed)"
  sleep 1
done
```

After the 5th request you will see the pod restart and `RESTARTS` increment in Terminal 1.

**Check status while running:**

```sh
curl http://localhost:8080/rotectl/status
# {"scenario":"crash-loop","description":"...","access_count":3,"crash_every":5}
```

The control API (`/rotectl/*`) always responds — even while the application is crashing.

**Reset:**

```sh
curl -X POST http://localhost:8080/rotectl/cmd \
  -H 'Content-Type: application/json' \
  -d '{"cmd": "reset"}'
```

---

The full list of scenarios with activation parameters, status extras, and diagnosis notes is in [docs/scenarios.md](docs/scenarios.md).

### Control API reference

All `/rotectl/` endpoints stay responsive even when a simulation is crashing or hanging.

| Method | Path | Description |
|---|---|---|
| `POST` | `/rotectl/cmd` | Activate (`set`), reset (`reset`), or checkpoint (`check`) a scenario |
| `GET` | `/rotectl/status` | Current scenario name, params, and scenario-specific extras |
| `GET` | `/rotectl/health` | Always 200 — used as the pod liveness probe |
| `GET` | `/health` | Application health — may fail depending on the active scenario |

---

### Cleanup

```sh
# Option A — remove Rotelle from your cluster
kubectl delete -f https://raw.githubusercontent.com/infraware-dev/infraware-rotelle/main/k8s/rotelle.yaml

# Option B — delete the Kind cluster entirely
kind delete cluster --name rotelle-test
```

---

### Troubleshooting

**Port 8080 is already in use**

The script exits before starting the port-forward. Find and stop whatever owns port 8080 (e.g. a local `just run` server), then re-run:

```sh
kill $(lsof -iTCP:8080 -sTCP:LISTEN -t)
./scripts/quickstart.sh
```

**Kind quickstart fails at cluster creation**

Check that Docker has enough resources allocated (minimum: 4 CPUs, 4 GB RAM). On Docker Desktop, adjust in Preferences → Resources.

---

## Contribute to Rotelle

Contributions are welcome — especially new failure scenarios.

See [CONTRIBUTING.md](CONTRIBUTING.md) for the full guide: development setup, how to add a scenario and the contribution workflow.