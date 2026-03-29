---
name: Known cleanup marker gaps in effect domain
description: Pre-existing effect entities (gravity_well, shockwave) that self-despawn but lack CleanupOnNodeExit, risking entity leaks on node transition
type: project
---

Several effect entities rely on self-despawn (timer expiry or radius reaching max) but lack `CleanupOnNodeExit` markers. If a node transition occurs while these entities are alive, they leak.

**Affected files (as of 2026-03-28):**
- `effect/effects/gravity_well.rs` — GravityWellMarker entity, no cleanup marker
- `effect/effects/shockwave.rs` — Shockwave entity, no cleanup marker (flagged in Phase 4 review)

**Not affected (correctly handled):**
- `effect/effects/pulse.rs` — PulseRing entities have `CleanupOnNodeExit`
- `effect/effects/explode.rs` — ExplodeRequest entities have `CleanupOnNodeExit`
- `effect/effects/second_wind.rs` — Wall `#[require]` auto-inserts `CleanupOnNodeExit`
- `effect/effects/chain_bolt.rs` — ChainBolt entities have `CleanupOnNodeExit`
- `effect/effects/spawn_phantom.rs` — PhantomBolt entities have `CleanupOnNodeExit`

**Why:** The `NoEntityLeaks` scenario invariant should catch these, but adding the marker is the correct fix per `docs/architecture/standards.md`.

**How to apply:** When reviewing any effect that spawns entities, always verify cleanup markers are present. Self-despawn is not sufficient — state transitions can interrupt the despawn lifecycle.
