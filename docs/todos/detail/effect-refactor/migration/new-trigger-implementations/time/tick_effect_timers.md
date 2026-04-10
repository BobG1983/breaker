# Name
tick_effect_timers

# Reads
`EffectTimers` component on entities.
`Time<Fixed>` resource (for `dt`).

# Dispatches
Nothing -- this is a game system, not a bridge. Sends `EffectTimerExpired` message.

# Scope
N/A (game system).

# TriggerContext
N/A (game system).

# Source Location
`src/effect/triggers/time/tick_timers.rs`

# Schedule
FixedUpdate.

# Behavior
1. Query all entities with `EffectTimers` component.
2. For each entity, iterate over the `timers` vec:
   a. Decrement `remaining_seconds` by `dt`.
   b. If `remaining_seconds <= 0.0`:
      - Send `EffectTimerExpired { entity }`.
      - Mark the timer entry for removal.
3. Remove all expired timer entries from the vec.
4. If the `timers` vec is now empty, remove the `EffectTimers` component from the entity.

- Does NOT dispatch triggers -- the `on_time_expires` bridge does that.
- Does NOT modify `BoundEffects` or `StagedEffects`.
- Does NOT reverse effects -- the walker does that when `Until` matches.
