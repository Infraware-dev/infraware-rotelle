# intro

Infra-Rotelle (di supporto) is a repository for a kubernetes app that can used
to simulate various app failure conditions. It can be used for training people
and AIs for working with kubernetes operations.

The core app is Rotelle, written as a rust web server.  The server can be
installed into a test kubernetes cluster and used to trigger various failure
conditions.

The support files around Rotelle can also be used to simulate installation and
configuration failures.

The web server conttains endpoints to control failure conditions:

## Control Endpoints
- rotectl/cmd
- rotectl/health
- rotectl/status
- rotectl/TBD...

## Simulation endpoints

Some endpoints of Rotelle are maintained as the simulated failing app

- /health


# Operation Notes

`kubectl` is the primary interface to the cluster. Any cluster reachable via
`kubectl cluster-info` is supported — the test tooling does not care how the
cluster was provisioned.

`scripts/setup-kind.sh` is provided as a reference setup using
[kind](https://kind.sigs.k8s.io). Future scripts (`setup-colima.sh`,
`setup-minikube.sh`, etc.) can be added alongside it following the same
convention: provision the cluster, configure the kubectl context, verify with
`kubectl cluster-info`.

### Local cluster options

**macOS:** [OrbStack](https://orbstack.dev) (recommended — fast, low overhead, built-in k8s) · [Docker Desktop](https://www.docker.com/products/docker-desktop/) · [Rancher Desktop](https://rancherdesktop.io) · [colima](https://github.com/abiosoft/colima) · [kind](https://kind.sigs.k8s.io)

**Linux:** [kind](https://kind.sigs.k8s.io) (recommended, used by `setup-kind.sh`) · [k3d](https://k3d.io) · [minikube](https://minikube.sigs.k8s.io) · [k3s](https://k3s.io) · [colima](https://github.com/abiosoft/colima)

Once a cluster is available, `scripts/load-rotelle.sh` handles the rest:
building the binary, packaging the image, loading it into the cluster, and
applying the manifests in `k8s/`.

## Cluster Image Loading

The main practical difference between cluster providers is **how a locally
built container image reaches the cluster nodes**.

| Provider | Mechanism | Notes |
|---|---|---|
| **kind** | `kind load docker-image` | kind nodes are isolated containers with their own image store, separate from the host Docker daemon. The image must be explicitly loaded. |
| **colima** | none needed | colima's k8s runs inside the same VM that hosts the Docker daemon. An image built with `docker build` is immediately visible to the cluster. |
| **Docker Desktop** | none needed | Same as colima — shared daemon. |
| **Rancher Desktop** | none needed | Same as colima — shared daemon. |
| **minikube** | `minikube image load` | minikube manages its own image cache; `minikube image load` transfers from the host daemon. |
| **remote cluster** | push to registry | The cluster cannot reach the host daemon. Build, tag, and push to a registry the cluster can pull from, then update the image reference in `k8s/rotelle.yaml`. |

`scripts/load-rotelle.sh` detects the active kubectl context name and selects
the right strategy automatically for the providers listed above.

raw yml kubectl specs should be used to load the app
with a frontend default loadbalancer (or TBD Helm, but more direct control is preferred)

Curl and/or hurl can be used to add scripts or just targets to help
access and control the rotelle interface

# Failure cases

see `docs/rotelle-cases.md`



