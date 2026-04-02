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

## Needs Detail

### 1. What is `setup_run`?
No system or schedule point by this name exists today. Options:
- **(a)** New system on `OnExit(GameState::RunSetup)` — bolt spawns before first `Playing` entry, during the run-setup-to-playing transition
- **(b)** Follow breaker's `spawn_or_reuse_breaker` pattern — rename to `spawn_or_reuse_bolt` on `OnEnter(Playing)`, convert from `&mut World` to regular system, keep the guard

These have very different scheduling and message-flow implications.

### 2. `BoltSpawned` message on subsequent nodes
`check_spawn_complete` (in `run/node/systems/`) needs `BoltSpawned` on **every node entry** to fire `SpawnNodeComplete`. If bolt spawns once at run start (option a above), who sends `BoltSpawned` on nodes 2+? Options:
- `reset_bolt` takes over sending `BoltSpawned`
- `check_spawn_complete` drops the `BoltSpawned` requirement and only checks cells/walls/breaker
- A new "bolt ready" message replaces `BoltSpawned` for the reuse case

### 3. Match breaker pattern or diverge?
Breaker uses "spawn or reuse" on `OnEnter(Playing)` and sends `BreakerSpawned` every time — even when reusing. Bolt could mirror this exactly (cleanest, lowest risk, consistent pattern) or use a different lifecycle. What's the intent?

## Status
`[NEEDS DETAIL]` — 3 open design questions above must be answered before planning
