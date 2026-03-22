---
name: dependency-snapshot
description: Crate versions at last audit (2026-03-22) — diff against this on next run to detect changes
type: project
---

## Last Audit: 2026-03-22

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

### Resolved versions (cargo tree, 2026-03-22)
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

### cargo outdated -R --workspace findings (2026-03-22)
| Crate | Current | Latest | Status |
|-------|---------|--------|--------|
| rand | 0.9.2 | 0.10.0 | INTENTIONAL PIN — matches Bevy 0.18 transitive |
| rand_chacha | 0.9.0 | 0.10.0 | INTENTIONAL PIN — matches Bevy 0.18 transitive |
| bevy_common_assets | 0.15.0 | 0.16.0 | DEFERRED — same ron ^0.11 dep, no benefit for this project; see known-findings |
| tracing-subscriber | 0.3.22 | — | RESOLVED — no longer flagged as outdated |

### cargo deny license gaps (2026-03-22)
| Issue | Crate | Via | Fix |
|-------|-------|-----|-----|
| OFL-1.1 not in allowlist | epaint_default_fonts 0.33.3 | bevy_egui | Add "OFL-1.1" to deny.toml allow |
| Ubuntu-font-1.0 not in allowlist | epaint_default_fonts 0.33.3 | bevy_egui | Add "Ubuntu-font-1.0" to deny.toml allow |
| CC0-1.0 not in allowlist | hexf-parse 0.2.1 | naga → bevy_render | Add "CC0-1.0" to deny.toml allow |
| Unicode-3.0 not in allowlist | unicode-ident | naga → bevy_render | Add "Unicode-3.0" to deny.toml allow |

### Previously-open items now RESOLVED
| Item | How resolved |
|------|-------------|
| BSL-1.0 not in allowlist | Added to deny.toml allow list |
| MIT-0 not in allowlist | Added to deny.toml allow list |
| Workspace crates unlicensed | private.ignore = true added to deny.toml |
| tracing-subscriber minor bump | Resolved (0.3.23 now resolves the 0.3.22 lockfile) |
