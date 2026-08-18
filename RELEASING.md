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
cargo test --manifest-path rotelle/Cargo.toml   # must pass
just test-hurl                                   # must pass (server running in terminal 1)
./scripts/check-version-invariants.sh            # must pass
```

> `check-version-invariants.sh` can also be added to CI or a git pre-push hook
> to catch out-of-sync versions before they reach `main`.

**2. Run `scripts/release-prep.sh`**

This script bumps the version across *all* version-gated files and
commits the result in one atomic change — no chance of forgetting one
of the five locations:

```sh
./scripts/release-prep.sh 0.2.1
```

It updates:
- `rotelle/Cargo.toml` (package version)
- `rotelle/Cargo.lock` (lock file entry)
- `helm/rotelle/Chart.yaml` (`version` + `appVersion`)
- `helm/rotelle/values.yaml` (default `image.tag`)
- `helm/rotelle/README.md` (doc reference to the default image)
- `CHANGELOG.md` (inserts a new `[0.2.1]` entry with today's date)

The commit message is `chore: release v0.2.1` — inspect the diff before
pushing.  Verify invariants hold:

```sh
./scripts/check-version-invariants.sh
```

> This pushes directly to `main` and requires maintainer/admin access to the repository. GitHub's branch protection is configured to allow admin bypass for exactly this use case. If you cannot push directly, open a short PR titled `chore: release v0.2.1` and merge it first.

**3. Tag the release**

Pushing the tag triggers the publish workflow (multi-arch build → GHCR push → GitHub Release creation).

```sh
git push origin main
git tag v0.2.1
git push origin v0.2.1
```

**4. Watch the publish workflow**

Go to [Actions → Publish to GHCR](https://github.com/infraware-dev/infraware-rotelle/actions/workflows/publish.yml) and confirm the `build`, `merge`, and `github-release` jobs complete.

The following are created automatically:
- `ghcr.io/infraware-dev/infraware-rotelle:v0.2.1`
- `ghcr.io/infraware-dev/infraware-rotelle:latest`
- A GitHub Release at the Releases page

**5. Verify the published image**

```sh
docker pull ghcr.io/infraware-dev/infraware-rotelle:v0.2.1
docker run --rm -p 8080:8080 ghcr.io/infraware-dev/infraware-rotelle:v0.2.1 &
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
