#!/usr/bin/env bash
# failctl.sh — send commands to the failer /failctl/cmd endpoint
#
# Usage:
#   ./scripts/failctl.sh [HOST] <case>
#
#   HOST  optional base URL (default: http://localhost:8080)
#   case  idle | c1
#
# Examples:
#   ./scripts/failctl.sh idle
#   ./scripts/failctl.sh c1
#   ./scripts/failctl.sh http://failer.example.com idle
#
# Equivalent curl commands:
#   curl -s -X POST http://localhost:8080/failctl/cmd \
#        -H 'Content-Type: application/json' \
#        -d '{"cmd":"reset"}'
#
#   curl -s -X POST http://localhost:8080/failctl/cmd \
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
    c1)
        BODY='{"cmd":"set","case":"c1"}'
        ;;
    "")
        echo "Usage: $(basename "$0") [HOST] <case>" >&2
        echo "Cases: idle | c1" >&2
        exit 1
        ;;
    *)
        echo "Unknown case: $CASE" >&2
        echo "Cases: idle | c1" >&2
        exit 1
        ;;
esac

echo "POST $HOST/failctl/cmd  $BODY"
curl -s -X POST "$HOST/failctl/cmd" \
     -H 'Content-Type: application/json' \
     -d "$BODY" | cat
echo
