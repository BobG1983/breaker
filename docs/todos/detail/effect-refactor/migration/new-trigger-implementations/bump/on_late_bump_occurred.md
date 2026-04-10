# Name
on_late_bump_occurred

# Reads
`BumpPerformed` message from `grade_bump`

# Dispatches
`LateBumpOccurred` trigger variant

# Scope
Global — walks all entities with `BoundEffects`/`StagedEffects`.

# TriggerContext
`TriggerContext { bolt: msg.bolt, breaker: Some(msg.breaker), ..default() }`

# Source Location
`src/effect/triggers/bump/bridges.rs`

# Schedule
FixedUpdate, in `EffectSystems::Bridge`, after `BreakerSystems::GradeBump`, with `run_if(in_state(NodeState::Playing))`

# Behavior
1. Read each `BumpPerformed` message.
2. If `msg.grade != BumpGrade::Late`, skip this message.
3. Build context: `TriggerContext { bolt: msg.bolt, breaker: Some(msg.breaker), ..default() }`.
4. Iterate all entities with `(Entity, &BoundEffects, &mut StagedEffects)`.
5. For each entity, call `evaluate_bound_effects` and `evaluate_staged_effects` with `Trigger::LateBumpOccurred`.

This bridge does NOT:
- Modify any entities
- Send any messages
- Decide bump grades
- Handle game logic
- Fire on non-Late grades
