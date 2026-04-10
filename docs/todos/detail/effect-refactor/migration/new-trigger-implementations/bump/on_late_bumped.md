# Name
on_late_bumped

# Reads
`BumpPerformed` message from `grade_bump`

# Dispatches
`LateBumped` trigger variant

# Scope
Local — walks only the bolt entity and the breaker entity from the message.

# TriggerContext
For bolt entity: `TriggerContext { breaker: Some(msg.breaker), ..default() }`
For breaker entity: `TriggerContext { bolt: msg.bolt, ..default() }` (bolt may be None — skip bolt walk in that case)

# Source Location
`src/effect/bridges/bump.rs`

# Schedule
FixedUpdate, in `EffectSystems::Bridge`, after `BreakerSystems::GradeBump`, with `run_if(in_state(NodeState::Playing))`

# Behavior
1. Read each `BumpPerformed` message.
2. If `msg.grade != BumpGrade::Late`, skip this message.
3. If `msg.bolt` is `Some(bolt)`:
   a. Query bolt entity for `(Entity, &BoundEffects, &mut StagedEffects)`.
   b. Build context with `breaker: Some(msg.breaker)`.
   c. Call `evaluate_bound_effects` and `evaluate_staged_effects` with `Trigger::LateBumped` on the bolt entity.
4. Query breaker entity (`msg.breaker`) for `(Entity, &BoundEffects, &mut StagedEffects)`.
5. Build context with `bolt: msg.bolt`.
6. Call `evaluate_bound_effects` and `evaluate_staged_effects` with `Trigger::LateBumped` on the breaker entity.

This bridge does NOT:
- Modify any entities
- Send any messages
- Decide bump grades
- Handle game logic
- Fire on non-Late grades
- Walk entities not involved in the bump
