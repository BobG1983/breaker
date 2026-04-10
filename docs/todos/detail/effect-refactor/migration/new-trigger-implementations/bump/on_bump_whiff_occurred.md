# Name
on_bump_whiff_occurred

# Reads
`BumpWhiffed` message from `grade_bump`

# Dispatches
`BumpWhiffOccurred` trigger variant

# Scope
Global — walks all entities with `BoundEffects`/`StagedEffects`.

# TriggerContext
`TriggerContext::default()` — no participant entities. BumpWhiffed is a unit message with no entity fields.

# Source Location
`src/effect/triggers/bump/bridges.rs`

# Schedule
FixedUpdate, in `EffectSystems::Bridge`, after `BreakerSystems::GradeBump`, with `run_if(in_state(NodeState::Playing))`

# Behavior
1. Read each `BumpWhiffed` message (unit struct, no fields).
2. Build context: `TriggerContext::default()`.
3. Iterate all entities with `(Entity, &BoundEffects, &mut StagedEffects)`.
4. For each entity, call `evaluate_bound_effects` and `evaluate_staged_effects` with `Trigger::BumpWhiffOccurred`.

This bridge does NOT:
- Modify any entities
- Send any messages
- Decide bump grades
- Handle game logic
- Populate any context fields (no participants in a whiff)
