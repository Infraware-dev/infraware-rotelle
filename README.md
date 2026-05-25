# Rotelle

**Simulate Kubernetes failure scenarios. Train SREs and AI systems to diagnose them.**

[![CI](https://github.com/infraware-dev/infraware-rotelle/actions/workflows/ci.yml/badge.svg)](https://github.com/infraware-dev/infraware-rotelle/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)
[![Container](https://img.shields.io/badge/container-GHCR-blue)](https://github.com/infraware-dev/infraware-rotelle/pkgs/container/infraware-rotelle)

Rotelle is a lightweight Rust web server that runs inside a Kubernetes cluster and lets you trigger realistic failure conditions on demand. It exposes a clean control API so you can activate, observe and reset scenarios programmatically — making it ideal for SRE training labs, AI evaluation pipelines and chaos engineering foundations.

## Features

- **Realistic failure scenarios** — crash-loop, OOMKill, missing env var (CrashLoopBackOff), hung connections, ingress conflicts and more
- **Always-responsive control plane** — `/rotectl/` endpoints stay up even when a simulation is hanging or crashing
- **Survives pod restarts** — active scenario is persisted to disk and resumed automatically
- **Pluggable architecture** — adding a new scenario is a single Rust file + one registration line
- **Multi-arch container** — published to GHCR for both `amd64` and `arm64`

---

## Quick Start

You need: `kubectl` pointed at any cluster. No cluster yet? Clone the repo and run `./scripts/quickstart.sh` — it creates a Kind cluster, deploys Rotelle, and runs a live demo in under 10 minutes. See [ONBOARDING.md](ONBOARDING.md) for the full walkthrough.

> **Note:** The manifest uses `type: LoadBalancer`. On cloud providers (EKS, GKE, AKS) this provisions a public IP with no authentication. Use a test/staging cluster, or change the Service type to `ClusterIP` and access via `kubectl port-forward`. See [ONBOARDING.md](ONBOARDING.md) for details.

```sh
kubectl apply -f https://raw.githubusercontent.com/infraware-dev/infraware-rotelle/main/k8s/rotelle.yaml
kubectl rollout status deployment/rotelle -n rotelle --timeout=60s
kubectl port-forward -n rotelle svc/rotelle 8080:8080
```

Trigger your first failure scenario — a pod crash every 5 requests:

```sh
curl -X POST http://localhost:8080/rotectl/cmd \
  -H 'Content-Type: application/json' \
  -d '{"cmd": "set", "scenario": "crash-loop"}'
```

Reset to idle:

```sh
curl -X POST http://localhost:8080/rotectl/cmd \
  -H 'Content-Type: application/json' \
  -d '{"cmd": "reset"}'
```

Remove when done:

```sh
kubectl delete -f https://raw.githubusercontent.com/infraware-dev/infraware-rotelle/main/k8s/rotelle.yaml
```

---

## Scenarios

See [docs/scenarios.md](docs/scenarios.md) for the full list of scenarios with parameters and activation examples.

---

## Control API

All control endpoints live under `/rotectl/` and stay responsive even when a simulation is hanging or crashing.

| Method | Path | Description |
|---|---|---|
| `POST` | `/rotectl/cmd` | Activate or reset a scenario |
| `GET` | `/rotectl/status` | Current scenario name and extras |
| `GET` | `/rotectl/health` | Always 200 — liveness probe |

---

## Local Development

**Prerequisites:** Rust (stable), `just`, `hurl`.

```sh
just run                         # start server on :8080 (no cluster needed)
just run-scenario crash-loop         # run a single scenario test
just test-hurl                   # run the control-path smoke test
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for the full developer guide, how to add new scenarios, and how to deploy to a local cluster.

---

## Contributing

Contributions are welcome — especially new failure scenarios. See [CONTRIBUTING.md](CONTRIBUTING.md) for the 4-step process (implement, register, test, document).

---

## License

Apache License 2.0 — see [LICENSE](LICENSE).
