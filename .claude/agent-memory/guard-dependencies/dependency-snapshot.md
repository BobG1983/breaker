---
name: dependency-snapshot
description: Crate versions at last audit (2026-03-28) — diff against this on next run to detect changes
type: project
---

Audit date: 2026-03-28
Branch: feature/runtime-effects (audited at Phase 5 commit — no new crate deps added)

## Direct Dependencies

### breaker-game
- bevy 0.18.1 (default-features = false, features = ["2d", "serialize"])
- bevy_egui 0.39 (optional, dev feature)
- rantzsoft_defaults (path)
- rantzsoft_physics2d (path)
- rantzsoft_spatial2d (path)
- tracing 0.1 (features = ["release_max_level_warn"])
- tracing-appender 0.2
- tracing-subscriber 0.3 (features = ["env-filter", "fmt"])
- serde 1 (features = ["derive"])
- ron 0.12
- iyes_progress 0.16
- rand 0.9
- rand_chacha 0.9
- [dev] proptest 1

### breaker-scenario-runner
- bevy 0.18.1 (default-features = false, features = ["2d"])
- breaker (path, default-features = false)
- rantzsoft_spatial2d (path)
- clap 4 (features = ["derive"])
- tracing 0.1
- tracing-subscriber 0.3 (features = ["env-filter"])
- ron 0.12
- serde 1 (features = ["derive"])
- rand 0.9

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

## Resolved versions (from Cargo.lock)
- rand 0.9.2, rand_chacha 0.9.0, rand_core 0.9.5
- proptest 1.10.0

## Known Outdated (as of audit)
- rand: 0.9.2 → 0.10.0 (BREAKING — semver major; deferred, see known-findings.md)
- rand_chacha: 0.9.0 → 0.10.0 (BREAKING — must match rand; deferred)
- proptest: 1.10.0 → 1.11.0 (non-breaking, dev-only — low priority)

## Transitive Advisory
- paste 1.0.15 (RUSTSEC-2024-0436, unmaintained) — pulled in by metal → wgpu-hal → wgpu → bevy_render
  No actionable fix: upstream Bevy 0.18 owns this dep. Will resolve with Bevy upgrade.

## Branch-Specific Notes (feature/runtime-effects)
- No new crate dependencies were added. Cargo.lock is identical to develop.
- rand `distr` module + `WeightedIndex` used in chips/offering.rs, effect/effects/entropy_engine.rs,
  effect/effects/random_effect.rs — all valid rand 0.9 API (distr renamed from distributions in 0.9).
- rand::Rng trait used in chain_bolt.rs and spawn_phantom.rs — valid rand 0.9 usage.
- rantzsoft_physics2d::ccd made pub — no Cargo.toml change required (visibility only).
