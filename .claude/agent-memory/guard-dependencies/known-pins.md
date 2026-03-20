---
name: known-pins
description: Intentional version pins and their rationale — do not flag these as outdated
type: project
---

## Intentional Version Pins

### rand = "0.9" and rand_chacha = "0.9"
- **Why:** Bevy 0.18 uses `rand 0.9.x` transitively via `bevy_math`. Upgrading to `rand 0.10`
  would introduce a second copy of `rand` in the dep tree (0.9 + 0.10), increasing compile time
  and binary size. Pinning to 0.9 keeps a single unified rand version across the tree.
- **Where:** `breaker-game/Cargo.toml` and `breaker-scenario-runner/Cargo.toml`
- **When to re-evaluate:** When Bevy upgrades to use rand 0.10+ in its own internals.

### ron = "0.11"
- **Why:** Bevy 0.18 pulls `ron 0.12` transitively (via bevy_asset, bevy_scene, bevy_animation).
  The direct dep is pinned at 0.11, creating a ron v0.11/v0.12 split in the tree already.
  Upgrading the direct dep to 0.12 would eliminate this split — this is actually RECOMMENDED.
  If ron 0.12 has no breaking API changes affecting this codebase, it's safe to upgrade.
- **Status:** NOT an intentional pin — upgrade is recommended. See audit report.
