---
name: Wave 1 scenario-coverage review (2026-03-30)
description: Review of stat-boost lazy init (5 files) and GravityWell/SpawnPhantom FIFO (2 files)
type: project
---

## Findings

### Confirmed bugs

1. `gravity_well/effect.rs` — missing `world.get_entity(entity).is_err()` early-return guard.
   All 5 stat-boost fire() functions have this guard. gravity_well and spawn_phantom do not.
   Impact: calling fire() for a despawned entity spawns a ghost well with PhantomOwner(dead_entity),
   pollutes GravityWellSpawnCounter with a dead entity key, and the ghost well's FIFO slot counts
   against the owner's cap even though it can never be cleaned up correctly.

2. `spawn_phantom/effect.rs` — same missing despawned-entity guard.
   Same impact pattern with PhantomOwner(dead_entity) and PhantomSpawnCounter pollution.

### Confirmed correct (do not re-flag)

- Stat-boost lazy-init pattern (speed_boost, damage_boost, size_boost, bump_force, piercing):
  - Two-step guard (check ActiveBoosts, then check Effective*) is safe — Bevy insert is idempotent for new components.
  - The `if let Some(mut active) = world.get_mut::<Active*>` at the end always succeeds because the component was inserted in the step above.
  - The "has Effective* but not Active*" overwrite concern is theoretical only — fire() always inserts both, there are no remove::<Active*> calls in the codebase.

- GravityWell FIFO logic:
  - `owned.len() - despawn_list.len() >= max as usize` — no usize underflow, loop terminates.
  - `owned[despawn_list.len()]` access — always valid because loop condition guarantees owned.len() > despawn_list.len().
  - Counter read-before-spawn, increment-after-spawn is correct FIFO ordering.
  - `if let Some` guard on SCOPE C is safe — resource was inserted by SCOPE A (exclusive &mut World).

- SpawnPhantom FIFO logic:
  - `while owned.len() >= max_active as usize` with `owned.remove(0)` — correct, terminates.
  - World::despawn inside loop on collected Vec is safe (not iterating live query).

## Status: 2 confirmed bugs filed as regression spec hints
