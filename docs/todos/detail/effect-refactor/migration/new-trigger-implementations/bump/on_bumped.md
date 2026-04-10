# Name
on_bumped

# Reads
`BumpPerformed` message from `grade_bump`

# Dispatches
`Bumped` trigger variant

# Scope
Local — walks only the bolt entity and the breaker entity from the message.

# TriggerContext
For bolt entity: `TriggerContext { breaker: Some(msg.breaker), ..default() }`
For breaker entity: `TriggerContext { bolt: msg.bolt, ..default() }` (bolt may be None if msg.bolt is None — skip bolt walk in that case)

Note: The current implementation only walks the bolt entity (via `query.get_mut(bolt)`). The new implementation should walk both bolt and breaker as participants. If `msg.bolt` is `None`, skip the bolt walk entirely but still walk the breaker.

# Source Location
`src/effect/bridges/bump.rs`

# Schedule
FixedUpdate, in `EffectSystems::Bridge`, after `BreakerSystems::GradeBump`, with `run_if(in_state(NodeState::Playing))`

# Behavior
1. Read each `BumpPerformed` message.
2. Check grade: if grade is `Perfect`, `Early`, or `Late` (any success), proceed. This trigger fires on ALL successful bumps regardless of timing grade.
3. If `msg.bolt` is `Some(bolt)`:
   a. Query bolt entity for `(Entity, &BoundEffects, &mut StagedEffects)`.
   b. Build context with `breaker: Some(msg.breaker)`.
   c. Call `evaluate_bound_effects` and `evaluate_staged_effects` with `Trigger::Bumped` on the bolt entity.
4. Query breaker entity (`msg.breaker`) for `(Entity, &BoundEffects, &mut StagedEffects)`.
5. Build context with `bolt: msg.bolt`.
6. Call `evaluate_bound_effects` and `evaluate_staged_effects` with `Trigger::Bumped` on the breaker entity.

This bridge does NOT:
- Modify any entities
- Send any messages
- Decide bump grades
- Handle game logic
- Fire on `Whiff` or `NoBump` grades
- Walk entities not involved in the bump
