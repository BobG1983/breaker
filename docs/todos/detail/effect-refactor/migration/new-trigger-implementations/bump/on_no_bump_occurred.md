# Name
on_no_bump_occurred

# Reads
`BoltImpactBreaker` message where `bump_status == BumpStatus::Inactive`. See `types.md` for the migration adding `bump_status` to `BoltImpactBreaker`.

# Dispatches
`NoBumpOccurred` trigger variant

# Scope
Global — walks all entities with `BoundEffects`/`StagedEffects`.

# TriggerContext
`TriggerContext::Bump { bolt: msg.bolt, breaker: msg.breaker }` — populated with bolt and breaker from the collision message.

# Source Location
`src/effect_v3/triggers/bump/bridges.rs`

# Schedule
FixedUpdate, in `EffectV3Systems::Bridge`, after `BoltSystems::BreakerCollision`, with `run_if(in_state(NodeState::Playing))`

# Behavior
1. Read each `BoltImpactBreaker` message.
2. If `bump_status` is `BumpStatus::Active`, skip — this is a bump event handled by the other bump bridges.
3. Build context: `TriggerContext::Bump { bolt: msg.bolt, breaker: msg.breaker }`.
4. Walk all entities with BoundEffects/StagedEffects with `Trigger::NoBumpOccurred` and the context.

This bridge does NOT:
- Modify any entities
- Send any messages
- Decide whether bump was active — the collision system sets `bump_status`
- Handle game logic
