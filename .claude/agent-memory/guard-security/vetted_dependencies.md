---
name: vetted_dependencies
description: Verified dependency versions and audit state as of 2026-03-23 (Wave 4 audit)
type: project
---

Vetted as of 2026-03-23 (feature/wave-3-offerings-transitions, Wave 4 audit). Prior vetted: 2026-03-22 (Wave 3 audit).

## Direct Dependencies

### breaker-game
- bevy 0.18.1 — current, trusted
- bevy_egui 0.39.1 (optional dev feature) — current, trusted
- breaker_derive (path) — internal
- tracing 0.1 — current, trusted
- tracing-appender 0.2 — current, trusted
- tracing-subscriber 0.3 — current, trusted
- serde 1 — current, trusted
- ron 0.12 — upgraded from 0.11, no known CVEs
- bevy_common_assets 0.15 — current, trusted
- bevy_asset_loader 0.25 — current, trusted
- iyes_progress 0.16 — current, trusted
- rand 0.9 — current, trusted
- rand_chacha 0.9 — current, trusted
- proptest 1 (dev) — current, trusted

### breaker-derive
- syn 2 — current, trusted
- quote 1 — current, trusted
- proc-macro2 1 — machete false positive; suppressed via `[package.metadata.cargo-machete]`

### breaker-scenario-runner
- bevy 0.18.1 — current, trusted
- breaker (path) — internal
- clap 4 — current, trusted
- tracing 0.1 — current, trusted
- tracing-subscriber 0.3 — current, trusted
- ron 0.12 — upgraded from 0.11, no known CVEs
- serde 1 — current, trusted
- rand 0.9 — current, trusted

## cargo audit result (2026-03-23, Wave 4 audit)
- 1 warning only: RUSTSEC-2024-0436 — `paste` 1.0.15 unmaintained (transitive through metal → wgpu-hal → bevy_render)
- No CVEs or errors
- No new dependencies added in Wave 4. Dep list unchanged from Wave 3 audit.

## cargo audit result (2026-03-22, Wave 3 audit)
- 1 warning only: RUSTSEC-2024-0436 — `paste` 1.0.15 unmaintained (transitive through metal → wgpu-hal → bevy_render)
- No CVEs or errors

## cargo audit result (2026-03-19, post-upgrade)
- 1 warning only: RUSTSEC-2024-0436 — `paste` 1.0.15 unmaintained (transitive through metal → wgpu-hal → bevy_render)
- No CVEs or errors
- Same result as prior audit — upgrade to ron 0.12 introduced no new advisories

## cargo machete result (2026-03-19)
- No unused dependencies found (proc-macro2 correctly suppressed with cargo-machete ignore)

## cargo deny check result (2026-03-19)
Errors (deny.toml incomplete):
- First-party workspace crates missing `license` field: breaker, breaker_derive, breaker_scenario_runner
- `clipboard-win 5.4.1` uses BSL-1.0 — not in deny.toml allowlist (transitive: bevy_egui → arboard)
- `encase 0.12.0` uses MIT-0 — not in deny.toml allowlist (transitive: bevy → bevy_color)

All three are permissive, low-risk licenses. deny.toml needs updating to add exceptions.

## Notes
- Two `ron` versions in lock: 0.11.0 (transitive) and 0.12.0 (direct). Not a security concern.
- First-party crates still have no `license` field — this causes `cargo deny` to fail but is a
  project hygiene issue, not a security concern for a non-published crate.
- Nightly toolchain pinned in rust-toolchain.toml (channel = "nightly", no version pin)
- Wave 4 added NO new crate dependencies.

**Why:** To detect new or changed deps in future audits without re-running full audit.
**How to apply:** On the next audit, diff current Cargo.toml against this list to identify additions or version bumps.
