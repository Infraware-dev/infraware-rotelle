# Getting Help

## Documentation

Start here before opening an issue:

- **[README.md](README.md)** — Quick start, scenario overview, control API reference
- **[docs/rotelle-cases.md](docs/rotelle-cases.md)** — Full parameter docs for every scenario
- **[docs/external-tester-guide.md](docs/external-tester-guide.md)** — Step-by-step walkthrough for first-time testers
- **[docs/architecture.md](docs/architecture.md)** — Internal module layout and request flow
- **[CONTRIBUTING.md](CONTRIBUTING.md)** — Development setup and how to add scenarios

## Asking a Question

If the documentation doesn't answer your question:

1. **Search existing issues** — someone may have already asked: [github.com/infraware-dev/infraware-rotelle/issues](https://github.com/infraware-dev/infraware-rotelle/issues)
2. **Open a new issue** using the [Bug Report](https://github.com/infraware-dev/infraware-rotelle/issues/new?template=bug_report.yml) or [Feature Request](https://github.com/infraware-dev/infraware-rotelle/issues/new?template=feature_request.yml) template.
3. **Open a new issue** if your question doesn't fit the existing templates — the maintainers monitor all issues.

## Reporting a Bug

Use the [Bug Report template](https://github.com/infraware-dev/infraware-rotelle/issues/new?template=bug_report.yml). The more detail you include (cluster provider, version, reproduction steps, logs), the faster we can help.

## Security Issues

**Do not open public issues for security vulnerabilities.** Contact the maintainers directly instead.

## Common Issues

**Port-forward drops after a pod restart**
Expected — the port-forward is tied to the pod. Restart it: `kubectl port-forward -n rotelle svc/rotelle 8080:8080`

**`intermittent-02` never OOMKills**
Check that memory limits are applied: `kubectl describe pod -n rotelle <pod> | grep -A5 Limits`

**`cargo build` fails with target not found**
Add the musl target: `rustup target add x86_64-unknown-linux-musl` (Intel/AMD) or `aarch64-unknown-linux-musl` (Apple Silicon / ARM64)

**kind image load fails**
Ensure you're using the right cluster name: `kind get clusters`, then `kind load docker-image rotelle:dev --name <cluster-name>`
