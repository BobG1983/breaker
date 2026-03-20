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

### ron = "0.12" (upgraded 2026-03-19)
- **Why (historical):** Previously pinned at 0.11 causing a ron 0.11/0.12 split in the tree.
  Upgraded to 0.12 in the 2026-03-19 session — now unified with Bevy 0.18's transitive ron 0.12.
- **Status:** RESOLVED — no longer a pin concern. Both game crate and scenario runner use 0.12.
