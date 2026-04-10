# Types

## EffectTimers (Component)

Component attached to entities that have `Until(TimeExpires(N), ...)` installed.

```
EffectTimers {
    timers: Vec<(OrderedFloat<f32>, OrderedFloat<f32>)>,
}
```

Each entry is `(remaining_seconds, original_duration)`.

- `remaining_seconds` -- decremented each frame by `dt`. When it reaches zero, the timer fires.
- `original_duration` -- preserved so the bridge can match `TimeExpires(original_duration)` in the trigger set.

Added by the tree walker when `Until(TimeExpires(N), ...)` is installed. The tree walker pushes `(N, N)` onto the `timers` vec.

## EffectTimerExpired (Message)

Sent by the `tick_effect_timers` game system when a timer reaches zero.

```
EffectTimerExpired { entity: Entity }
```
