#!/usr/bin/env bash
set -euo pipefail

CLUSTER_NAME="${KIND_CLUSTER_NAME:-failer-test}"
KIND_VERSION="v0.27.0"
KUBECTL_MIN_VERSION="1.29"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info()    { echo -e "${GREEN}[INFO]${NC} $*"; }
warn()    { echo -e "${YELLOW}[WARN]${NC} $*"; }
error()   { echo -e "${RED}[ERROR]${NC} $*" >&2; }
die()     { error "$*"; exit 1; }

check_command() {
    local cmd="$1"
    local install_hint="$2"
    if ! command -v "$cmd" &>/dev/null; then
        die "'$cmd' not found. $install_hint"
    fi
}

check_prerequisites() {
    info "Checking prerequisites..."
    check_command docker   "Install Docker: https://docs.docker.com/get-docker/"
    check_command kind     "Install kind: https://kind.sigs.k8s.io/docs/user/quick-start/#installation"
    check_command kubectl  "Install kubectl: https://kubernetes.io/docs/tasks/tools/"

    if ! docker info &>/dev/null; then
        die "Docker daemon is not running. Please start Docker."
    fi

    info "All prerequisites satisfied."
}

cluster_exists() {
    kind get clusters 2>/dev/null | grep -qx "$CLUSTER_NAME"
}

create_cluster() {
    if cluster_exists; then
        warn "Cluster '$CLUSTER_NAME' already exists. Skipping creation."
        warn "To recreate: kind delete cluster --name $CLUSTER_NAME && $0"
        return 0
    fi

    info "Creating kind cluster '$CLUSTER_NAME'..."
    kind create cluster --name "$CLUSTER_NAME" --config - <<'EOF'
kind: Cluster
apiVersion: kind.x-k8s.io/v1alpha4
nodes:
  - role: control-plane
  - role: worker
  - role: worker
EOF
    info "Cluster '$CLUSTER_NAME' created."
}

set_kubeconfig() {
    info "Configuring kubectl context..."
    kind export kubeconfig --name "$CLUSTER_NAME"
    kubectl cluster-info --context "kind-${CLUSTER_NAME}"
}

wait_for_nodes() {
    info "Waiting for nodes to be Ready..."
    kubectl wait --for=condition=Ready nodes --all --timeout=120s
    info "All nodes are Ready."
}

main() {
    echo "================================================"
    echo " kind cluster setup: $CLUSTER_NAME"
    echo "================================================"

    check_prerequisites
    create_cluster
    set_kubeconfig
    wait_for_nodes

    echo ""
    info "Setup complete. Cluster '$CLUSTER_NAME' is ready."
    info "To load the failer image: ./scripts/load-failer.sh"
    info "To delete the cluster:    kind delete cluster --name $CLUSTER_NAME"
}

main "$@"
