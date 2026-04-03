set dotenv-load

image := env("ROTELLE_IMAGE", "rotelle:dev")
rotelle_service_host := env("ROTELLE_SERVICE_HOST", "http://127.0.0.1:8080")
arch  := env("ROTELLE_MUSL_ARCH", `uname -m | sed 's/arm64/aarch64/'`)
musl  := arch + "-unknown-linux-musl"

_default:
    @just --list

# Apply k8s manifests and wait for rollout
deploy: docker-build
    kubectl apply -f k8s/rotelle.yaml
    kubectl rollout status deployment/rotelle -n rotelle --timeout=60s

# Delete using the k8s manifest (for a clean start)
undeploy:
    kubectl delete -f k8s/rotelle.yaml

# Restart pods to pick up a newly loaded image (imagePullPolicy: Never)
restart:
    kubectl rollout restart deployment/rotelle -n rotelle
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

# Run a single failure file against the rotelle service.
run-case FILE:
    hurl \
      --variable rotelle_service_host={{rotelle_service_host}} \
      tests/hurl-case/{{FILE}}.hurl

# Run hurl control-path test sequence against the rotelle service (future multitest sequence)
test-hurl:
    hurl \
      --test \
      --variable rotelle_service_host={{rotelle_service_host}} \
      tests/hurl-case/check.hurl

# Full pipeline: build binary, build image, load into cluster, deploy
ship:
    scripts/load-rotelle.sh
