# keep-alive-timeout.hurl — activate keep-alive-timeout mode
#
# Serves valid HTTP 200 responses but forces `Connection: close` on each
# request, simulating a pod whose keepalive idle timeout is shorter than its
# ingress/LB. In production this causes intermittent 502s when the proxy
# reuses a connection the pod has already closed — pod healthy, logs clean.
#
# Usage:
#   hurl --variable rotelle_service_host=<URL> tests/hurl-scenario/keep-alive-timeout.hurl
#   just run-scenario keep-alive-timeout

# ── activate keep-alive-timeout ───────────────────────────────────────────────

POST {{rotelle_service_host}}/rotectl/cmd
Content-Type: application/json
{"cmd": "set", "scenario": "keep-alive-timeout", "idle_secs": 10}

HTTP 200
[Asserts]
jsonpath "$.ok" == true
jsonpath "$.scenario" == "keep-alive-timeout"

# ── verify status reports the scenario and its extras ─────────────────────────

GET {{rotelle_service_host}}/rotectl/status
HTTP 200
[Asserts]
jsonpath "$.scenario" == "keep-alive-timeout"
jsonpath "$.idle_secs" == 10

# ── the failure signature: valid 200, but the connection is closed ────────────

GET {{rotelle_service_host}}/
HTTP 200
[Asserts]
header "connection" == "close"
body contains "keep-alive-timeout"

# ── each request increments the closed-connection counter ─────────────────────

GET {{rotelle_service_host}}/rotectl/status
HTTP 200
[Asserts]
jsonpath "$.closed_connections" >= 1

# ── reset back to idle ────────────────────────────────────────────────────────

POST {{rotelle_service_host}}/rotectl/cmd
Content-Type: application/json
{"cmd": "reset"}

HTTP 200
[Asserts]
jsonpath "$.ok" == true
jsonpath "$.scenario" == "none, idle"