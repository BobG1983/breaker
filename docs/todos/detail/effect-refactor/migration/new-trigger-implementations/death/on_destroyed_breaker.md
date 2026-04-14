# Name
on_destroyed::\<Breaker\>

# Reads
`Destroyed<Breaker>` message from the unified death pipeline.

# Dispatches
Three triggers per death:
1. `Trigger::Died` — on victim
2. `Trigger::Killed(Breaker)` — on killer (if present)
3. `Trigger::DeathOccurred(Breaker)` — globally

# Scope
Mixed:
- `Died` — Local, on victim entity only.
- `Killed(Breaker)` — Local, on killer entity only.
- `DeathOccurred(Breaker)` — Global, walks all entities with BoundEffects/StagedEffects.

# TriggerContext
`TriggerContext::Death { victim, killer }` for all three triggers. `killer` is `Some(entity)` when a killer exists, `None` for environmental deaths.

# Source Location
`src/effect_v3/triggers/death/bridges.rs` — same generic system, monomorphized for Breaker. Registered by EffectV3Plugin.

# Schedule
FixedUpdate, in `EffectV3Systems::Bridge`, after domain kill handlers have sent `Destroyed<T>`.

# Behavior
1. Read each `Destroyed<Breaker>` message.
2. Dispatch `Died` trigger on `msg.victim` (Local, victim only). Context: `Death { victim: msg.victim, killer: msg.killer }`.
3. If `msg.killer` is `Some(killer)`:
   a. Classify the killer entity's type at runtime.
   b. Dispatch `Killed(Breaker)` trigger on `killer` (Local, killer only). Same context.
4. If `msg.killer` is `Some(killer)` but the killer entity no longer exists in the world, skip step 3 with a debug warning.
5. Dispatch `DeathOccurred(Breaker)` trigger globally on all entities with BoundEffects/StagedEffects. Same context.

Died fires before Killed fires before DeathOccurred — local triggers resolve before global.

Breaker deaths are typically environmental (all lives lost from bolt loss), so killer is usually None and step 3 is skipped.

This bridge does NOT:
- Despawn any entity. Despawn happens via DespawnEntity in PostFixedUpdate.
- Deal damage or decrement Hp.
- Modify any components.
- Send any messages.
- Fire Killed when killer is None (environmental death). Died and DeathOccurred still fire.
