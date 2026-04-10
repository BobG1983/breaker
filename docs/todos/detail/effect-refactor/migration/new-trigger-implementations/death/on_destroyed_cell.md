# Name
on_destroyed::\<Cell\>

# Reads
`Destroyed<Cell>` message from the unified death pipeline.

# Dispatches
Three triggers per death:
1. `Trigger::Died` — on victim
2. `Trigger::Killed(Cell)` — on killer (if present)
3. `Trigger::DeathOccurred(Cell)` — globally

# Scope
Mixed:
- `Died` — Local, on victim entity only.
- `Killed(Cell)` — Local, on killer entity only.
- `DeathOccurred(Cell)` — Global, walks all entities with BoundEffects/StagedEffects.

# TriggerContext
`TriggerContext::Death { victim, killer }` for all three triggers. `killer` is `Some(entity)` when a killer exists, `None` for environmental deaths.

# Source Location
`src/effect/triggers/death/bridges.rs` — generic system, monomorphized per T. Registered by EffectPlugin.

# Schedule
FixedUpdate, in `EffectSystems::Bridge`, after domain kill handlers have sent `Destroyed<T>`.

# Behavior
1. Read each `Destroyed<Cell>` message.
2. Dispatch `Died` trigger on `msg.victim` (Local, victim only). Context: `Death { victim: msg.victim, killer: msg.killer }`.
3. If `msg.killer` is `Some(killer)`:
   a. Classify the killer entity's type at runtime (inspect components for Bolt/Breaker/Cell/Wall).
   b. Dispatch `Killed(Cell)` trigger on `killer` (Local, killer only). Same context.
4. If `msg.killer` is `Some(killer)` but the killer entity no longer exists in the world, skip step 3 with a debug warning.
5. Dispatch `DeathOccurred(Cell)` trigger globally on all entities with BoundEffects/StagedEffects. Same context.

Died fires before Killed fires before DeathOccurred — local triggers resolve before global.

This bridge does NOT:
- Despawn any entity. Despawn happens via DespawnEntity in PostFixedUpdate.
- Deal damage or decrement Hp.
- Modify any components.
- Send any messages.
- Fire Killed when killer is None (environmental death). Died and DeathOccurred still fire.
