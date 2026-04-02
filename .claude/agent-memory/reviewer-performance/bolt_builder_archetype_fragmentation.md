---
name: Bolt builder archetype fragmentation from spawn_inner
description: spawn_inner uses 6-10 conditional .insert() calls that produce multiple archetypes per bolt variant — acceptable at 1-few bolt count
type: project
---

`spawn_inner` in `breaker-game/src/bolt/builder/core.rs` conditionally inserts several optional components after the initial `world.spawn(core)`:

- Role + cleanup (`PrimaryBolt + CleanupOnRunEnd` or `ExtraBolt + CleanupOnNodeExit`) — always present
- `BoltServing` — serving bolts only
- `BoltBaseDamage`, `BoltDefinitionRef`, `BoltAngleSpread`, `BoltSpawnOffsetY` — when `definition_params` is Some
- `MinRadius`, `MaxRadius` — only when definition specifies them
- `SpawnedByEvolution` — evolution-spawned bolts only
- `BoltLifespan` — lifespan bolts only
- `BoundEffects` — only when effects are present
- Rendered visual components (`Mesh2d`, `MeshMaterial2d`, `GameDrawLayer::Bolt`) — always done in a separate insert after spawn_inner returns

Each conditional insert that adds/removes a component creates a new archetype. This means bolt entities can land in different archetypes depending on their configuration, and each spawn causes multiple archetype transitions (each `.insert()` call is a separate archetype move).

**At current scale (1 bolt)**: totally acceptable. Zero per-frame impact; bolt spawn is a rare event. Multiple archetype transitions at spawn time are fine.

**At scale (dozens of bolts per second, e.g., chain reaction evolution)**: still fine — spawns are one-time events, not per-frame, and the archetype count stays bounded by the combination space. The `With<Bolt>` filter ensures systems only match bolt archetypes.

**How to apply:** Do not flag conditional inserts in spawn_inner as a concern. Flag only if bolt count scales to hundreds of *persistent* entities queried every FixedUpdate.
