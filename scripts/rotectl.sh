#!/usr/bin/env bash
# rotectl.sh — send commands to the rotelle /rotectl/cmd endpoint
#
# Usage:
#   ./scripts/rotectl.sh [HOST] <case>
#
#   HOST  optional base URL (default: http://localhost:8080)
#   case  idle | check | crash-loop | oom-kill | missing-env-var | service-unreachable | ingress-conflict
#
# Examples:
#   ./scripts/rotectl.sh idle
#   ./scripts/rotectl.sh crash-loop
#   ./scripts/rotectl.sh http://rotelle.example.com idle
#
# Equivalent curl commands:
#   curl -s -X POST http://localhost:8080/rotectl/cmd \
#        -H 'Content-Type: application/json' \
#        -d '{"cmd":"reset"}'
#
#   curl -s -X POST http://localhost:8080/rotectl/cmd \
#        -H 'Content-Type: application/json' \
#        -d '{"cmd":"set","case":"crash-loop"}'

set -euo pipefail

HOST="http://localhost:8080"

# If first arg looks like a URL, use it as the host
if [[ "${1:-}" == http* ]]; then
    HOST="${1%/}"
    shift
fi

CASE="${1:-}"

case "$CASE" in
    idle)
        BODY='{"cmd":"reset"}'
        ;;
    check)
        BODY='{"cmd":"check"}'
        ;;
    crash-loop)
        BODY='{"cmd":"set","case":"crash-loop"}'
        ;;
    oom-kill)
        # Optional env overrides: LOOP_TIME_SECS (default 10), LOOP_AMOUNT_MB (default 10)
        LOOP_TIME_SECS="${LOOP_TIME_SECS:-10}"
        LOOP_AMOUNT_MB="${LOOP_AMOUNT_MB:-10}"
        BODY="{\"cmd\":\"set\",\"case\":\"oom-kill\",\"loop_time_secs\":${LOOP_TIME_SECS},\"loop_amount_mb\":${LOOP_AMOUNT_MB}}"
        ;;
    missing-env-var)
        BODY='{"cmd":"set","case":"missing-env-var"}'
        ;;
    service-unreachable)
        BODY='{"cmd":"set","case":"service-unreachable"}'
        ;;
    ingress-conflict)
        BODY='{"cmd":"set","case":"ingress-conflict"}'
        ;;
    "")
        echo "Usage: $(basename "$0") [HOST] <case>" >&2
        echo "Cases: idle | check | crash-loop | oom-kill | missing-env-var | service-unreachable | ingress-conflict" >&2
        exit 1
        ;;
    *)
        echo "Unknown case: $CASE" >&2
        echo "Cases: idle | check | crash-loop | oom-kill | missing-env-var | service-unreachable | ingress-conflict" >&2
        exit 1
        ;;
esac

echo "POST $HOST/rotectl/cmd  $BODY"
curl -s -X POST "$HOST/rotectl/cmd" \
     -H 'Content-Type: application/json' \
     -d "$BODY" | cat
echo
