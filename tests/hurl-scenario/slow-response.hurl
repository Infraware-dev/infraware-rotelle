# slow-response.hurl — activate slow-response degraded-pod mode
#
# Delays every GET / response by delay_ms milliseconds while the control plane
# (/rotectl/*) stays instant. Simulates a pod that is alive but responding so
# slowly it exceeds readiness probe timeouts and is marked NotReady.
#
# Usage:
#   hurl --variable rotelle_service_host=<URL> tests/hurl-scenario/slow-response.hurl
#   just run-scenario slow-response

# ── activate slow-response ────────────────────────────────────────────────────

POST {{rotelle_service_host}}/rotectl/cmd
Content-Type: application/json
{"cmd": "set", "scenario": "slow-response", "delay_ms": 500}

HTTP 200
[Asserts]
jsonpath "$.ok" == true
jsonpath "$.scenario" == "slow-response"

GET {{rotelle_service_host}}/rotectl/status
HTTP 200
[Asserts]
jsonpath "$.scenario" == "slow-response"
jsonpath "$.delay_ms" == 500

# ── GET / responds, but only after the configured delay ───────────────────────

GET {{rotelle_service_host}}/
HTTP 200
[Asserts]
body contains "slow-response"
duration >= 500

# ── reset restores normal behaviour ───────────────────────────────────────────

POST {{rotelle_service_host}}/rotectl/cmd
Content-Type: application/json
{"cmd": "reset"}

HTTP 200
[Asserts]
jsonpath "$.ok" == true
jsonpath "$.scenario" == "none, idle"
