# Name
on_bump_occurred

# Reads
`BumpPerformed` message from `grade_bump`

# Dispatches
`BumpOccurred` trigger variant

# Scope
Global — walks all entities with `BoundEffects`/`StagedEffects`.

# TriggerContext
`TriggerContext::Bump { bolt: msg.bolt, breaker: msg.breaker }`

All global triggers populate their participant context.

# Source Location
`src/effect/triggers/bump/bridges.rs`

# Schedule
FixedUpdate, in `EffectSystems::Bridge`, after `BreakerSystems::GradeBump`, with `run_if(in_state(NodeState::Playing))`

# Behavior
1. Read each `BumpPerformed` message.
2. Check grade: if grade is `Perfect`, `Early`, or `Late` (any success), proceed. Always fires alongside exactly one timing-graded global variant.
3. Build context: `TriggerContext::Bump { bolt: msg.bolt, breaker: msg.breaker }`.
4. Iterate all entities with `(Entity, &BoundEffects, &StagedEffects)`.
5. For each entity, call `walk_effects(entity, &Trigger::BumpOccurred, &context, bound, staged, &mut commands)`.

This bridge does NOT:
- Modify any entities or components directly — all mutations are deferred via commands
- Send any messages
- Decide bump grades
- Handle game logic
- Fire on `Whiff` or `NoBump` grades
