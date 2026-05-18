# intermittent-01.hurl — activate intermittent crash mode
#
# Every 5th GET / causes the process to exit; k8s restarts the pod.
# Makes 4 requests (one short of the crash threshold) to verify the scenario
# is active without actually crashing the local server during tests.
#
# Usage:
#   hurl --variable rotelle_service_host=<URL> tests/hurl-case/intermittent-01.hurl
#   just run-case intermittent-01

# ── activate intermittent-01 ──────────────────────────────────────────────────

POST {{rotelle_service_host}}/rotectl/cmd
Content-Type: application/json
{"cmd": "set", "case": "intermittent-01"}

HTTP 200
[Asserts]
jsonpath "$.ok" == true
jsonpath "$.failure_case" == "intermittent-01"

GET {{rotelle_service_host}}/rotectl/status
HTTP 200
[Asserts]
jsonpath "$.failure_case" == "intermittent-01"

# ── verify service responds (n_accesses succeed; next crash triggers exit) ───

GET {{rotelle_service_host}}/
[Options]
repeat: 4
HTTP 200
[Asserts]
body contains "intermittent-01"

# access (n_accesses+1) will exit(1) — observe via kubectl logs / pod restarts
