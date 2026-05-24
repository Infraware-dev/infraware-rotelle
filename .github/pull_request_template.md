## What

<!-- One paragraph: what does this PR add or change? -->

## Why

<!-- For new scenarios: what real Kubernetes failure does this simulate, and why is it useful for SREs or AI systems to observe it?
     For fixes/docs: what problem does this solve? -->

## How to test

<!--
Steps to manually verify this PR. Usually:

  1. just run                          # terminal 1 — start server on :8080
  2. just run-case <your-scenario>     # terminal 2 — run the hurl test
  3. just test-hurl                    # control-path regression (must still pass)

If this is a Kubernetes-level scenario (e.g. one that triggers a pod restart), add the kubectl commands needed to observe it.
-->

## Design decisions

<!-- Optional. Note anything non-obvious: why you chose a particular IndexEffect, why state needs to persist across restarts, etc. -->

---

## Checklist

- [ ] Commit messages follow [Conventional Commits](https://www.conventionalcommits.org/) (`feat: ...`, `fix: ...`, etc.)
- [ ] `cargo fmt --all --manifest-path rotelle/Cargo.toml` passes
- [ ] `cargo clippy --manifest-path rotelle/Cargo.toml --all-targets -- -D warnings` passes
- [ ] `cargo deny --manifest-path rotelle/Cargo.toml check` passes
- [ ] `cargo test --manifest-path rotelle/Cargo.toml` passes
- [ ] `just run-case <scenario>` passes
- [ ] `just test-hurl` still passes (control-path regression)
- [ ] `docs/scenarios.md` has an entry for any new scenario
- [ ] PR targets `main` and is focused on one scenario or one concern
