set dotenv-load

image := env("ROTELLE_IMAGE", "rotelle:dev")
rotelle_service_host := env("ROTELLE_SERVICE_HOST", "http://127.0.0.1:8080")
arch  := env("ROTELLE_MUSL_ARCH", `uname -m | sed 's/arm64/aarch64/'`)
musl  := arch + "-unknown-linux-musl"

_default:
    @just --list

# Apply k8s manifests (sim + control pod) and wait for rollout
deploy: docker-build
    kubectl apply -f k8s/rotelle-dev.yaml
    kubectl rollout status deployment/rotelle -n rotelle-dev --timeout=60s
    kubectl apply -f k8s/rotelle-control-dev.yaml
    kubectl rollout status deployment/rotelle-control -n rotelle-system --timeout=60s

# Delete both sim and control pod manifests
undeploy:
    kubectl delete -f k8s/rotelle-dev.yaml --ignore-not-found
    kubectl delete -f k8s/rotelle-control-dev.yaml --ignore-not-found

# Deploy release manifests (sim + control pod)
deploy-rel:
    kubectl apply -f k8s/rotelle.yaml
    kubectl rollout status deployment/rotelle -n rotelle --timeout=60s
    kubectl apply -f k8s/rotelle-control.yaml
    kubectl rollout status deployment/rotelle-control -n rotelle-system --timeout=60s

# Delete release manifests
undeploy-rel:
    kubectl delete -f k8s/rotelle.yaml --ignore-not-found
    kubectl delete -f k8s/rotelle-control.yaml --ignore-not-found

# Restart sim and control pods to pick up a newly loaded image
restart:
    kubectl rollout restart deployment/rotelle -n rotelle-dev
    kubectl rollout status deployment/rotelle -n rotelle-dev --timeout=60s
    kubectl rollout restart deployment/rotelle-control -n rotelle-system
    kubectl rollout status deployment/rotelle-control -n rotelle-system --timeout=60s

# Restart release pods
restart-rel:
    kubectl rollout restart deployment/rotelle -n rotelle
    kubectl rollout status deployment/rotelle -n rotelle --timeout=60s
    kubectl rollout restart deployment/rotelle-control -n rotelle-system
    kubectl rollout status deployment/rotelle-control -n rotelle-system --timeout=60s

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

# Run the hurl test for a single scenario (pass the scenario name, not the file path).
run-scenario FILE:
    hurl \
      --variable rotelle_service_host={{rotelle_service_host}} \
      tests/hurl-scenario/{{FILE}}.hurl

# Run the control-path smoke test (check.hurl). For per-scenario tests use run-scenario.
test-hurl:
    hurl \
      --test \
      --variable rotelle_service_host={{rotelle_service_host}} \
      tests/hurl-scenario/check.hurl

# Full pipeline: build binary, build image, load into cluster, deploy.
# NOTE: deploys to the 'rotelle' namespace (production-style manifest, not rotelle-dev).
# For local dev iteration use `just deploy` instead.
ship:
    scripts/load-rotelle.sh
