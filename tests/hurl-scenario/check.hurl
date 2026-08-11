# check.hurl — control path verification sequence
#
# Usage:
#   hurl --variable rotelle_service_host=<URL> tests/hurl-scenario/check.hurl
#   just test-hurl

# ── reset to idle ─────────────────────────────────────────────────────────────

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
body contains "<title>Rotelle &mdash; Status</title>"
body contains "src=\"/rotelle-mark.png\""
body contains "open-source chaos lab by <a href=\"https://infraware.dev\">infraware.dev</a>"

GET {{rotelle_service_host}}/rotectl/control
HTTP 200
[Asserts]
body contains "<title>Rotelle &mdash; Control</title>"
body contains "role=\"listbox\""

GET {{rotelle_service_host}}/rotelle-mark.png
HTTP 200
[Asserts]
header "content-type" == "image/png"

# ── set a scenario ────────────────────────────────────────────────────────────

POST {{rotelle_service_host}}/rotectl/cmd
Content-Type: application/json
{"cmd": "set", "scenario": "ingress-conflict"}

HTTP 200
[Asserts]
jsonpath "$.ok" == true
jsonpath "$.scenario" == "ingress-conflict"

GET {{rotelle_service_host}}/rotectl/status
HTTP 200
[Asserts]
jsonpath "$.scenario" == "ingress-conflict"

# ── set check scenario ────────────────────────────────────────────────────────

POST {{rotelle_service_host}}/rotectl/cmd
Content-Type: application/json
{"cmd": "check"}

HTTP 200
[Asserts]
jsonpath "$.ok" == true
jsonpath "$.scenario" == "check"

GET {{rotelle_service_host}}/
HTTP 200
[Asserts]
body contains "check"

GET {{rotelle_service_host}}/rotectl/status
HTTP 200
[Asserts]
jsonpath "$.scenario" == "check"

# ── reset back to idle ────────────────────────────────────────────────────────

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
