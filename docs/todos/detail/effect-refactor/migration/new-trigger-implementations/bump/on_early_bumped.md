# Name
on_early_bumped

# Reads
`BumpPerformed` message from `grade_bump`

# Dispatches
`EarlyBumped` trigger variant

# Scope
Local — walks only the bolt entity and the breaker entity from the message.

# TriggerContext
`TriggerContext::Bump { bolt: msg.bolt, breaker: msg.breaker }`

# Source Location
`src/effect_v3/triggers/bump/bridges.rs`

# Schedule
FixedUpdate, in `EffectV3Systems::Bridge`, after `BreakerSystems::GradeBump`, with `run_if(in_state(NodeState::Playing))`

# Behavior
1. Read each `BumpPerformed` message.
2. If `msg.grade != BumpGrade::Early`, skip this message.
3. Build context: `TriggerContext::Bump { bolt: msg.bolt, breaker: msg.breaker }`.
4. If `msg.bolt` is `Some(bolt)`:
   a. Query bolt entity for `(Entity, &BoundEffects, &StagedEffects)`.
   b. Call `walk_effects(bolt, &Trigger::EarlyBumped, &context, bound, staged, &mut commands)`.
5. Query breaker entity for `(Entity, &BoundEffects, &StagedEffects)`.
6. Call `walk_effects(breaker, &Trigger::EarlyBumped, &context, bound, staged, &mut commands)`.

This bridge does NOT:
- Modify any entities or components directly — all mutations are deferred via commands
- Send any messages
- Decide bump grades
- Handle game logic
- Fire on non-Early grades
- Walk entities not involved in the bump
