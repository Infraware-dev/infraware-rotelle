# oom-kill.hurl — activate oom-kill memory leak mode
#
# Allocates loop_amount_mb every loop_time_secs seconds until the pod is
# OOMKilled by k8s (memory limit: 64Mi).  At 15 MB / 10 s the pod exceeds
# the limit after roughly 3-4 allocation cycles (~30-40 s).
#
# Usage:
#   hurl --variable rotelle_service_host=<URL> tests/hurl-scenario/oom-kill.hurl
#   just run-scenario oom-kill

# ── activate oom-kill ─────────────────────────────────────────────────────────

POST {{rotelle_service_host}}/rotectl/cmd
Content-Type: application/json
{"cmd": "set", "scenario": "oom-kill", "loop_time_secs": 10, "loop_amount_mb": 15}

HTTP 200
[Asserts]
jsonpath "$.ok" == true
jsonpath "$.scenario" == "oom-kill"

GET {{rotelle_service_host}}/rotectl/status
HTTP 200
[Asserts]
jsonpath "$.scenario" == "oom-kill"

# ── verify service is responding before leak takes hold ───────────────────────

GET {{rotelle_service_host}}/
HTTP 200
[Asserts]
body contains "oom-kill"

# ── observe OOMKill via: kubectl get pod -n rotelle -w ────────────────────────
