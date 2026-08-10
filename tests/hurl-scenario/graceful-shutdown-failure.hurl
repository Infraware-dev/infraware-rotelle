# graceful-shutdown-failure.hurl — control path verification
#
# Hurl cannot send signals, so this covers activation, parameter updates and the
# fact that the service keeps serving 200 while armed. The failure itself only
# manifests on a pod lifecycle event — see docs/scenarios.md for the kubectl runbook.
#
# Usage:
#   hurl --variable rotelle_service_host=<URL> tests/hurl-scenario/graceful-shutdown-failure.hurl
#   just run-scenario graceful-shutdown-failure

# ── activate (default ignore_secs) ────────────────────────────────────────────

POST {{rotelle_service_host}}/rotectl/cmd
Content-Type: application/json
{"cmd": "set", "scenario": "graceful-shutdown-failure"}

HTTP 200
[Asserts]
jsonpath "$.ok" == true
jsonpath "$.scenario" == "graceful-shutdown-failure"

GET {{rotelle_service_host}}/rotectl/status
HTTP 200
[Asserts]
jsonpath "$.scenario" == "graceful-shutdown-failure"
jsonpath "$.ignore_secs" == 30
jsonpath "$.armed" == true

# ── the pod keeps serving normally while armed ───────────────────────────────

GET {{rotelle_service_host}}/
HTTP 200
[Asserts]
body contains "30s"

# ── longer window than the grace period → SIGKILL path ───────────────────────

POST {{rotelle_service_host}}/rotectl/cmd
Content-Type: application/json
{"cmd": "set", "scenario": "graceful-shutdown-failure", "ignore_secs": 90}

HTTP 200
[Asserts]
jsonpath "$.ok" == true

GET {{rotelle_service_host}}/rotectl/status
HTTP 200
[Asserts]
jsonpath "$.ignore_secs" == 90

# ── reset disarms the handler ────────────────────────────────────────────────

POST {{rotelle_service_host}}/rotectl/cmd
Content-Type: application/json
{"cmd": "reset"}

HTTP 200
[Asserts]
jsonpath "$.ok" == true
jsonpath "$.scenario" == "none, idle"
