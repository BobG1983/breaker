# Name
on_bumped

# Reads
`BumpPerformed` message from `grade_bump`

# Dispatches
`Bumped` trigger variant

# Scope
Local — walks only the bolt entity and the breaker entity from the message.

# TriggerContext
`TriggerContext::Bump { bolt: msg.bolt, breaker: msg.breaker }`

If `msg.bolt` is `None`, skip the bolt walk entirely but still walk the breaker.

# Source Location
`src/effect_v3/triggers/bump/bridges.rs`

# Schedule
FixedUpdate, in `EffectV3Systems::Bridge`, after `BreakerSystems::GradeBump`, with `run_if(in_state(NodeState::Playing))`

# Behavior
1. Read each `BumpPerformed` message.
2. Check grade: if grade is `Perfect`, `Early`, or `Late` (any success), proceed. This trigger fires on ALL successful bumps regardless of timing grade.
3. Build context: `TriggerContext::Bump { bolt: msg.bolt, breaker: msg.breaker }`.
4. If `msg.bolt` is `Some(bolt)`:
   a. Query bolt entity for `(Entity, &BoundEffects, &StagedEffects)`.
   b. Call `walk_effects(bolt, &Trigger::Bumped, &context, bound, staged, &mut commands)`.
5. Query breaker entity for `(Entity, &BoundEffects, &StagedEffects)`.
6. Call `walk_effects(breaker, &Trigger::Bumped, &context, bound, staged, &mut commands)`.

This bridge does NOT:
- Modify any entities or components directly — all mutations are deferred via commands
- Send any messages
- Decide bump grades
- Handle game logic
- Fire on `Whiff` or `NoBump` grades
- Walk entities not involved in the bump
