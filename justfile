image := env("FAILER_IMAGE", "failer:dev")
musl  := "x86_64-unknown-linux-musl"

_default:
    @just --list

# Build the failer binary (musl) and Docker image
docker-build:
    cd failer && rustup target add {{musl}}
    cd failer && cargo build --release --target {{musl}}
    docker build -f docker/Dockerfile -t {{image}} .

# Apply k8s manifests and wait for rollout
deploy:
    kubectl apply -f k8s/failer.yaml
    kubectl rollout status deployment/failer -n failer --timeout=60s

# Full pipeline: build binary, build image, load into cluster, deploy
ship:
    scripts/load-failer.sh
