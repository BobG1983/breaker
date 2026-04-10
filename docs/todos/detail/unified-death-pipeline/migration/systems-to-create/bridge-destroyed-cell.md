# Name
bridge_destroyed::\<Cell\>

# SystemSet
New: `EffectSystems::Bridge`. Runs in FixedUpdate, after domain kill handlers have sent `Destroyed<T>`.

# Filepath
`src/effect/bridges/destroyed.rs` — lives in the effect domain because it calls `walk_effects`. Registered by EffectPlugin, not the death pipeline plugin. — generic system, monomorphized per T.

# Queries/Filters
No queries — reads `Destroyed<Cell>` messages only. Dispatches triggers via the effect system's walking algorithm.

# Description
Read all `Destroyed<Cell>` messages. For each:

1. Dispatch `Died` trigger on the victim entity (Local, on victim only). Trigger context: `Death { victim, killer }`.
2. If killer is Some, classify the killer entity's type at runtime (inspect components for Bolt/Breaker/Cell/Wall). Dispatch `Killed(Cell)` trigger on the killer entity (Local, on killer only). Same trigger context.
3. Dispatch `DeathOccurred(Cell)` trigger globally on all entities with BoundEffects/StagedEffects. Same trigger context.

If killer is None (environmental death), skip step 2 — Killed is not fired. Died and DeathOccurred still fire.

If the killer entity no longer exists in the world (despawned between damage and bridge), skip step 2 with a debug warning.

DO fire Died before Killed before DeathOccurred — local triggers resolve before global.
DO NOT despawn the entity. That happens via DespawnEntity in PostFixedUpdate.
