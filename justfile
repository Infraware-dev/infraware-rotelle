image := env("ROTELLE_IMAGE", "rotelle:dev")
musl  := "aarch64-unknown-linux-musl"

_default:
    @just --list

# Build the rotelle binary (musl) and Docker image
docker-build:
    cd rotelle && rustup target add {{musl}}
    cd rotelle && cargo build --release --target {{musl}}
    docker build -f docker/Dockerfile -t {{image}} .

# Apply k8s manifests and wait for rollout
deploy:
    kubectl apply -f k8s/rotelle.yaml
    kubectl rollout status deployment/rotelle -n rotelle --timeout=60s

# Full pipeline: build binary, build image, load into cluster, deploy
ship:
    scripts/load-rotelle.sh
