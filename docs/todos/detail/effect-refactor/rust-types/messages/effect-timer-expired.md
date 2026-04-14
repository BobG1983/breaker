# Name
EffectTimerExpired

# Struct
```rust
#[derive(Message, Clone, Debug)]
struct EffectTimerExpired {
    entity: Entity,
    original_duration: OrderedFloat<f32>,
}
```

# Location
`src/effect_v3/triggers/time/messages.rs`

# Description
Sent by `tick_effect_timers` when an entry in `EffectTimers` reaches zero. Read by the `on_time_expires` bridge which dispatches `Trigger::TimeExpires(original_duration)` on the entity (Self scope).

The `original_duration` field carries the timer's original duration so the bridge can construct the correct `TimeExpires` trigger variant. `tick_effect_timers` must include this value in the message before removing the expired entry from `EffectTimers`.
