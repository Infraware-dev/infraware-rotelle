# missing-env-var.hurl — control path verification
#
# Tests activation and status. The crash behaviour (CrashLoopBackOff) only
# triggers when REQUIRED_APP_SECRET is absent from the pod environment — run
# this test with that variable unset to observe the actual exit.
#
# Usage:
#   hurl --variable rotelle_service_host=<URL> tests/hurl-scenario/missing-env-var.hurl
#   just run-scenario missing-env-var

# ── activate ──────────────────────────────────────────────────────────────────

POST {{rotelle_service_host}}/rotectl/cmd
Content-Type: application/json
{"cmd": "set", "scenario": "missing-env-var"}

HTTP 200
[Asserts]
jsonpath "$.ok" == true
jsonpath "$.scenario" == "missing-env-var"

# ── status reflects the required var ─────────────────────────────────────────

GET {{rotelle_service_host}}/rotectl/status
HTTP 200
[Asserts]
jsonpath "$.scenario" == "missing-env-var"
jsonpath "$.required_var" == "REQUIRED_APP_SECRET"

# ── activate with a custom var name ──────────────────────────────────────────

POST {{rotelle_service_host}}/rotectl/cmd
Content-Type: application/json
{"cmd": "set", "scenario": "missing-env-var", "required_var": "DATABASE_URL"}

HTTP 200
[Asserts]
jsonpath "$.ok" == true

GET {{rotelle_service_host}}/rotectl/status
HTTP 200
[Asserts]
jsonpath "$.required_var" == "DATABASE_URL"

# ── reset ─────────────────────────────────────────────────────────────────────

POST {{rotelle_service_host}}/rotectl/cmd
Content-Type: application/json
{"cmd": "reset"}

HTTP 200
[Asserts]
jsonpath "$.ok" == true
jsonpath "$.scenario" == "none, idle"
