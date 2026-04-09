---
name: dependency-snapshot
description: Crate versions at last audit (2026-04-06) — diff against this on next run to detect changes
type: project
---

Audit date: 2026-04-09 (fourth run — lint config change only; no dep changes)
Branch: develop (commit a663a96a)
Prior audit: 2026-04-08 (third run — Toughness + HP Scaling feature)

## Direct Dependencies

### breaker-game
- bevy 0.18.1 (default-features = false, features = ["2d", "serialize"])
- bevy_egui 0.39 → resolved 0.39.1 (optional, dev feature)
- rantzsoft_defaults (path)
- rantzsoft_physics2d (path)
- rantzsoft_stateflow (path)
- rantzsoft_spatial2d (path)
- tracing 0.1 (features = ["release_max_level_warn"])
- tracing-appender 0.2
- tracing-subscriber 0.3 (features = ["env-filter", "fmt"])
- serde 1 (features = ["derive"])
- ron 0.12
- iyes_progress 0.16 → resolved 0.16.0
- rand 0.9 → resolved 0.9.2
- rand_chacha 0.9 → resolved 0.9.0
- [dev-dependencies]: EMPTY

### breaker-scenario-runner
- bevy 0.18.1 (default-features = false, features = ["2d"])
- breaker (path, default-features = false)
- rantzsoft_stateflow (path)  ← NEW since last audit
- rantzsoft_spatial2d (path)
- rantzsoft_physics2d (path)
- clap 4 (features = ["derive"])
- tracing 0.1
- tracing-subscriber 0.3 (features = ["env-filter"])
- ron 0.12
- serde 1 (features = ["derive"])
- rand 0.9 → resolved 0.9.2

### rantzsoft_stateflow  ← NEW workspace member (added on feature/wall-builder-pattern)
- bevy 0.18.1 (default-features = false, features = ["2d"])
- tracing 0.1

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

## Resolved versions (from cargo tree -d — verified 2026-04-06)
- rand 0.9.2, rand_chacha 0.9.0
- bevy_egui 0.39.1
- iyes_progress 0.16.0
- ron 0.12.0 (0.12.1 now available — patch bump eligible, see known-findings)

## Transitive duplicates (cargo tree -d — 2026-04-06)
All entries below match known-findings.md — no new duplicates introduced by rantzsoft_stateflow:
- bitflags v1.3.2 + v2.11.0 (known wontfix)
- block2 v0.5.1 + v0.6.2 (known wontfix)
- core-foundation v0.9.4 + v0.10.1 (known wontfix)
- core-graphics-types v0.1.3 + v0.2.0 (known wontfix)
- either v1.15.0 (same-version dual-path — not a real dup)
- foldhash v0.1.5 + v0.2.0 (known wontfix)
- getrandom v0.3.4 + v0.4.2 (known wontfix)
- hashbrown v0.15.5 + v0.16.1 (known wontfix)
- indexmap v2.13.0 (same-version dual-path — not a real dup)
- itertools v0.13.0 + v0.14.0 (known wontfix)
- libc v0.2.183 (same-version dual-path — not a real dup)
- memchr v2.8.0 (same-version dual-path — not a real dup)
- objc2 v0.5.2 + v0.6.4 (known wontfix)
- objc2-app-kit v0.2.2 + v0.3.2 (known wontfix — same chain as objc2)
- objc2-foundation v0.2.2 + v0.3.2 (known wontfix — same chain as objc2)
- read-fonts v0.35.0 + v0.36.0 (known wontfix)
- regex v1.12.3 (same-version dual-path — not a real dup)
- regex-automata v0.4.14 (same-version dual-path — not a real dup)
- regex-syntax v0.8.10 (same-version dual-path — not a real dup)
- rustc-hash v1.1.0 + v2.1.1 (known wontfix)
- skrifa v0.37.0 + v0.39.0 (known wontfix)

## Changes since prior audit (2026-04-08, third run — Toughness + HP Scaling)
- Cargo.toml (workspace root) modified: [workspace.lints] section restructured only
  — warn→deny promotions, specific nursery lint opt-ins replacing blanket nursery group
  — NO dependency changes; [dependencies], [features], [profile] sections untouched
- cargo-machete: not re-run (Bash denied this session); prior result CLEAN still valid
- cargo outdated -R: not re-run (no dep version changes)
- cargo deny: not re-run (no dep changes; license state unchanged)
- Transitive dups: not re-run (dependency tree unchanged)
- Feature flags: unchanged

## Known Outdated (as of audit)
- rand: 0.9.2 → 0.10.0 (BREAKING — semver major; deferred, see known-findings.md)
- rand_chacha: 0.9.0 → 0.10.0 (BREAKING — must match rand; deferred)
- ron: 0.12.0 → 0.12.1 (PATCH — no breaking changes; low-risk bump; three Cargo.toml files)

## Transitive Advisory
- paste 1.0.15 (RUSTSEC-2024-0436, unmaintained) — pulled in by metal → wgpu-hal → wgpu → bevy_render
  No actionable fix: upstream Bevy 0.18 owns this dep. Will resolve with Bevy upgrade.

## License Compliance (as of audit)
- cargo deny check licenses: PASS
- One harmless warning: Unicode-DFS-2016 pre-allowlisted but not currently matched by any dep
- r-efi (LGPL-2.1-or-later): deny.toml exception present; platform-target-only (Android/WASM)
- self_cell (GPL-2.0-only): deny.toml exception present; runtime dep via cosmic-text → bevy_text

## Feature Flag Audit (as of audit)
- bevy/dynamic_linking: ONLY in .cargo/config.toml aliases; NOT in any Cargo.toml or release profile. CLEAN.
- bevy features ("2d", "serialize"): appropriate; serialize needed for RON/serde scene support.
- bevy_egui: optional behind "dev" feature; not included in release/scenario builds. CLEAN.
- iyes_progress: optional behind "progress" feature in rantzsoft_defaults. CLEAN.
- hot-reload: feature-gated in rantzsoft_defaults; activated by breaker-game dev feature. CLEAN.
- rantzsoft_stateflow: no features declared. CLEAN.
