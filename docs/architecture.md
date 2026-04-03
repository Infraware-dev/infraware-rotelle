
# Notes

Currently Rotelle is structured as a single binary run in in a kubernetes
deployment, later it could be built into a k8s operator which can then more
actively deploy, monitor test-apps that may function/malfunction in planned ways
according to the `rotelle-cases.md`

Right now the binary is loosely modeled as one "mode" mapped to one failure case.
This will likely change, but is currently simple and understandable in the
server.
