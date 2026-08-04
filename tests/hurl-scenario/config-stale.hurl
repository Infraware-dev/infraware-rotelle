# config-stale.hurl — control path verification
#
# Activates config-stale, checks the served version in status, updates it with a
# new `version` param (simulating a ConfigMap edit the operator forces through),
# then resets. The silent drift itself is only observable in-cluster by comparing
# GET / against the current ConfigMap.
#
# Usage:
#   hurl --variable rotelle_service_host=<URL> tests/hurl-scenario/config-stale.hurl
#   just run-scenario config-stale

# ── activate (default version) ────────────────────────────────────────────────

POST {{rotelle_service_host}}/rotectl/cmd
Content-Type: application/json
{"cmd": "set", "scenario": "config-stale"}

HTTP 200
[Asserts]
jsonpath "$.ok" == true
jsonpath "$.scenario" == "config-stale"

GET {{rotelle_service_host}}/rotectl/status
HTTP 200
[Asserts]
jsonpath "$.scenario" == "config-stale"
jsonpath "$.served_version" == "v1"

# ── served body reflects the version ─────────────────────────────────────────

GET {{rotelle_service_host}}/
HTTP 200
[Asserts]
body contains "version=v1"

# ── "ConfigMap change" → new version ─────────────────────────────────────────

POST {{rotelle_service_host}}/rotectl/cmd
Content-Type: application/json
{"cmd": "set", "scenario": "config-stale", "version": "v2"}

HTTP 200
[Asserts]
jsonpath "$.ok" == true

GET {{rotelle_service_host}}/rotectl/status
HTTP 200
[Asserts]
jsonpath "$.served_version" == "v2"

GET {{rotelle_service_host}}/
HTTP 200
[Asserts]
body contains "version=v2"

# ── reset ─────────────────────────────────────────────────────────────────────

POST {{rotelle_service_host}}/rotectl/cmd
Content-Type: application/json
{"cmd": "reset"}

HTTP 200
[Asserts]
jsonpath "$.ok" == true
jsonpath "$.scenario" == "none, idle"