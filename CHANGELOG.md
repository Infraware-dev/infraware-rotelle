# Changelog

All notable changes to Rotelle are documented here.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).
Versions follow [Semantic Versioning](https://semver.org/).

<!-- Maintained manually. Add a new entry as part of the release checklist in RELEASING.md. -->

## [0.1.3-1] - 2026-07-27

### Changed

- chore: update helm chart with otel k8s events collector fix

## [0.1.3] - 2026-07-24

### Added

- feat: add slow-response scenario
- feat: package helm charts in GHCR publish workflow
- feat: add helm/rotelle chart
- feat: add helm/observability-gp-mini chart

### Fixed

- fix: helm observability simplify

## [0.1.2] - 2026-07-22

(Skipped — folded into 0.1.3.)

## [0.1.1] - 2026-07-22

### Changed

- refactor: restyle control and status pages to a dark monochrome theme

## [0.1.0] - 2026-05-18

### Added

- Initial release
- Pluggable `Scenario` trait with automatic registry wiring
- Scenarios: `idle`, `check`, `crash-loop`, `oom-kill`, `missing-env-var`, `service-unreachable`, `ingress-conflict`
- Control API: `POST /rotectl/cmd`, `GET /rotectl/status`, `GET /rotectl/health`
- State persistence across pod restarts via `/data/state.json`
- Multi-arch Docker image (amd64 + arm64) published to GHCR
