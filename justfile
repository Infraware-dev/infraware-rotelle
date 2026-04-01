set dotenv-load

image := env("ROTELLE_IMAGE", "rotelle:dev")
arch  := env("ROTELLE_MUSL_ARCH", `uname -m | sed 's/arm64/aarch64/'`)
musl  := arch + "-unknown-linux-musl"

_default:
    @just --list

# Apply k8s manifests and wait for rollout
deploy:
    kubectl apply -f k8s/rotelle.yaml
    kubectl rollout status deployment/rotelle -n rotelle --timeout=60s

# Build the rotelle binary (musl) and Docker image
docker-build:
    cd rotelle && rustup target add {{musl}}
    cd rotelle && cargo build --release --target {{musl}}

    docker build -f docker/Dockerfile --build-arg MUSL_TARGET={{musl}} -t {{image}} .

# Report on the detected config of this system
config:
    @echo "image {{image}}"
    @echo "arch  {{arch}}"
    @echo "musl  {{musl}}"

# Run rotelle directly (no k8s) for quick local testing
run:
    mkdir -p tmp/data
    DATA_DIR=./tmp/data cargo run --manifest-path rotelle/Cargo.toml

# Full pipeline: build binary, build image, load into cluster, deploy
ship:
    scripts/load-rotelle.sh
