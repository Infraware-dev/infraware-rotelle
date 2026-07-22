# rotelle

Deploys Rotelle's two pods as a single Helm release:

| Component | Namespace (default) | What it does |
|-----------|---------------------|--------------|
| `rotelle` (sim pod) | `rotelle` | Runs scenarios, serves the simulation surface and its own `/rotectl/*` API. Intentionally crashable. |
| `rotelle-control` (control pod) | `rotelle-system` | Proxies commands to the sim pod's API over HTTP. Stays up regardless of what happens to the sim pod. |

The two-namespace split (see `docs/scenarios.md` / `docs/architecture.md` in
the parent repo) is what keeps the control pod reachable even when a
scenario takes the sim pod's own namespace down with it — not just
organizational. Both namespaces are created automatically; set
`createNamespaces: false` to bring your own.

See `values.yaml` for the full set of overridable knobs (image, resources,
persistence, service types, RBAC-free by design — this chart has no
cluster-scoped permissions of its own).

## Install

Default install — pulls the published image from GHCR
(`ghcr.io/infraware-dev/infraware-rotelle:v0.1.1`, overridable via
`image.tag`):

```sh
helm install rotelle ./helm/rotelle
```

```sh
kubectl port-forward -n rotelle svc/rotelle 8080:8080 &
kubectl port-forward -n rotelle-system svc/rotelle-control 9090:9090 &
```

### Local dev image

**If you're iterating on the Rust binary** (rebuild, redeploy, repeat), use
`k8s/rotelle-dev.yaml` + `k8s/rotelle-control-dev.yaml` directly instead of
this chart — that's what `./scripts/quickstart.sh --build-from-source`
already does. They deploy into their own `rotelle-dev` namespace, so
nothing here needs to be involved, and there's no Helm release state to
keep in sync with each rebuild:

```sh
kubectl apply -f k8s/rotelle-dev.yaml
kubectl apply -f k8s/rotelle-control-dev.yaml
```

Don't reach for `helm upgrade --set image.tag=dev` or `kubectl set image`
against an existing Helm release for this — both work once, but neither
updates what Helm thinks the release looks like, so the next
`helm upgrade`/`diff` silently reverts your patched image.

**If you're instead validating the chart itself** (namespace wiring, RBAC,
values plumbing) against a local image, use the bundled override file:

```sh
helm install rotelle ./helm/rotelle -f ./helm/rotelle/values-dev.yaml
```

This swaps in `rotelle:dev` with `imagePullPolicy: Never` — load the image
into your cluster first (e.g. `kind load docker-image rotelle:dev`).

## Installing alongside observability-gp-mini

Rotelle doesn't emit OTLP itself — but every scenario it runs still shows up
in two places that [`helm/observability-gp-mini`](../observability-gp-mini)
already watches cluster-wide:

- **Container logs** — `otel-node-collector` tails every pod's log file on
  the node, rotelle's included, no extra wiring needed.
- **Kubernetes events** — `k8s-events-exporter` ships every `Event` object
  (pod scheduled, `CrashLoopBackOff`, `OOMKilled`, etc.) into gigapipe. This
  is exactly what rotelle's `crash-loop` and `oom-kill` scenarios produce.

Install both into the same cluster (separate releases, separate
namespaces):

```sh
helm install observability ./helm/observability-gp-mini -n observability --create-namespace
helm install rotelle ./helm/rotelle
```

Then, once a scenario is active (see the top-level README's Quick Start),
query gigapipe for what it caused:

```logql
{k8s_container_name="rotelle"}   # rotelle's own container logs
{job="k8s-events"}               # cluster events, including crashes/OOMKills
```

## Uninstall

```sh
helm uninstall rotelle
```

With `createNamespaces: true` (the default), this deletes the `rotelle` and
`rotelle-system` namespaces too — and with them, the sim pod's PVC and its
contents. Set `createNamespaces: false` and manage the namespaces yourself
if you'd rather they outlive the release.
