# ingress-conflict.hurl — routing conflict verification
#
# Exercises the alternating 200/502 pattern: requests 1 and 2 succeed, request 3
# returns 502, then the cycle repeats. Counter resets on activation.
#
# Usage:
#   hurl --variable rotelle_service_host=<URL> tests/hurl-case/ingress-conflict.hurl
#   just run-case ingress-conflict

# ── activate ──────────────────────────────────────────────────────────────────

POST {{rotelle_service_host}}/rotectl/cmd
Content-Type: application/json
{"cmd": "set", "case": "ingress-conflict"}

HTTP 200
[Asserts]
jsonpath "$.ok" == true
jsonpath "$.failure_case" == "ingress-conflict"

GET {{rotelle_service_host}}/rotectl/status
HTTP 200
[Asserts]
jsonpath "$.failure_case" == "ingress-conflict"
jsonpath "$.fail_every" == 3

# ── alternating 200 / 502 pattern ────────────────────────────────────────────

GET {{rotelle_service_host}}/
HTTP 200
[Asserts]
body contains "ingress-conflict"

GET {{rotelle_service_host}}/
HTTP 200
[Asserts]
body contains "ingress-conflict"

GET {{rotelle_service_host}}/
HTTP 502
[Asserts]
body contains "502 Bad Gateway"

GET {{rotelle_service_host}}/
HTTP 200

GET {{rotelle_service_host}}/
HTTP 200

GET {{rotelle_service_host}}/
HTTP 502

# ── reset ─────────────────────────────────────────────────────────────────────

POST {{rotelle_service_host}}/rotectl/cmd
Content-Type: application/json
{"cmd": "reset"}

HTTP 200
[Asserts]
jsonpath "$.ok" == true
jsonpath "$.failure_case" == "none, idle"
