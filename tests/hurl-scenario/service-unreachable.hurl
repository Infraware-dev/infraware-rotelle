# service-unreachable.hurl — control path verification
#
# Tests activation and status only. GET / hangs indefinitely when this scenario
# is active — do not include a GET / assertion here without a client-side timeout.
# To observe the hang manually: curl --max-time 5 <host>/
#
# Usage:
#   hurl --variable rotelle_service_host=<URL> tests/hurl-scenario/service-unreachable.hurl
#   just run-scenario service-unreachable

# ── activate ──────────────────────────────────────────────────────────────────

POST {{rotelle_service_host}}/rotectl/cmd
Content-Type: application/json
{"cmd": "set", "scenario": "service-unreachable"}

HTTP 200
[Asserts]
jsonpath "$.ok" == true
jsonpath "$.scenario" == "service-unreachable"

# ── status reachable while GET / hangs ────────────────────────────────────────

GET {{rotelle_service_host}}/rotectl/status
HTTP 200
[Asserts]
jsonpath "$.scenario" == "service-unreachable"
jsonpath "$.description" exists

# ── reset restores normal behaviour ──────────────────────────────────────────

POST {{rotelle_service_host}}/rotectl/cmd
Content-Type: application/json
{"cmd": "reset"}

HTTP 200
[Asserts]
jsonpath "$.ok" == true
jsonpath "$.scenario" == "none, idle"

GET {{rotelle_service_host}}/
HTTP 200
[Asserts]
body contains "none, idle"
