#!/usr/bin/env bash
# rotectl.sh — send commands to the rotelle /rotectl/cmd endpoint
#
# Usage:
#   ./scripts/rotectl.sh [HOST] <scenario>
#
#   HOST  optional base URL
#         default: http://localhost:8080  (local dev: `just run` runs full mode on 8080)
#         K8s:     http://localhost:9090  (control sidecar is port-forwarded on 9090)
#   scenario  idle | check | crash-loop | oom-kill | missing-env-var | service-unreachable | ingress-conflict
#
# Examples (local dev):
#   ./scripts/rotectl.sh idle
#   ./scripts/rotectl.sh crash-loop
#
# Examples (K8s — after: kubectl port-forward -n rotelle svc/rotelle-control 9090:9090 &):
#   ./scripts/rotectl.sh http://localhost:9090 crash-loop
#   ./scripts/rotectl.sh http://rotelle.example.com idle

set -euo pipefail

HOST="http://localhost:8080"

# If first arg looks like a URL, use it as the host
if [[ "${1:-}" == http* ]]; then
    HOST="${1%/}"
    shift
fi

SCENARIO="${1:-}"

case "$SCENARIO" in
    idle)
        BODY='{"cmd":"reset"}'
        ;;
    check)
        BODY='{"cmd":"check"}'
        ;;
    crash-loop)
        BODY='{"cmd":"set","scenario":"crash-loop"}'
        ;;
    oom-kill)
        # Optional env overrides: LOOP_TIME_SECS (default 10), LOOP_AMOUNT_MB (default 10)
        LOOP_TIME_SECS="${LOOP_TIME_SECS:-10}"
        LOOP_AMOUNT_MB="${LOOP_AMOUNT_MB:-10}"
        BODY="{\"cmd\":\"set\",\"scenario\":\"oom-kill\",\"loop_time_secs\":${LOOP_TIME_SECS},\"loop_amount_mb\":${LOOP_AMOUNT_MB}}"
        ;;
    missing-env-var)
        BODY='{"cmd":"set","scenario":"missing-env-var"}'
        ;;
    service-unreachable)
        BODY='{"cmd":"set","scenario":"service-unreachable"}'
        ;;
    ingress-conflict)
        BODY='{"cmd":"set","scenario":"ingress-conflict"}'
        ;;
    "")
        echo "Usage: $(basename "$0") [HOST] <scenario>" >&2
        echo "Scenarios: idle | check | crash-loop | oom-kill | missing-env-var | service-unreachable | ingress-conflict" >&2
        exit 1
        ;;
    *)
        echo "Unknown scenario: $SCENARIO" >&2
        echo "Scenarios: idle | check | crash-loop | oom-kill | missing-env-var | service-unreachable | ingress-conflict" >&2
        exit 1
        ;;
esac

echo "POST $HOST/rotectl/cmd  $BODY"
curl -s -X POST "$HOST/rotectl/cmd" \
     -H 'Content-Type: application/json' \
     -d "$BODY" | cat
echo
