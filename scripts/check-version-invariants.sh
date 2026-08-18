#!/usr/bin/env bash
# Usage: ./scripts/check-version-invariants.sh
#
# Verifies that all version-gated files agree on the app version.
# Exit code 0 = all green, 1 = mismatch found.
#
# The hard invariants:
#   - rotelle/Cargo.toml package version == helm values.yaml image.tag
#   - helm Chart.yaml appVersion == same version
#   - rotelle/Cargo.lock root crate version == same version
#
# The flexible invariant:
#   - helm Chart.yaml version (the *chart* version) may differ — it
#     bumps on chart structure changes, independent of the app version.

set -euo pipefail
ERRORS=0

die() { echo "ERROR: $*" >&2; ERRORS=$((ERRORS + 1)); }

# ---------------------------------------------------------------------------
# Extract versions
# ---------------------------------------------------------------------------
CARGO_TOML=$(grep '^version' rotelle/Cargo.toml | head -1 | sed 's/version = "//;s/"//')
CARGO_LOCK=$(grep -A1 'name = "rotelle"' rotelle/Cargo.lock | grep '^version' | sed 's/version = "//;s/"//')
CHART_APP=$(grep '^appVersion:' helm/rotelle/Chart.yaml | sed 's/appVersion: *"//;s/"//')
CHART_VER=$(grep '^version:' helm/rotelle/Chart.yaml | head -1 | sed 's/version: //')
# values.yaml tag includes the 'v' prefix — strip it for numeric comparison
VALUES_TAG_RAW=$(grep '^  tag:' helm/rotelle/values.yaml | sed 's|.*tag: *"\(.*\)".*|\1|')
VALUES_TAG_UNV=${VALUES_TAG_RAW#v}

# ---------------------------------------------------------------------------
# Invariant 1: Cargo.toml == Cargo.lock
# ---------------------------------------------------------------------------
if [[ "$CARGO_TOML" != "$CARGO_LOCK" ]]; then
  die "Cargo.toml ($CARGO_TOML) != Cargo.lock ($CARGO_LOCK)"
else
  echo "✓ Cargo.toml == Cargo.lock  ($CARGO_TOML)"
fi

# ---------------------------------------------------------------------------
# Invariant 2: Cargo.toml == Chart.yaml appVersion
# ---------------------------------------------------------------------------
if [[ "$CARGO_TOML" != "$CHART_APP" ]]; then
  die "Cargo.toml ($CARGO_TOML) != Chart.yaml appVersion ($CHART_APP)"
else
  echo "✓ appVersion matches Cargo.toml  ($CARGO_TOML)"
fi

# ---------------------------------------------------------------------------
# Invariant 3: Cargo.toml == values.yaml image.tag (stripped of 'v' prefix)
# ---------------------------------------------------------------------------
if [[ "$CARGO_TOML" != "$VALUES_TAG_UNV" ]]; then
  die "Cargo.toml ($CARGO_TOML) != values.yaml image.tag (v$VALUES_TAG_UNV)"
else
  echo "✓ image.tag matches Cargo.toml  ($VALUES_TAG_RAW)"
fi

# ---------------------------------------------------------------------------
# Flexible: chart version (informational only)
# ---------------------------------------------------------------------------
if [[ "$CARGO_TOML" != "$CHART_VER" ]]; then
  echo "ℹ Chart version differs — app=$CARGO_TOML  chart=$CHART_VER (ok — chart version bumps independently)"
else
  echo "✓ Chart version matches app version  ($CARGO_TOML)"
fi

# ---------------------------------------------------------------------------
# Summary
# ---------------------------------------------------------------------------
if [[ $ERRORS -gt 0 ]]; then
  echo ""
  echo "❌ $ERRORS invariant(s) violated — versions are out of sync."
  echo "Run ./scripts/release-prep.sh <version> to fix."
  exit 1
fi

echo ""
echo "✅ All version invariants satisfied."
exit 0
