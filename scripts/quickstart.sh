#!/usr/bin/env bash
# quickstart.sh — clone → cluster → deploy → first scenario in <10 minutes
#
# Usage:
#   ./scripts/quickstart.sh                  # use published GHCR image (default)
#   ./scripts/quickstart.sh --build-from-source  # build binary + image locally (requires Rust)
#   ./scripts/quickstart.sh --help
set -euo pipefail

# ── formatting ─────────────────────────────────────────────────────────────────
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m'; NC='\033[0m'

info()  { echo -e "${GREEN}[info]${NC}  $*"; }
step()  { echo -e "\n${BOLD}${CYAN}▶  $*${NC}"; }
warn()  { echo -e "${YELLOW}[warn]${NC}  $*"; }
die()   { echo -e "${RED}[error]${NC} $*" >&2; exit 1; }
hr()    { echo -e "${BOLD}────────────────────────────────────────${NC}"; }

# ── config ─────────────────────────────────────────────────────────────────────
BUILD_FROM_SOURCE=false
CLUSTER_NAME="${KIND_CLUSTER_NAME:-rotelle-test}"
GHCR_IMAGE="ghcr.io/infraware-dev/infraware-rotelle:latest"
LOCAL_IMAGE="rotelle:dev"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SIM_PORT=8080
CONTROL_PORT=9090
PF_SIM_PID=""
PF_CONTROL_PID=""
NAMESPACE=""
MANIFEST=""
CONTROL_MANIFEST=""
SIM_API_URL=""

# ── argument parsing ────────────────────────────────────────────────────────────
for arg in "$@"; do
  case "$arg" in
    --build-from-source) BUILD_FROM_SOURCE=true ;;
    --help|-h)
      echo "Usage: $0 [--build-from-source]"
      echo ""
      echo "  (default)            Pull the published image from GHCR."
      echo "                       Requires: docker, kind, kubectl, curl"
      echo ""
      echo "  --build-from-source  Build the binary and container image locally."
      echo "                       Requires: docker, kind, kubectl, curl, cargo, rustup"
      exit 0
      ;;
    *) die "Unknown argument: $arg. Run with --help for usage." ;;
  esac
done

