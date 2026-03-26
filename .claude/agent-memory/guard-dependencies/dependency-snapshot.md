---
name: dependency-snapshot
description: Crate versions at last audit (2026-03-24) — diff against this on next run to detect changes
type: project
---

## Last Audit: 2026-03-24 (pre-merge guard, wave-3-offerings-transitions)

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
| ron | 0.12 | matches Bevy 0.18 transitive |
| bevy_common_assets | 0.15 | features=["ron"] — 0.16 available but DEFERRED (see known-findings) |
| bevy_asset_loader | 0.25 | features=["progress_tracking"] |
| iyes_progress | 0.16 | |
| rand | 0.9 | pinned to match Bevy 0.18 internals |
| rand_chacha | 0.9 | pinned to match Bevy 0.18 internals |
| proptest | 1 | dev-dependency |

### rantzsoft_spatial2d direct dependencies
| Crate | Version | Notes |
|-------|---------|-------|
| bevy | 0.18.1 | default-features=false, features=["2d"] — minimal, correct |

### rantzsoft_physics2d direct dependencies
| Crate | Version | Notes |
|-------|---------|-------|
| bevy | 0.18.1 | default-features=false, features=["2d"] — minimal, correct |
| rantzsoft_spatial2d | path | correct dep chain |

### rantzsoft_defaults direct dependencies
| Crate | Version | Notes |
|-------|---------|-------|
| rantzsoft_defaults_derive | path | proc-macro crate |

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
| breaker | path | default-features=false — dev-feature-leak FIXED |
| rantzsoft_spatial2d | path | used directly in invariant checkers and lifecycle |
| clap | 4 | features=["derive"] |
| tracing | 0.1 | |
| tracing-subscriber | 0.3 | features=["env-filter"] |
| ron | 0.12 | consistent with game crate |
| serde | 1 | features=["derive"] |
| rand | 0.9 | pinned (see known-pins) |

### Resolved versions (cargo tree, 2026-03-24)
| Crate | Resolved |
|-------|---------|
| rand | 0.9.2 |
| rand_chacha | 0.9.0 |
| tracing | 0.1.44 |
| tracing-appender | 0.2.4 |
| tracing-subscriber | 0.3.22 |
| serde | 1.0.228 |
| ron | 0.12.0 (direct), 0.11.0 (transitive via bevy_common_assets) |
| clap | 4.6.0 |
| proptest | 1.10.0 |

### Transitive duplicates (cargo tree -d, 2026-03-24)

#### Known / Bevy-ecosystem (no project action)
| Crate | Versions | Source |
|-------|---------|--------|
| ron | 0.11.0 / 0.12.0 | bevy_common_assets pins ^0.11 — WONTFIX |
| foldhash | 0.1.5 / 0.2.0 | Bevy internal |
| getrandom | 0.3.4 / 0.4.2 | Bevy internal |
| hashbrown | 0.15.5 / 0.16.1 | Bevy internal |
| itertools | 0.13.0 / 0.14.0 | Bevy internal |
| bitflags | 1.3.2 / 2.11.0 | macOS platform crates |

#### macOS platform / objc2 ecosystem (all Bevy-controlled, no project action)
| Crate | Versions |
|-------|---------|
| block2 | 0.5.1 / 0.6.2 |
| core-foundation | 0.9.4 / 0.10.1 |
| core-graphics-types | 0.1.3 / 0.2.0 |
| objc2 | 0.5.2 / 0.6.4 |
| objc2-app-kit | 0.2.2 / 0.3.2 |
| objc2-foundation | 0.2.2 / 0.3.2 |
| quick-error | 1.2.3 / 2.0.1 |
| read-fonts | 0.35.0 / 0.36.0 |
| rustc-hash | 1.1.0 / 2.1.1 |
| skrifa | 0.37.0 / 0.39.0 |

These macOS platform duplicates first appeared 2026-03-24. All driven by bevy_egui / winit
pulling different objc2 generations. Resolves when bevy_egui or winit unify their objc2 pins.

### cargo outdated -R --workspace findings (2026-03-24)
| Crate | Current | Latest | Status |
|-------|---------|--------|--------|
| rand | 0.9.2 | 0.10.0 | INTENTIONAL PIN — matches Bevy 0.18 transitive |
| rand_chacha | 0.9.0 | 0.10.0 | INTENTIONAL PIN — matches Bevy 0.18 transitive |
| bevy_common_assets | 0.15.0 | 0.16.0 | DEFERRED — same ron ^0.11 dep, no benefit; see known-findings |

### cargo deny check licenses (2026-03-24)
Result: `licenses ok`
Warning: `Unicode-DFS-2016` in allowlist not encountered — harmless, keep for future Bevy versions.

### Workspace crates license status — RESOLVED
All six workspace crates have `publish = false`:
- breaker-game, breaker-scenario-runner, rantzsoft_spatial2d, rantzsoft_physics2d,
  rantzsoft_defaults, rantzsoft_defaults_derive
`cargo deny check licenses` passes with no errors.
Note: `breaker-derive` referenced in prior audits no longer exists — replaced by `rantzsoft_defaults_derive`.
