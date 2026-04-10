# Name
on_time_expires

# Reads
`EffectTimerExpired` message.

# Dispatches
`Trigger::TimeExpires(original_duration)`

# Scope
Self (only the owning entity).

Walk the entity referenced in the message that has `TimeExpires(original_duration)` in its trigger set.

# TriggerContext
`TriggerContext::None`

# Source Location
`src/effect/triggers/time/bridges.rs`

# Schedule
FixedUpdate, after `tick_effect_timers`.

# Behavior
1. Read each `EffectTimerExpired { entity }` message.
2. Walk the trigger set on `entity` for `TimeExpires(original_duration)`.
3. For each match, invoke the tree walker with `TriggerContext::None`.
4. The walker handles the `Until` reversal logic -- the bridge only fires the trigger.

- Does NOT tick timers -- the `tick_effect_timers` game system does that.
- Does NOT reverse effects -- the walker does that when the `Until` condition matches.
- Does NOT modify `EffectTimers`, `BoundEffects`, or `StagedEffects`.
