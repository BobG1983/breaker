---
name: TetherBeam standard mode bolt accumulation
description: fire_standard() spawns 2 extra bolts per Bumped event with no mid-node cleanup, causing unbounded BoltCountReasonable violations in tether_beam_stress.
type: project
---

## Bug: TetherBeam Bolts Never Cleaned Up Mid-Node

**Invariant:** BoltCountReasonable
**Scenario:** `tether_beam_stress` — bolts=51 at frame 4143..4866, max_bolt_count=16

**Root cause:**

`fire_standard()` in `breaker-game/src/effect/effects/tether_beam/effect.rs` spawns two tether bolts via `Bolt::builder()` (NOTE: `spawn_extra_bolt` was removed in builder migration), inserting `TetherBoltMarker` on both. These bolts have `CleanupOnNodeExit` so they are removed at node end, but there is **no system that despawns tether bolts mid-node**.

`tick_tether_beam` despawns the beam entity when `bolt_a` or `bolt_b` goes missing — but it does NOT despawn the surviving bolt. The beam is linked to two specific entity IDs; if one bolt falls out of bounds (BoltLost), the other persists permanently as an orphan.

Additionally, with `When(trigger: Bumped)` in the scenario, every bump spawns 2 new bolts. The Dense layout + chaos 0.5 input generates 50+ bumps before node end, accumulating 100+ tether bolts that bounce around inside the play area (they rarely fall out of bounds).

**Why this happens but tests don't catch it:** Unit tests for `fire_standard` verify that 2 `TetherBoltMarker` entities are spawned and that the beam component references them. No test checks that these bolts are removed when the beam despawns.

**Fix location:** `breaker-game/src/effect/effects/tether_beam/effect.rs`
- In `tick_tether_beam`, when despawning the beam entity because a bolt is missing, also despawn the surviving bolt.
- OR: add a cleanup system that despawns `TetherBoltMarker` bolts when their paired beam no longer exists.
