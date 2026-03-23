---
name: dependency-snapshot
description: Crate versions at last audit (2026-03-22) — diff against this on next run to detect changes
type: project
---

## Last Audit: 2026-03-23 (Wave 4 post-merge verification)

### breaker-game direct dependencies
| Crate | Version | Notes |
|-------|---------|-------|
| bevy | 0.18.1 | default-features=false, features=["2d","serialize"] |
| bevy_egui | 0.39 | optional, gated by `dev` feature |
| breaker_derive | path | proc-macro crate |
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

### breaker-derive direct dependencies
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
| clap | 4 | features=["derive"] |
| tracing | 0.1 | |
| tracing-subscriber | 0.3 | features=["env-filter"] |
| ron | 0.12 | consistent with game crate |
| serde | 1 | features=["derive"] |
| rand | 0.9 | pinned (see known-pins) |

### Resolved versions (cargo tree, 2026-03-23)
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

### New transitive duplicates (cargo tree -d, 2026-03-23)
The following duplicates are new since the 2026-03-22 audit (Bevy internal dep tree evolution):
| Crate | Versions |
|-------|---------|
| foldhash | 0.1.5 / 0.2.0 |
| getrandom | 0.3.4 / 0.4.2 |
| hashbrown | 0.15.5 / 0.16.1 |
| itertools | 0.13.0 / 0.14.0 |
All are transitively from Bevy ecosystem — not introduced by project code.

### cargo outdated -R --workspace findings (2026-03-23)
| Crate | Current | Latest | Status |
|-------|---------|--------|--------|
| rand | 0.9.2 | 0.10.0 | INTENTIONAL PIN — matches Bevy 0.18 transitive |
| rand_chacha | 0.9.0 | 0.10.0 | INTENTIONAL PIN — matches Bevy 0.18 transitive |
| bevy_common_assets | 0.15.0 | 0.16.0 | DEFERRED — same ron ^0.11 dep, no benefit for this project; see known-findings |
(tracing-subscriber no longer flagged as outdated)

### cargo deny license gaps (2026-03-23)
| Issue | Crate | Via | Fix |
|-------|-------|-----|-----|
| OPEN: workspace crates unlicensed | breaker, breaker_derive, breaker_scenario_runner | — | Add `publish = false` to each [package] in Cargo.toml |

### Previously-open items now RESOLVED (as of 2026-03-22)
| Item | How resolved |
|------|-------------|
| BSL-1.0 not in allowlist | Added to deny.toml allow list |
| MIT-0 not in allowlist | Added to deny.toml allow list |
| OFL-1.1 not in allowlist | Added to deny.toml allow list |
| Ubuntu-font-1.0 not in allowlist | Added to deny.toml allow list |
| CC0-1.0 not in allowlist | Added to deny.toml allow list |
| Unicode-3.0 not in allowlist | Added to deny.toml allow list |
| tracing-subscriber minor bump | No longer flagged as outdated |

### Note: private.ignore = true does NOT work without publish = false
private.ignore = true in deny.toml requires `publish = false` in each workspace crate's
[package] section. Without it, cargo-deny 0.19 still errors on unlicensed workspace crates.
The previous audit incorrectly marked this as RESOLVED.
