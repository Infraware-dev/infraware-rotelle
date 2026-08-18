#!/usr/bin/env bash
# Usage: ./scripts/release-prep.sh 0.2.1
#
# Bumps rotelle from its current version to the given version across
# all version-gated files, then commits the result.
#
# Run this *before* tagging. The commit message is deterministic so
# a maintainer can always review what changed.
#
set -euo pipefail
shopt -s nullglob

if [[ $# -ne 1 ]]; then
  echo "Usage: $0 <new-version>"
  echo "  No leading 'v' — use 0.2.1, not v0.2.1"
  exit 1
fi

NEW="$1"

# ---------------------------------------------------------------------------
# Sanity checks
# ---------------------------------------------------------------------------
if [[ ! "$NEW" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  echo "ERROR: version must be X.Y.Z (got '$NEW')"
  exit 1
fi

# Work in a clean tree — don't risk partial commits
if ! git diff --quiet || ! git diff --cached --quiet; then
  echo "ERROR: working tree is dirty. Commit or stash changes first."
  exit 1
fi

# ---------------------------------------------------------------------------
# Current version (used for CHANGELOG bump context)
# ---------------------------------------------------------------------------
OLD=$(grep '^version' rotelle/Cargo.toml | head -1 | sed 's/version = "//;s/"//')

# ---------------------------------------------------------------------------
# 1. Bump Cargo.toml
# ---------------------------------------------------------------------------
sed -i "s/version = \"${OLD}\"/version = \"${NEW}\"/" rotelle/Cargo.toml

# 2. Bump Cargo.lock (the lock file also has a [package] version line for
#    the root crate)
sed -i "s/^version = \"${OLD}\"/version = \"${NEW}\"/" rotelle/Cargo.lock

# 3. Helm chart — appVersion and chart version
sed -i "s/^appVersion:.*/appVersion: \"${NEW}\"/" helm/rotelle/Chart.yaml
sed -i "s/^version: [0-9.]\+/version: ${NEW}/" helm/rotelle/Chart.yaml

# 4. Helm default image tag
sed -i "s|tag: \"v[0-9.]*\"|tag: \"v${NEW}\"|" helm/rotelle/values.yaml

# 5. Helm README — doc reference to the default image
sed -i "s|infraware-rotelle:v[0-9.]*|infraware-rotelle:v${NEW}|" helm/rotelle/README.md

# 6. Changelog — insert new entry at the top (after the description block,
#    before the first version header which follows the comment line)
TODAY=$(date +%F)
CHANGELOG="CHANGELOG.md"
TEMP=$(mktemp)

# Write the new section header
cat > "$TEMP" <<EOF
## [${NEW}] - ${TODAY}

### Changed

- (describe the release)

EOF

# Append the old changelog content (everything from the comment line onwards)
# We start from the comment line so the file retains its metadata comment.
sed -n '/<!-- Maintained manually/,$p' "$CHANGELOG" >> "$TEMP"
mv "$TEMP" "$CHANGELOG"

# ---------------------------------------------------------------------------
# Commit
# ---------------------------------------------------------------------------
git add rotelle/Cargo.toml rotelle/Cargo.lock \
      helm/rotelle/Chart.yaml helm/rotelle/values.yaml helm/rotelle/README.md \
      CHANGELOG.md

git commit -m "chore: release v${NEW}"

echo "Done — committed as v${NEW} (was ${OLD})."
echo ""
echo "Next steps:"
echo "  git push origin main"
echo "  git tag v${NEW}"
echo "  git push origin v${NEW}"
