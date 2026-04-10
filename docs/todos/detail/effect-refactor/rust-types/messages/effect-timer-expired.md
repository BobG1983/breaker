# Name
EffectTimerExpired

# Struct
```rust
#[derive(Message, Clone, Debug)]
struct EffectTimerExpired {
    entity: Entity,
}
```

# Location
`src/effect/triggers/time/messages.rs`

# Description
Sent by `tick_effect_timers` when an entry in `EffectTimers` reaches zero. Read by the `on_time_expires` bridge which dispatches `TimeExpires` on the entity (Self scope).

Contains only the entity — the bridge matches the trigger against the entity's BoundEffects/StagedEffects.
