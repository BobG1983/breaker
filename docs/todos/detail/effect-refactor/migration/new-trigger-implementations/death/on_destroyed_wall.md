# Name
on_destroyed::\<Wall\>

# Reads
`Destroyed<Wall>` message from the unified death pipeline.

# Dispatches
Three triggers per death:
1. `Trigger::Died` — on victim
2. `Trigger::Killed(Wall)` — on killer (if present)
3. `Trigger::DeathOccurred(Wall)` — globally

# Scope
Mixed:
- `Died` — Local, on victim entity only.
- `Killed(Wall)` — Local, on killer entity only.
- `DeathOccurred(Wall)` — Global, walks all entities with BoundEffects/StagedEffects.

# TriggerContext
`TriggerContext::Death { victim, killer }` for all three triggers. `killer` is `Some(entity)` when a killer exists, `None` for environmental deaths.

# Source Location
`src/effect_v3/triggers/death/bridges.rs` — same generic system, monomorphized for Wall. Registered by EffectV3Plugin.

# Schedule
FixedUpdate, in `EffectV3Systems::Bridge`, after domain kill handlers have sent `Destroyed<T>`.

# Behavior
1. Read each `Destroyed<Wall>` message.
2. Dispatch `Died` trigger on `msg.victim` (Local, victim only). Context: `Death { victim: msg.victim, killer: msg.killer }`.
3. If `msg.killer` is `Some(killer)`:
   a. Classify the killer entity's type at runtime.
   b. Dispatch `Killed(Wall)` trigger on `killer` (Local, killer only). Same context.
4. If `msg.killer` is `Some(killer)` but the killer entity no longer exists in the world, skip step 3 with a debug warning.
5. Dispatch `DeathOccurred(Wall)` trigger globally on all entities with BoundEffects/StagedEffects. Same context.

Died fires before Killed fires before DeathOccurred — local triggers resolve before global.

This bridge does NOT:
- Despawn any entity. Despawn happens via DespawnEntity in PostFixedUpdate.
- Deal damage or decrement Hp.
- Modify any components.
- Send any messages.
- Fire Killed when killer is None (environmental death). Died and DeathOccurred still fire.
