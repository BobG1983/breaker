# spawn_bolt → setup_run Migration

## Summary
Eliminate `spawn_bolt` system. Primary bolt should spawn once at run start in `setup_run`, not on every `OnEnter(GameState::Playing)`.

## Context
The current flow re-spawns the bolt on every state entry (including returning from chip select), requiring guards against duplicate bolts. The bolt is conceptually per-run, not per-node — `reset_bolt` already handles repositioning between nodes. This is a lifecycle correctness issue that makes the bolt spawn path unnecessarily complex.

## Scope
- In: `setup_run` spawns primary bolt once with `CleanupOnRunEnd`, `reset_bolt` repositions at node transitions, delete `spawn_bolt` system entirely, render assets (Mesh2d, MeshMaterial2d) inserted during setup_run
- Out: Effect-spawned bolts (those use the builder, not this system)

## Dependencies
- Depends on: Bolt builder being stable (it is — already migrated)
- Blocks: Bolt birthing animation (needs spawn lifecycle clarity first)

## Notes
Current wrong flow:
- `spawn_bolt` runs on `OnEnter(GameState::Playing)` — re-enters after chip select
- Needs guards to avoid duplicate bolts
- Conceptually spawning per-node when the bolt is per-run

Target flow:
- `setup_run` spawns primary bolt once with `CleanupOnRunEnd`
- `reset_bolt` repositions at each node transition
- `spawn_bolt` system deleted entirely

## Status
`ready`
