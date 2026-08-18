# thread-exhaustion.hurl — activate thread-exhaustion concurrency-starvation mode
#
# Every GET / must claim one of max_concurrent slots and holds it for hold_ms.
# Sequentially each request just takes hold_ms; under concurrent load requests
# queue and latency grows with offered concurrency. Hurl is sequential, so the
# queueing itself is verified manually:
#
#   seq 20 | xargs -P 20 -I{} curl -s -o /dev/null -w '%{time_total}\n' \
#     http://localhost:8080/ | sort -n
#
# With hold_ms=500 max_concurrent=3, the 20 timings climb in ~0.5 s steps
# (0.50, 0.50, 0.50, 1.00, 1.00, 1.00, 1.50, …) — three served per window.
#
# Usage:
#   hurl --variable rotelle_service_host=<URL> tests/hurl-scenario/thread-exhaustion.hurl
#   just run-scenario thread-exhaustion

# ── activate thread-exhaustion ────────────────────────────────────────────────

POST {{rotelle_service_host}}/rotectl/cmd
Content-Type: application/json
{"cmd": "set", "scenario": "thread-exhaustion", "hold_ms": 500, "max_concurrent": 3}

HTTP 200
[Asserts]
jsonpath "$.ok" == true
jsonpath "$.scenario" == "thread-exhaustion"

GET {{rotelle_service_host}}/rotectl/status
HTTP 200
[Asserts]
jsonpath "$.scenario" == "thread-exhaustion"
jsonpath "$.hold_ms" == 500
jsonpath "$.max_concurrent" == 3
jsonpath "$.slots_in_use" == 0
jsonpath "$.total_requests" == 0

# ── a single request holds one slot for hold_ms, then responds 200 ────────────

GET {{rotelle_service_host}}/
HTTP 200
[Asserts]
body contains "thread-exhaustion"
duration >= 500

# ── the slot is released again; the request was counted ──────────────────────

GET {{rotelle_service_host}}/rotectl/status
HTTP 200
[Asserts]
jsonpath "$.slots_in_use" == 0
jsonpath "$.total_requests" == 1

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
duration < 500
