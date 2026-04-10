# Name
on_perfect_bumped

# Reads
`BumpPerformed` message from `grade_bump`

# Dispatches
`PerfectBumped` trigger variant

# Scope
Local — walks only the bolt entity and the breaker entity from the message.

# TriggerContext
`TriggerContext::Bump { bolt: msg.bolt, breaker: msg.breaker }`

# Source Location
`src/effect/triggers/bump/bridges.rs`

# Schedule
FixedUpdate, in `EffectSystems::Bridge`, after `BreakerSystems::GradeBump`, with `run_if(in_state(NodeState::Playing))`

# Behavior
1. Read each `BumpPerformed` message.
2. If `msg.grade != BumpGrade::Perfect`, skip this message.
3. Build context: `TriggerContext::Bump { bolt: msg.bolt, breaker: msg.breaker }`.
4. If `msg.bolt` is `Some(bolt)`:
   a. Query bolt entity for `(Entity, &BoundEffects, &StagedEffects)`.
   b. Call `walk_effects(bolt, &Trigger::PerfectBumped, &context, bound, staged, &mut commands)`.
5. Query breaker entity for `(Entity, &BoundEffects, &StagedEffects)`.
6. Call `walk_effects(breaker, &Trigger::PerfectBumped, &context, bound, staged, &mut commands)`.

This bridge does NOT:
- Modify any entities or components directly — all mutations are deferred via commands
- Send any messages
- Decide bump grades
- Handle game logic
- Fire on non-Perfect grades
- Walk entities not involved in the bump
