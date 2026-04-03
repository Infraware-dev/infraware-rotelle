# c0-c1-check.hurl — control path verification sequence
#
# Usage:
#   default URL value "http://127.0.0.1:8080"
#   hurl --variable rotelle_service_host=<URL> tests/hurl/c0-c1-check.hurl
#   just test-hurl

# ── c0: reset to idle ────────────────────────────────────────────────────────

POST {{rotelle_service_host}}/rotectl/cmd
Content-Type: application/json
{"cmd": "reset"}

HTTP 200
[Asserts]
jsonpath "$.ok" == true
jsonpath "$.failure_case" == "none, idle"

GET {{rotelle_service_host}}/
HTTP 200
[Asserts]
body contains "none, idle"

# ── c1: set failure case ──────────────────────────────────────────────────────

POST {{rotelle_service_host}}/rotectl/cmd
Content-Type: application/json
{"cmd": "set", "case": "c1"}

HTTP 200
[Asserts]
jsonpath "$.ok" == true
jsonpath "$.failure_case" == "c1"

GET {{rotelle_service_host}}/
HTTP 200
[Asserts]
body contains "c1"

GET {{rotelle_service_host}}/rotectl/status
HTTP 200
[Asserts]
jsonpath "$.failure_case" == "c1"

# ── check: set check failure case ────────────────────────────────────────────

POST {{rotelle_service_host}}/rotectl/cmd
Content-Type: application/json
{"cmd": "check"}

HTTP 200
[Asserts]
jsonpath "$.ok" == true
jsonpath "$.failure_case" == "check"

GET {{rotelle_service_host}}/
HTTP 200
[Asserts]
body contains "check"

GET {{rotelle_service_host}}/rotectl/status
HTTP 200
[Asserts]
jsonpath "$.failure_case" == "check"

# ── c0: reset back to idle ────────────────────────────────────────────────────

POST {{rotelle_service_host}}/rotectl/cmd
Content-Type: application/json
{"cmd": "reset"}

HTTP 200
[Asserts]
jsonpath "$.ok" == true
jsonpath "$.failure_case" == "none, idle"

GET {{rotelle_service_host}}/
HTTP 200
[Asserts]
body contains "none, idle"
