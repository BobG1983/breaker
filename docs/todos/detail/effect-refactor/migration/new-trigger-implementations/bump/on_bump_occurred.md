# Name
on_bump_occurred

# Reads
`BumpPerformed` message from `grade_bump`

# Dispatches
`BumpOccurred` trigger variant

# Scope
Global — walks all entities with `BoundEffects`/`StagedEffects`.

# TriggerContext
`TriggerContext { bolt: msg.bolt, breaker: Some(msg.breaker), ..default() }`

Note: Global bump triggers DO populate bolt and breaker context (unlike global impact triggers). This matches the current `bridge_bump` implementation, which sets `bolt: msg.bolt, breaker: Some(msg.breaker)` on the context. This allows `On(Bolt)` and `On(Breaker)` target resolution within global bump trigger trees.

# Source Location
`src/effect/bridges/bump.rs`

# Schedule
FixedUpdate, in `EffectSystems::Bridge`, after `BreakerSystems::GradeBump`, with `run_if(in_state(NodeState::Playing))`

# Behavior
1. Read each `BumpPerformed` message.
2. Check grade: if grade is `Perfect`, `Early`, or `Late` (any success), proceed. Always fires alongside exactly one timing-graded global variant.
3. Build context: `TriggerContext { bolt: msg.bolt, breaker: Some(msg.breaker), ..default() }`.
4. Iterate all entities with `(Entity, &BoundEffects, &mut StagedEffects)`.
5. For each entity, call `evaluate_bound_effects` and `evaluate_staged_effects` with `Trigger::BumpOccurred`.

This bridge does NOT:
- Modify any entities
- Send any messages
- Decide bump grades
- Handle game logic
- Fire on `Whiff` or `NoBump` grades
