---
name: dependency-snapshot
description: Crate versions at last audit (2026-03-30) — diff against this on next run to detect changes
type: project
---

Audit date: 2026-03-30
Branch: develop

## Direct Dependencies

### breaker-game
- bevy 0.18.1 (default-features = false, features = ["2d", "serialize"])
- bevy_egui 0.39 → resolved 0.39.1 (optional, dev feature)
- rantzsoft_defaults (path)
- rantzsoft_physics2d (path)
- rantzsoft_spatial2d (path)
- tracing 0.1 (features = ["release_max_level_warn"])
- tracing-appender 0.2
- tracing-subscriber 0.3 (features = ["env-filter", "fmt"])
- serde 1 (features = ["derive"])
- ron 0.12
- iyes_progress 0.16 → resolved 0.16.0
- rand 0.9 → resolved 0.9.2
- rand_chacha 0.9 → resolved 0.9.0
- [dev-dependencies]: EMPTY (proptest removed since prior audit)

### breaker-scenario-runner
- bevy 0.18.1 (default-features = false, features = ["2d"])
- breaker (path, default-features = false)
- rantzsoft_spatial2d (path)
- clap 4 (features = ["derive"])
- tracing 0.1
- tracing-subscriber 0.3 (features = ["env-filter"])
- ron 0.12
- serde 1 (features = ["derive"])
- rand 0.9 → resolved 0.9.2

### rantzsoft_spatial2d
- bevy 0.18.1 (default-features = false, features = ["2d"])

### rantzsoft_physics2d
- bevy 0.18.1 (default-features = false, features = ["2d"])
- rantzsoft_spatial2d (path)

### rantzsoft_defaults
- rantzsoft_defaults_derive (path)
- bevy 0.18.1 (default-features = false, features = ["2d"])
- ron 0.12
- serde 1 (features = ["derive"])
- iyes_progress 0.16 (optional, progress feature)

### rantzsoft_defaults_derive
- syn 2 (features = ["full"])
- quote 1
- proc-macro2 1 (ignored by machete — required for proc-macro crates)

## Resolved versions (from Cargo.lock — verified 2026-03-30)
- rand 0.9.2, rand_chacha 0.9.0
- bevy_egui 0.39.1
- iyes_progress 0.16.0
- objc2 v0.5.2 + v0.6.4 (dual — known wontfix)
- bitflags v1.3.2 + v2.11.0 (dual — known wontfix)
- getrandom v0.3.4 + v0.4.2 (dual — known wontfix)
- foldhash v0.1.5 + v0.2.0 (dual — known wontfix)
- r-efi v5.3.0 + v6.0.0 (dual — platform-conditional, WASM+Android targets only; not loaded on macOS)

## Changes since 2026-03-29 audit
- proptest removed entirely from breaker-game dev-dependencies and Cargo.lock
- r-efi dual versions newly observed (v5.3.0 via getrandom 0.3 → cc build-dep; v6.0.0 via getrandom 0.4 → uuid → Bevy)
  Both are target-conditional (Android/WASM) and not present in macOS host build.

## Known Outdated (as of audit)
- rand: 0.9.2 → 0.10.0 (BREAKING — semver major; deferred, see known-findings.md)
- rand_chacha: 0.9.0 → 0.10.0 (BREAKING — must match rand; deferred)
- cargo outdated -R shows no other outdated direct deps

## Transitive Advisory
- paste 1.0.15 (RUSTSEC-2024-0436, unmaintained) — pulled in by metal → wgpu-hal → wgpu → bevy_render
  No actionable fix: upstream Bevy 0.18 owns this dep. Will resolve with Bevy upgrade.
  Confirmed still present 2026-03-30 via cargo deny check advisories.

## License Compliance (as of audit)
- cargo deny check licenses: PASS (one harmless warning: Unicode-DFS-2016 pre-allowlisted but not currently used)
- r-efi (LGPL-2.1-or-later): deny.toml exception present; r-efi is platform-target-only (Android/WASM)
  Not loaded on macOS. Exception is appropriate; flag for review if project targets Android/WASM.
- self_cell (GPL-2.0-only): deny.toml exception present; self_cell → cosmic-text → bevy_text → bevy
  This is a runtime dep on all platforms. GPL-2.0-only exception requires scrutiny — see known-findings.md.

## Feature Flag Audit (as of audit)
- bevy/dynamic_linking: ONLY in .cargo/config.toml aliases; NOT in any Cargo.toml or release profile. CLEAN.
- bevy features ("2d", "serialize"): appropriate; serialize is needed for RON/serde scene support.
- bevy_egui: optional behind "dev" feature; not included in release/scenario builds. CLEAN.
- iyes_progress: optional behind "progress" feature in rantzsoft_defaults. CLEAN.
- hot-reload: feature-gated in rantzsoft_defaults; activated by breaker-game dev feature. CLEAN.