# ── cleanup ─────────────────────────────────────────────────────────────────────
cleanup() {
  local exit_code=$?
  if [[ $exit_code -ne 0 ]]; then
    echo ""
    warn "Quickstart stopped early (exit $exit_code)."
    warn "Check the error above, then re-run: ./scripts/quickstart.sh"
    if ! $BUILD_FROM_SOURCE; then
      warn "If the failure was an image pull error, try: ./scripts/quickstart.sh --build-from-source"
    fi
  fi
  local pids=()
  [[ -n "$PF_SIM_PID" ]]     && kill -0 "$PF_SIM_PID"     2>/dev/null && pids+=("$PF_SIM_PID")
  [[ -n "$PF_CONTROL_PID" ]] && kill -0 "$PF_CONTROL_PID" 2>/dev/null && pids+=("$PF_CONTROL_PID")
  if [[ ${#pids[@]} -gt 0 ]]; then
    info "Port-forward loops still running (PIDs ${pids[*]})."
    info "To stop them: kill ${pids[*]}"
  fi
}
trap cleanup EXIT

# ── prerequisites ───────────────────────────────────────────────────────────────
require() {
  local cmd="$1" hint="$2"
  command -v "$cmd" &>/dev/null || die "'$cmd' not found. $hint"
}

check_prerequisites() {
  step "Checking prerequisites"
  require docker  "Install Docker: https://docs.docker.com/get-docker/"
  require kind    "Install kind:   https://kind.sigs.k8s.io/docs/user/quick-start/#installation"
  require kubectl "Install kubectl: https://kubernetes.io/docs/tasks/tools/"
  require curl    "Install curl via your system package manager"

  docker info &>/dev/null || die "Docker daemon is not running. Start Docker and try again."

  if $BUILD_FROM_SOURCE; then
    require cargo  "Install Rust: https://rustup.rs"
    require rustup "Install Rust: https://rustup.rs"
  fi

  info "All prerequisites satisfied."
}

# ── kind cluster ────────────────────────────────────────────────────────────────
setup_cluster() {
  step "Setting up Kind cluster '$CLUSTER_NAME'"

  if kind get clusters 2>/dev/null | grep -qx "$CLUSTER_NAME"; then
    warn "Cluster '$CLUSTER_NAME' already exists — reusing it."
    warn "To start fresh: kind delete cluster --name $CLUSTER_NAME"
  else
    info "Creating cluster (1 control-plane + 2 workers)…"
    kind create cluster --name "$CLUSTER_NAME" --config - <<'EOF'
kind: Cluster
apiVersion: kind.x-k8s.io/v1alpha4
nodes:
  - role: control-plane
  - role: worker
  - role: worker
EOF
    info "Cluster created."
  fi

  info "Exporting kubeconfig…"
  kind export kubeconfig --name "$CLUSTER_NAME"

  info "Waiting for nodes to be Ready…"
  kubectl wait --for=condition=Ready nodes --all --timeout=120s
  info "All nodes are Ready."
}

# ── image & manifest selection ──────────────────────────────────────────────────
prepare_image() {
  if $BUILD_FROM_SOURCE; then
    step "Building Rotelle from source"

    if kubectl get namespace rotelle &>/dev/null 2>&1; then
      warn "Namespace 'rotelle' already exists in this cluster (published-image deployment)."
      warn "This run will deploy to 'rotelle-dev' and port-forward there."
      warn "To remove the other deployment: kubectl delete namespace rotelle"
    fi

    local detected_arch musl_arch musl_target
    detected_arch="$(uname -m | sed 's/arm64/aarch64/')"
    musl_arch="${ROTELLE_MUSL_ARCH:-$detected_arch}"
    musl_target="${musl_arch}-unknown-linux-musl"

    if ! rustup target list --installed 2>/dev/null | grep -q "$musl_target"; then
      info "Adding rustup target $musl_target…"
      rustup target add "$musl_target"
    fi

    info "Compiling (target: $musl_target) — this takes ~1–2 minutes…"
    (cd "$REPO_ROOT/rotelle" && cargo build --release --target "$musl_target")

    info "Building Docker image '$LOCAL_IMAGE'…"
    docker build \
      --build-arg MUSL_TARGET="$musl_target" \
      -f "$REPO_ROOT/docker/Dockerfile" \
      -t "$LOCAL_IMAGE" \
      "$REPO_ROOT"

    info "Loading image into Kind cluster…"
    kind load docker-image "$LOCAL_IMAGE" --name "$CLUSTER_NAME"

    MANIFEST="$REPO_ROOT/k8s/rotelle-dev.yaml"
    CONTROL_MANIFEST="$REPO_ROOT/k8s/rotelle-control-dev.yaml"
    NAMESPACE="rotelle-dev"
    SIM_API_URL="http://rotelle.rotelle-dev.svc.cluster.local:${SIM_PORT}"
  else
    step "Using published image from GHCR"

    if kubectl get namespace rotelle-dev &>/dev/null 2>&1; then
      warn "Namespace 'rotelle-dev' already exists in this cluster (build-from-source deployment)."
      warn "This run will deploy to 'rotelle' and port-forward there."
      warn "To remove the other deployment: kubectl delete namespace rotelle-dev"
    fi

    info "Kind nodes will pull $GHCR_IMAGE directly from the registry."
    info "No local build needed."
    warn "This deploys the last published release — not your local source."
    warn "To run your local code instead: ./scripts/quickstart.sh --build-from-source"

    MANIFEST="$REPO_ROOT/k8s/rotelle.yaml"
    CONTROL_MANIFEST="$REPO_ROOT/k8s/rotelle-control.yaml"
    NAMESPACE="rotelle"
    SIM_API_URL="http://rotelle.rotelle.svc.cluster.local:${SIM_PORT}"
  fi
}

# ── deploy sim ──────────────────────────────────────────────────────────────────
deploy() {
  step "Deploying Rotelle (sim pod — namespace: $NAMESPACE)"
  info "Applying manifests…"
  kubectl apply -f "$MANIFEST"

  info "Waiting for rollout (up to 90 s)…"
  if ! kubectl rollout status deployment/rotelle -n "$NAMESPACE" --timeout=90s; then
    echo ""
    if kubectl get pods -n "$NAMESPACE" 2>/dev/null \
        | grep -qE "ImagePullBackOff|ErrImagePull"; then
      die "Pod failed to start: image pull error.
       The published GHCR image may not exist yet or the registry is unreachable.
       Try building from source instead:
         ./scripts/quickstart.sh --build-from-source"
    fi
    die "Rollout did not complete in time. Check pod status:
       kubectl get pods -n $NAMESPACE
       kubectl describe pod -n $NAMESPACE \$(kubectl get pods -n $NAMESPACE -o name | head -1)"
  fi
  info "Sim pod is running in namespace '$NAMESPACE'."
}

# ── deploy control pod ──────────────────────────────────────────────────────────
deploy_control_pod() {
  step "Deploying control pod (namespace: rotelle-system)"
  info "Applying manifest: $CONTROL_MANIFEST"
  kubectl apply -f "$CONTROL_MANIFEST"

  info "Waiting for control pod rollout (up to 60 s)…"
  kubectl rollout status deployment/rotelle-control -n rotelle-system --timeout=60s
  info "Control pod is running in namespace 'rotelle-system'."
}

# ── port-forwards ───────────────────────────────────────────────────────────────
start_port_forwards() {
  step "Opening port-forwards → localhost:$SIM_PORT (sim) and localhost:$CONTROL_PORT (control)"

  free_port_or_die() {
    local port="$1"
    local pids owner proc
    pids=$(lsof -iTCP:"$port" -sTCP:LISTEN -t 2>/dev/null) || return 0
    proc=$(lsof -iTCP:"$port" -sTCP:LISTEN 2>/dev/null | awk 'NR==2 {print $1}')
    if [[ "$proc" == "kubectl" ]]; then
      warn "Port $port held by a previous kubectl port-forward — cleaning it up."
      kill $pids 2>/dev/null || true
      sleep 1
    else
      owner=$(lsof -iTCP:"$port" -sTCP:LISTEN 2>/dev/null | awk 'NR==2 {print $1, "(PID "$2")"}')
      die "Port $port is already in use${owner:+ by $owner}. Run this to free it:
           kill \$(lsof -iTCP:$port -sTCP:LISTEN -t)"
    fi
  }

  free_port_or_die "$SIM_PORT"
  free_port_or_die "$CONTROL_PORT"

  # Run each port-forward in a loop so it reconnects automatically when the sim
  # pod restarts (e.g. after a crash-loop or OOM-kill scenario).
  (
    trap 'kill $(jobs -p) 2>/dev/null' EXIT
    while true; do
      kubectl port-forward -n "$NAMESPACE" svc/rotelle "${SIM_PORT}:${SIM_PORT}" >/dev/null 2>&1 || true
      sleep 1
    done
  ) &
  PF_SIM_PID=$!

  (
    trap 'kill $(jobs -p) 2>/dev/null' EXIT
    while true; do
      kubectl port-forward -n rotelle-system svc/rotelle-control "${CONTROL_PORT}:${CONTROL_PORT}" >/dev/null 2>&1 || true
      sleep 1
    done
  ) &
  PF_CONTROL_PID=$!

  # Wait for the control pod to become ready — it doesn't crash so this is reliable.
  local ready=false
  for _ in 1 2 3 4 5; do
    sleep 1
    if curl -sf "http://localhost:${CONTROL_PORT}/rotectl/health" &>/dev/null; then
      ready=true
      break
    fi
  done

  if ! $ready; then
    warn "Port-forwards may not be ready yet. If the demo fails, run manually:"
    warn "  kubectl port-forward -n $NAMESPACE svc/rotelle ${SIM_PORT}:${SIM_PORT} &"
    warn "  kubectl port-forward -n rotelle-system svc/rotelle-control ${CONTROL_PORT}:${CONTROL_PORT} &"
  else
    info "Sim surface  → http://localhost:${SIM_PORT}"
    info "Control pod  → http://localhost:${CONTROL_PORT}/rotectl/control"
  fi
}

# ── demo ────────────────────────────────────────────────────────────────────────
run_demo() {
  step "Running your first failure scenario"
  echo ""
  echo "  The control pod (port ${CONTROL_PORT}) proxies commands to the sim pod (port ${SIM_PORT})."
  echo "  Let's activate a scenario and observe both."
  echo ""

  hr
  echo -e "  ${CYAN}1. Check current status via control pod — should be idle:${NC}"
  hr
  echo -n "  \$ curl http://localhost:${CONTROL_PORT}/rotectl/status"
  echo ""
  curl -sf "http://localhost:${CONTROL_PORT}/rotectl/status" && echo
  echo ""

  hr
  echo -e "  ${CYAN}2. Activate 'crash-loop' via control pod — crash every 5th request:${NC}"
  hr
  echo "  \$ curl -X POST http://localhost:${CONTROL_PORT}/rotectl/cmd \\"
  echo "      -H 'Content-Type: application/json' \\"
  echo "      -d '{\"cmd\": \"set\", \"scenario\": \"crash-loop\"}'"
  echo ""
  curl -sf -X POST "http://localhost:${CONTROL_PORT}/rotectl/cmd" \
    -H 'Content-Type: application/json' \
    -d '{"cmd": "set", "scenario": "crash-loop"}' && echo
  echo ""

  hr
  echo -e "  ${CYAN}3. Drive 4 requests to the sim pod — the 5th would crash it:${NC}"
  hr
  for i in 1 2 3 4; do
    echo -n "  Request $i: "
    curl -sf --max-time 5 "http://localhost:${SIM_PORT}/" \
      | grep -o '<title>[^<]*</title>' \
      | sed 's/<[^>]*>//g' \
      || echo "(no title)"
    sleep 0.5
  done
  echo ""

  hr
  echo -e "  ${CYAN}4. Status via control pod — shows access_count and crash_every:${NC}"
  hr
  curl -sf "http://localhost:${CONTROL_PORT}/rotectl/status" && echo
  echo ""

  hr
  echo -e "  ${CYAN}5. Reset via control pod:${NC}"
  hr
  curl -sf -X POST "http://localhost:${CONTROL_PORT}/rotectl/cmd" \
    -H 'Content-Type: application/json' \
    -d '{"cmd": "reset"}' && echo
  echo ""
}

# ── summary ──────────────────────────────────────────────────────────────────────
print_summary() {
  echo ""
  echo -e "${BOLD}${GREEN}╔══════════════════════════════════════════╗${NC}"
  echo -e "${BOLD}${GREEN}║   Rotelle is ready — cluster is running  ║${NC}"
  echo -e "${BOLD}${GREEN}╚══════════════════════════════════════════╝${NC}"
  echo ""
  local running_image
  running_image=$(kubectl get pod -n "$NAMESPACE" -o jsonpath='{.items[0].spec.containers[0].image}' 2>/dev/null || echo "unknown")
  echo -e "  Sim surface  →  ${CYAN}http://localhost:${SIM_PORT}${NC}                  (namespace: $NAMESPACE)"
  echo -e "  Control panel→  ${CYAN}http://localhost:${CONTROL_PORT}/rotectl/control${NC}   (namespace: rotelle-system)"
  echo -e "  Control API  →  ${CYAN}http://localhost:${CONTROL_PORT}/rotectl/status${NC}    (namespace: rotelle-system)"
  echo -e "  Running image→  ${CYAN}${running_image}${NC}"
  echo -e "  Next steps   →  ${CYAN}ONBOARDING.md${NC}"
  echo -e "  All scenarios →  ${CYAN}docs/scenarios.md${NC}"
  echo ""
  echo -e "  Port-forwards running in background (PIDs $PF_SIM_PID $PF_CONTROL_PID)."
  echo -e "  To stop:      ${YELLOW}kill $PF_SIM_PID $PF_CONTROL_PID${NC}"
  echo -e "  To tear down: ${YELLOW}kind delete cluster --name $CLUSTER_NAME${NC}"
  echo ""
}

# ── main ─────────────────────────────────────────────────────────────────────────
main() {
  echo ""
  echo -e "${BOLD}╔══════════════════════════════════════════╗${NC}"
  echo -e "${BOLD}║          Rotelle  ·  Quickstart          ║${NC}"
  if $BUILD_FROM_SOURCE; then
    echo -e "${BOLD}║          mode: build from source         ║${NC}"
  else
    echo -e "${BOLD}║          mode: published GHCR image      ║${NC}"
  fi
  echo -e "${BOLD}╚══════════════════════════════════════════╝${NC}"
  echo ""

  check_prerequisites
  setup_cluster
  prepare_image
  deploy
  deploy_control_pod
  start_port_forwards
  run_demo
  print_summary
}

main "$@"
