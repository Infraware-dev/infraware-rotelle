# Release Process

This document is for maintainers.

---

## Versioning

Rotelle follows [Semantic Versioning](https://semver.org/):

- **Patch** (`v0.1.1`) — bug fixes, documentation corrections, dependency bumps
- **Minor** (`v0.2.0`) — new scenarios, new control API features, backwards-compatible changes
- **Major** (`v1.0.0`) — breaking changes to the control API or scenario interface

---

## Release checklist

**1. Confirm main is ready**

```sh
git checkout main
git pull upstream main
cargo test                          # must pass
just test-hurl                      # must pass (server running in terminal 1)
```

**2. Update the version in `rotelle/Cargo.toml`**

```toml
[package]
version = "0.2.0"   # ← bump this
```

**3. Update `rotelle/CHANGELOG.md`**

Add an entry for the new version. Follow the existing format:

```markdown
## [0.2.0] - YYYY-MM-DD

### Added
- feat: ...

### Fixed
- fix: ...
```

**4. Commit and push the version bump**

```sh
git add rotelle/Cargo.toml rotelle/CHANGELOG.md
git commit -m "chore: release v0.2.0"
git push upstream main
```

> This pushes directly to `main` and requires maintainer/admin access to the repository. GitHub's branch protection is configured to allow admin bypass for exactly this use case. If you cannot push directly, open a short PR titled `chore: release v0.2.0` and merge it first.

**5. Tag the release**

Pushing the tag triggers the publish workflow (multi-arch build → GHCR push → GitHub Release creation).

```sh
git tag v0.2.0
git push upstream v0.2.0
```

**6. Watch the publish workflow**

Go to [Actions → Publish to GHCR](https://github.com/infraware-dev/infraware-rotelle/actions/workflows/publish.yml) and confirm the `build`, `merge`, and `github-release` jobs complete.

The following are created automatically:
- `ghcr.io/infraware-dev/infraware-rotelle:v0.2.0`
- `ghcr.io/infraware-dev/infraware-rotelle:latest`
- A GitHub Release at the Releases page

**7. Verify the published image**

```sh
docker pull ghcr.io/infraware-dev/infraware-rotelle:v0.2.0
docker run --rm -p 8080:8080 ghcr.io/infraware-dev/infraware-rotelle:v0.2.0 &
curl http://localhost:8080/rotectl/status
```

---

## `:latest` tag policy

The `latest` tag always points to the most recent release tag. It is never published from `main` directly — only from version tags.

---

## Hotfix releases

For a critical fix that cannot wait for the next planned release:

```sh
git checkout -b fix/critical-description
# apply fix, test
git commit -m "fix: description"
```

Open a PR targeting `main`, get it reviewed and merged normally. Then follow the release checklist from step 1.
