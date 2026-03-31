---
name: Known cleanup marker gaps in effect domain
description: Pre-existing effect entities (gravity_well, shockwave) that self-despawn but lack CleanupOnNodeExit, risking entity leaks on node transition
type: project
---

All known effect entities have `CleanupOnNodeExit` markers as of 2026-03-30 (full-verification-fixes branch).

**Confirmed correctly handled:**
- `effect/effects/gravity_well/effect.rs` — FIXED (full-verification-fixes): spawned `GravityWellMarker` entity now includes `CleanupOnNodeExit` at spawn site (fire(), confirmed present in directory module). Tests assert presence.
- `effect/effects/shockwave.rs` — has `CleanupOnNodeExit` (fixed in Phase 4)
- `effect/effects/pulse.rs` — PulseRing entities have `CleanupOnNodeExit`
- `effect/effects/explode.rs` — ExplodeRequest entities have `CleanupOnNodeExit`
- `effect/effects/second_wind.rs` — Wall `#[require]` auto-inserts `CleanupOnNodeExit`
- `effect/effects/chain_bolt.rs` — ChainBolt entities have `CleanupOnNodeExit`
- `effect/effects/spawn_phantom.rs` — PhantomBolt entities have `CleanupOnNodeExit`

**No currently open gaps.** When reviewing any NEW effect that spawns entities, always verify cleanup markers are present. Self-despawn is not sufficient — state transitions can interrupt the despawn lifecycle.

**Why:** The `NoEntityLeaks` scenario invariant should catch these, but adding the marker is the correct fix per `docs/architecture/standards.md`.

**How to apply:** When reviewing any effect that spawns entities, always verify cleanup markers are present. Self-despawn is not sufficient — state transitions can interrupt the despawn lifecycle.
