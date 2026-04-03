#!/usr/bin/env bash
# rotectl.sh — send commands to the rotelle /rotectl/cmd endpoint
#
# Usage:
#   ./scripts/rotectl.sh [HOST] <case>
#
#   HOST  optional base URL (default: http://localhost:8080)
#   case  idle | c1 | check | intermittent-01 | intermittent-02
#
# Examples:
#   ./scripts/rotectl.sh idle
#   ./scripts/rotectl.sh c1
#   ./scripts/rotectl.sh http://rotelle.example.com idle
#
# Equivalent curl commands:
#   curl -s -X POST http://localhost:8080/rotectl/cmd \
#        -H 'Content-Type: application/json' \
#        -d '{"cmd":"reset"}'
#
#   curl -s -X POST http://localhost:8080/rotectl/cmd \
#        -H 'Content-Type: application/json' \
#        -d '{"cmd":"set","case":"c1"}'

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
    intermittent-01)
        BODY='{"cmd":"set","case":"intermittent-01"}'
        ;;
    intermittent-02)
        # Optional env overrides: LOOP_TIME_SECS (default 10), LOOP_AMOUNT_MB (default 10)
        LOOP_TIME_SECS="${LOOP_TIME_SECS:-10}"
        LOOP_AMOUNT_MB="${LOOP_AMOUNT_MB:-10}"
        BODY="{\"cmd\":\"set\",\"case\":\"intermittent-02\",\"loop_time_secs\":${LOOP_TIME_SECS},\"loop_amount_mb\":${LOOP_AMOUNT_MB}}"
        ;;
    "")
        echo "Usage: $(basename "$0") [HOST] <case>" >&2
        echo "Cases: idle | c1 | check | intermittent-01 | intermittent-02" >&2
        exit 1
        ;;
    *)
        echo "Unknown case: $CASE" >&2
        echo "Cases: idle | c1 | check | intermittent-01 | intermittent-02" >&2
        exit 1
        ;;
esac

echo "POST $HOST/rotectl/cmd  $BODY"
curl -s -X POST "$HOST/rotectl/cmd" \
     -H 'Content-Type: application/json' \
     -d "$BODY" | cat
echo
