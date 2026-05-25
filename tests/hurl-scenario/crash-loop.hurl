# crash-loop.hurl — activate crash-loop mode
#
# Every 5th GET / causes the process to exit; k8s restarts the pod.
# Makes 4 requests (one short of the crash threshold) to verify the scenario
# is active without actually crashing the local server during tests.
#
# Usage:
#   hurl --variable rotelle_service_host=<URL> tests/hurl-scenario/crash-loop.hurl
#   just run-scenario crash-loop

# ── activate crash-loop ───────────────────────────────────────────────────────

POST {{rotelle_service_host}}/rotectl/cmd
Content-Type: application/json
{"cmd": "set", "scenario": "crash-loop"}

HTTP 200
[Asserts]
jsonpath "$.ok" == true
jsonpath "$.scenario" == "crash-loop"

GET {{rotelle_service_host}}/rotectl/status
HTTP 200
[Asserts]
jsonpath "$.scenario" == "crash-loop"

# ── verify service responds (n_accesses succeed; next crash triggers exit) ───

GET {{rotelle_service_host}}/
[Options]
repeat: 4
HTTP 200
[Asserts]
body contains "crash-loop"

# access (n_accesses+1) will exit(1) — observe via kubectl logs / pod restarts
