# Name
on_no_bump_occurred

# Reads
No message exists yet. The current `no_bump.rs` is a placeholder stub. This bridge requires a new message (e.g., `BoltContactedBreakerNoBump` or equivalent) sent by the breaker domain when a bolt hits the breaker with no bump input active. See `types.md` for details.

# Dispatches
`NoBumpOccurred` trigger variant

# Scope
Global — walks all entities with `BoundEffects`/`StagedEffects`.

# TriggerContext
`TriggerContext::default()` — no participant context for global triggers.

# Source Location
`src/effect/bridges/bump.rs`

# Schedule
FixedUpdate, in `EffectSystems::Bridge`, after `BreakerSystems::GradeBump`, with `run_if(in_state(NodeState::Playing))`

# Behavior
1. Read each NoBump message (type TBD — blocked on breaker domain adding the message).
2. Build context: `TriggerContext::default()`.
3. Iterate all entities with `(Entity, &BoundEffects, &mut StagedEffects)`.
4. For each entity, call `evaluate_bound_effects` and `evaluate_staged_effects` with `Trigger::NoBumpOccurred`.

This bridge does NOT:
- Modify any entities
- Send any messages
- Decide whether a "no bump" occurred — that is the breaker domain's job
- Handle game logic
- Populate any context fields

Note: This bridge requires a new message from the breaker domain (e.g., `BoltContactedBreakerNoBump`). The breaker domain does not currently produce this message.
