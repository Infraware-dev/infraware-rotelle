# observability-gp-mini

A minimal reference observability stack built around [gigapipe](https://github.com/metrico/qryn)
(qryn) — one chart, one `helm install`, everything wired together:

| Component | What it ships |
|-----------|----------------|
| `gigapipe` (dependency) | OTLP logs/traces + Loki-push ingestion, Prometheus remote-write, backed by ClickHouse. |
| `clickhouse` (this chart's own template) | Single-node ClickHouse for gigapipe to write to. Not fit for production — no auth, no replication, emptyDir by default. |
| `otel-gateway` | A minimal upstream OpenTelemetry Collector translating app-level OTLP metrics into the Prometheus remote-write protocol gigapipe accepts natively (it has no OTLP metrics receiver of its own). |
| `otel-node-collector` | A per-node DaemonSet: tails every container's logs, scrapes node OS metrics, and scrapes kubelet pod/container stats. |
| `k8s-events-exporter` | Ships Kubernetes `Event` objects (pod scheduled, container crashed, etc.) into gigapipe so they outlive `kubectl get events`'s short TTL. |

Not fit for production as shipped — single replicas throughout, no auth on
gigapipe or ClickHouse. This exists to give a demo, training, or dev cluster
somewhere to point OTLP metrics/logs and Kubernetes events at with one
command, not to be a production observability backend.

## Why these four components live in one chart instead of one each

They only exist to feed gigapipe — nothing else in this chart (or, as far as
this repo is concerned, anywhere) consumes them independently today. Folding
them in as plain templates avoids a `file://` sibling-chart dependency that
would break the moment this chart moves to a repository of its own, and cuts
out a `helm dependency build` step for pieces that only ever travel together.
Each still lives under its own `templates/<component>/` subdirectory, so
splitting one back out into its own chart later — if a real second consumer
ever shows up — is a directory move, not a rewrite.

## Pointing an external app at this stack

An app outside this chart's own namespace reaches these over Kubernetes
cross-namespace Service DNS — there's no other channel once they're separate
releases:

```
http://gigapipe.<this-chart's-namespace>.svc.cluster.local:3100        # logs/traces (OTLP), Loki push
http://otel-gateway.<this-chart's-namespace>.svc.cluster.local:4318/v1/metrics   # metrics (OTLP, note the required path)
```

## Values

See `values.yaml` — each component's block carries its own inline
documentation (image, resources, RBAC, and its rendered-verbatim `config`).

## Example queries

Kubernetes events:
```logql
{job="k8s-events"}
```

Node CPU:
```promql
system_cpu_utilization
```

Logs from a specific pod:
```logql
{k8s_container_name="agent-server"}
```
