#!/usr/bin/env bash
set -euo pipefail

IMAGE_NAME="${FAILER_IMAGE:-failer:dev}"
MUSL_TARGET="x86_64-unknown-linux-musl"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

info()  { echo -e "${GREEN}[INFO]${NC} $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC} $*"; }
die()   { echo -e "${RED}[ERROR]${NC} $*" >&2; exit 1; }

check_prerequisites() {
    for cmd in cargo docker kubectl; do
        command -v "$cmd" &>/dev/null || die "'$cmd' not found."
    done

    kubectl cluster-info &>/dev/null || \
        die "kubectl cannot reach a cluster. Configure your kubeconfig and try again."
}

ensure_musl_target() {
    if ! rustup target list --installed 2>/dev/null | grep -q "$MUSL_TARGET"; then
        info "Adding rustup target $MUSL_TARGET..."
        rustup target add "$MUSL_TARGET"
    fi
}

build_binary() {
    info "Building failer binary (target: $MUSL_TARGET)..."
    (cd "$REPO_ROOT/failer" && cargo build --release --target "$MUSL_TARGET")
}

build_image() {
    info "Building Docker image '$IMAGE_NAME'..."
    docker build \
        -f "$REPO_ROOT/docker/Dockerfile" \
        -t "$IMAGE_NAME" \
        "$REPO_ROOT"
}

# Detect cluster type from the current kubectl context name and load the image
# using the appropriate method for that provider.
#
# kind:                needs `kind load docker-image` because kind nodes are
#                      isolated containers with their own image store.
# colima / docker-desktop / rancher-desktop:
#                      k8s runs inside the same Docker daemon the host uses,
#                      so the image is already visible after `docker build`.
# minikube:            supports `minikube image load` when available; falls
#                      back to the shared-daemon assumption otherwise.
# unknown / remote:    warn and skip — the user must push to a registry.
load_image() {
    local context
    context="$(kubectl config current-context 2>/dev/null || echo "")"
    info "Current kubectl context: ${context:-<none>}"

    if [[ "$context" == kind-* ]]; then
        local cluster_name="${context#kind-}"
        info "kind cluster detected. Loading image via kind..."
        kind load docker-image "$IMAGE_NAME" --name "$cluster_name"

    elif [[ "$context" == "colima" || "$context" == "docker-desktop" || "$context" == "rancher-desktop" ]]; then
        info "Local cluster ($context) shares the host Docker daemon — no image load needed."

    elif [[ "$context" == "minikube" ]]; then
        if command -v minikube &>/dev/null; then
            info "minikube detected. Loading image via minikube..."
            minikube image load "$IMAGE_NAME"
        else
            info "minikube context but minikube CLI not found — assuming shared daemon."
        fi

    else
        warn "Unknown context '$context'. Assuming the cluster can pull '$IMAGE_NAME'."
        warn "If not, push the image to a registry your cluster can access."
    fi
}

deploy() {
    local manifest="$REPO_ROOT/k8s/failer.yaml"
    info "Applying manifests..."
    kubectl apply -f "$manifest"

    info "Waiting for failer deployment to roll out..."
    kubectl rollout status deployment/failer -n failer --timeout=60s
}

main() {
    echo "================================================"
    echo " Build and deploy failer"
    echo "================================================"

    check_prerequisites
    ensure_musl_target
    build_binary
    build_image
    load_image
    deploy

    echo ""
    info "Done. failer deployed to namespace 'failer'."
    info "  kubectl port-forward -n failer svc/failer 8080:8080"
}

main "$@"
