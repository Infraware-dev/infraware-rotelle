#!/usr/bin/env bash
# rotectl.sh — send commands to the rotelle /rotectl/cmd endpoint
#
# Usage:
#   ./scripts/rotectl.sh [HOST] <case>
#
#   HOST  optional base URL (default: http://localhost:8080)
#   case  idle | c1
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

echo "POST $HOST/rotectl/cmd  $BODY"
curl -s -X POST "$HOST/rotectl/cmd" \
     -H 'Content-Type: application/json' \
     -d "$BODY" | cat
echo
