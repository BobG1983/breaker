---
name: vetted_dependencies
description: Verified dependency versions and audit state as of 2026-03-19
type: project
---

Vetted as of 2026-03-19 (develop, commit 7986274).

## Direct Dependencies

### breaker-game
- bevy 0.18.1 — current, trusted
- bevy_egui 0.39 (optional dev feature) — current, trusted
- breaker_derive (path) — internal
- tracing 0.1 — current, trusted
- tracing-appender 0.2 — current, trusted
- tracing-subscriber 0.3 — current, trusted
- serde 1 — current, trusted
- ron 0.11 — current, no known CVEs
- bevy_common_assets 0.15 — current, trusted
- bevy_asset_loader 0.25 — current, trusted
- iyes_progress 0.16 — current, trusted
- rand 0.9 — current, trusted
- rand_chacha 0.9 — current, trusted
- proptest 1 (dev) — current, trusted

### breaker-derive
- syn 2 — current, trusted
- quote 1 — current, trusted
- proc-macro2 1 — listed but UNUSED (machete finding, see audit)

### breaker-scenario-runner
- bevy 0.18.1 — current, trusted
- breaker (path) — internal
- clap 4 — current, trusted
- tracing 0.1 — current, trusted
- tracing-subscriber 0.3 — current, trusted
- ron 0.11 — current, no known CVEs
- serde 1 — current, trusted
- rand 0.9 — current, trusted

## cargo audit result (2026-03-19)
- 1 warning only: RUSTSEC-2024-0436 — `paste` 1.0.15 unmaintained (transitive through metal → wgpu-hal → bevy_render)
- No CVEs or errors

## Notes
- Two `ron` versions in lock: 0.11.0 (direct) and 0.12.0 (transitive). Not a security concern.
- Nightly toolchain pinned in rust-toolchain.toml (no version pinned, just channel = "nightly")

**Why:** To detect new or changed deps in future audits without re-running full audit.
**How to apply:** On the next audit, diff current Cargo.toml against this list to identify additions or version bumps.
