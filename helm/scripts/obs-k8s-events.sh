#!/usr/bin/env bash
# Query gigapipe's Loki-compatible API for logs — defaults to the k8s
# events pipeline (otel-gateway's k8s_events receiver), but works for any
# LogQL selector via -q, so it's also handy for eyeballing regular
# container logs (e.g. -q '{k8s_pod_name="rotelle-..."}').
#
# Talks to gigapipe over a kubectl port-forward, reusing one that's already
# running on the target port if present, otherwise starting (and cleaning
# up) its own.
set -euo pipefail

NAMESPACE="observability"
SERVICE="gigapipe"
LOCAL_PORT="3100"
QUERY='{k8s_event_reason=~".+"}'
SINCE="1h"
LIMIT="50"
RAW=0

usage() {
  cat <<EOF
Usage: $(basename "$0") [-q LOGQL_QUERY] [-s SINCE] [-l LIMIT] [-n NAMESPACE] [-p LOCAL_PORT] [-r]

  -q  LogQL query (default: '$QUERY')
  -s  Lookback window, e.g. 10m, 1h, 2d (default: $SINCE)
  -l  Max lines to return (default: $LIMIT)
  -n  Namespace gigapipe runs in (default: $NAMESPACE)
  -p  Local port to use for the port-forward (default: $LOCAL_PORT)
  -r  Raw JSON output (skip pretty-printing/formatting)
  -h  Show this help

Examples:
  # k8s events from the last hour (default)
  $(basename "$0")

  # pod restarts/backoffs specifically, last 6 hours
  $(basename "$0") -q '{k8s_event_reason=~".+"} |= "BackOff"' -s 6h

  # regular container logs for a pod
  $(basename "$0") -q '{k8s_pod_name="rotelle-5449997cd5-md8mh"}'
EOF
}

while getopts "q:s:l:n:p:rh" opt; do
  case "$opt" in
    q) QUERY="$OPTARG" ;;
    s) SINCE="$OPTARG" ;;
    l) LIMIT="$OPTARG" ;;
    n) NAMESPACE="$OPTARG" ;;
    p) LOCAL_PORT="$OPTARG" ;;
    r) RAW=1 ;;
    h) usage; exit 0 ;;
    *) usage; exit 1 ;;
  esac
done

since_to_seconds() {
  local s="$1" num unit
  num="${s%[smhd]}"
  unit="${s: -1}"
  case "$unit" in
    s) echo "$((num))" ;;
    m) echo "$((num * 60))" ;;
    h) echo "$((num * 3600))" ;;
    d) echo "$((num * 86400))" ;;
    *) echo "error: SINCE must end in s/m/h/d (got '$s')" >&2; exit 1 ;;
  esac
}

PORT_FORWARD_PID=""
cleanup() {
  if [[ -n "$PORT_FORWARD_PID" ]]; then
    kill "$PORT_FORWARD_PID" 2>/dev/null || true
    wait "$PORT_FORWARD_PID" 2>/dev/null || true
  fi
}
trap cleanup EXIT

# Reuse an existing port-forward on this port if there's one already
# serving gigapipe; otherwise start our own for the duration of this script.
if ! curl -s -o /dev/null --max-time 1 "http://localhost:${LOCAL_PORT}/loki/api/v1/labels"; then
  kubectl port-forward -n "$NAMESPACE" "svc/${SERVICE}" "${LOCAL_PORT}:3100" >/dev/null 2>&1 &
  PORT_FORWARD_PID=$!
  for _ in $(seq 1 20); do
    curl -s -o /dev/null --max-time 1 "http://localhost:${LOCAL_PORT}/loki/api/v1/labels" && break
    sleep 0.25
  done
fi

END_NS="$(($(date +%s) * 1000000000))"
START_NS="$(( (($(date +%s) - $(since_to_seconds "$SINCE"))) * 1000000000 ))"

RESPONSE="$(curl -s -G "http://localhost:${LOCAL_PORT}/loki/api/v1/query_range" \
  --data-urlencode "query=${QUERY}" \
  --data-urlencode "start=${START_NS}" \
  --data-urlencode "end=${END_NS}" \
  --data-urlencode "limit=${LIMIT}")"

if [[ "$RAW" -eq 1 ]]; then
  printf '%s\n' "$RESPONSE"
elif command -v jq >/dev/null 2>&1; then
  printf '%s\n' "$RESPONSE" | jq -r '
    .data.result[]? as $stream
    | $stream.values[]?
    | [(.[0] | tonumber / 1000000000 | strftime("%Y-%m-%dT%H:%M:%SZ")), $stream.stream.k8s_namespace_name, $stream.stream.k8s_object_kind, $stream.stream.k8s_object_name, .[1]]
    | @tsv
  ' 2>/dev/null || printf '%s\n' "$RESPONSE" | jq .
else
  printf '%s\n' "$RESPONSE" | python3 -m json.tool
fi
