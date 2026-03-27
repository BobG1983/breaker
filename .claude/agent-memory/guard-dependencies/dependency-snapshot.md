---
name: dependency-snapshot
description: Crate versions at last audit (2026-03-26) — diff against this on next run to detect changes
type: project
---

## Last Audit: 2026-03-26 (post-removal of bevy_common_assets + bevy_asset_loader)

### Change delta since 2026-03-24
- REMOVED from breaker-game: `bevy_common_assets 0.15`, `bevy_asset_loader 0.25`
- REMOVED from Cargo.lock: both of the above, plus their unique transitives
  (bevy_common_assets pulled ron 0.11; bevy_asset_loader pulled its own transitives)
- RETAINED in breaker-game: `iyes_progress 0.16` (still used directly by screen/loading and chips)
- CHANGED rantzsoft_defaults: now declares `iyes_progress` as optional dep gated by `progress` feature
  and carries a `hot-reload = []` empty feature. `default = ["progress"]`. Correct.
- RESOLVED transitive duplicate: `ron 0.11.0` is now gone — no longer pulled by bevy_common_assets.
  Only `ron 0.12.0` remains. Tree is now unified.

### breaker-game direct dependencies
| Crate | Version | Notes |
|-------|---------|-------|
| bevy | 0.18.1 | default-features=false, features=["2d","serialize"] |
| bevy_egui | 0.39 | optional, gated by `dev` feature |
| rantzsoft_defaults | path | config/defaults pipeline |
| rantzsoft_physics2d | path | physics primitives |
| rantzsoft_spatial2d | path | spatial transform |
| tracing | 0.1 | features=["release_max_level_warn"] |
| tracing-appender | 0.2 | used in app.rs file logger |
| tracing-subscriber | 0.3 | features=["env-filter","fmt"] |
| serde | 1 | features=["derive"] |
| ron | 0.12 | matches Bevy 0.18 transitive; ron 0.11 duplicate ELIMINATED |
| iyes_progress | 0.16 | used by screen/loading and chips — NOT optional in breaker-game |
| rand | 0.9 | pinned to match Bevy 0.18 internals |
| rand_chacha | 0.9 | pinned to match Bevy 0.18 internals |
| proptest | 1 | dev-dependency |

### rantzsoft_defaults direct dependencies
| Crate | Version | Notes |
|-------|---------|-------|
| rantzsoft_defaults_derive | path | proc-macro crate |
| bevy | 0.18.1 | default-features=false, features=["2d"] — minimal, correct |
| ron | 0.12 | |
| serde | 1 | features=["derive"] |
| iyes_progress | 0.16 | optional, gated by `progress` feature (default enabled) |

### rantzsoft_defaults features (verified 2026-03-26)
| Feature | Deps activated | Status |
|---------|---------------|--------|
| default = ["progress"] | iyes_progress | CORRECT — progress enabled by default |
| progress | dep:iyes_progress | CORRECT — optional dep properly declared |
| hot-reload = [] | (none — empty) | CORRECT — activates no deps; enables cfg gates in plugin/systems |

### rantzsoft_spatial2d direct dependencies
| Crate | Version | Notes |
|-------|---------|-------|
| bevy | 0.18.1 | default-features=false, features=["2d"] — minimal, correct |

### rantzsoft_physics2d direct dependencies
| Crate | Version | Notes |
|-------|---------|-------|
| bevy | 0.18.1 | default-features=false, features=["2d"] — minimal, correct |
| rantzsoft_spatial2d | path | correct dep chain |

### rantzsoft_defaults_derive direct dependencies
| Crate | Version | Notes |
|-------|---------|-------|
| syn | 2 | features=["full"] |
| quote | 1 | |
| proc-macro2 | 1 | machete suppressed via [package.metadata.cargo-machete] |

### breaker-scenario-runner direct dependencies
| Crate | Version | Notes |
|-------|---------|-------|
| bevy | 0.18.1 | default-features=false, features=["2d"] |
| breaker | path | default-features=false — dev-feature-leak fixed |
| rantzsoft_spatial2d | path | used directly in invariant checkers and lifecycle |
| clap | 4 | features=["derive"] |
| tracing | 0.1 | |
| tracing-subscriber | 0.3 | features=["env-filter"] |
| ron | 0.12 | consistent with game crate |
| serde | 1 | features=["derive"] |
| rand | 0.9 | pinned (see known-pins) |

### Transitive duplicates status (2026-03-26)
- ron 0.11/0.12 split: RESOLVED — bevy_common_assets removed, only ron 0.12 remains
- foldhash, getrandom, hashbrown, itertools, bitflags: still present (Bevy internal, no action)
- macOS/objc2 ecosystem splits: still present (Bevy-controlled, no action)

### Known findings still applicable
- rand 0.9 / rand_chacha 0.9 pins: INTENTIONAL (Bevy 0.18 internals)
- bevy/serialize feature: DEFER (see known-findings)
- proc-macro2 machete false positive: suppressed (see known-findings)
- macOS objc2 version split: WONTFIX (upstream)
- `cargo deny check licenses`: last clean run 2026-03-24, no dependency changes that
  would affect license set (iyes_progress is Apache-2.0, already in tree)
