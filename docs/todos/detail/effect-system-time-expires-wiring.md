# Effect system: wire `Until(TimeExpires(...), ...)` scope entry to install `EffectTimers`

## Problems addressed

- `audit/protocols/kickstart.md` Issue 2 \u2014 `Until(TimeExpires(3.0), ...)` never reverses because no production path arms `EffectTimers`. This remediation wires the missing half. Kickstart and Ricochet remain code-driven (tuning-only RON) per user direction; this wiring is for ANY future consumer of `Until(TimeExpires(...), ...)`.
- `audit/protocols/kickstart.md` Issue 6 \u2014 Declared-but-unimplemented timer-arm capability. Same root cause, same fix.
- `audit/protocols/ricochet.md` (verification finding) \u2014 User confirmed "Until(TimeExpires) SHOULD be wired" during Ricochet interrogation. This bug is a latent blocker for any RON-declared effect tree that uses `Until(TimeExpires(...), ...)`.

## How this shipped broken

This is a coverage-gap lesson. The individual pieces have good tests:

- `tick_effect_timers` has thorough unit tests in `triggers/time/tick_timers.rs`.
- `on_time_expires` bridge has thorough tests in `triggers/time/bridges/tests.rs`, including "bridge dispatches TimeExpires trigger when EffectTimerExpired received" and "non-matching duration does not trigger."

Each component works correctly in isolation. But no integration test ever asserted the end-to-end path: "RON declares `Until(TimeExpires(3.0), ...)` \u2192 scope is entered \u2192 `EffectTimers` is installed \u2192 timer ticks down \u2192 `EffectTimerExpired` fires \u2192 bridge dispatches \u2192 scope reverses." The missing middle step (install EffectTimers on scope entry) was never written, and no test caught it because every test that could have caught it spawned `EffectTimers` manually as a fixture.

Remediation adds the scope-entry integration test that would have caught this at ship-time. Treat this as a template for every other "declared in RON but maybe not wired" effect primitive \u2014 search for similar gaps in the audit.

## Remediation

### Current state

- `EffectTimers` component exists in `breaker-game/src/effect_v3/triggers/time/components.rs`.
- `tick_effect_timers` decrements entries and emits `EffectTimerExpired` when one reaches zero (`triggers/time/tick_timers.rs`).
- `on_time_expires` bridge reads `EffectTimerExpired` and walks the trees on the entity with a `TimeExpires(original_duration)` trigger (`triggers/time/bridges/system.rs`).
- `Trigger::TimeExpires(OrderedFloat<f32>)` variant exists in `types/trigger.rs`.

What's missing: the tree walker (wherever `Until` scopes are entered) never calls `commands.entity(e).insert(EffectTimers { .. })`. So `TimeExpires` triggers never fire in production because no timer is ever installed.

### The fix

Locate the walker code that handles entering an `Until(trigger, inner)` scope \u2014 likely `conditions/evaluate_conditions/system.rs` or `walking/walk_*.rs`. When the walker encounters `Until(Trigger::TimeExpires(duration), inner_tree)` and the scope is entered (the triggering outer `When(...)` fired), install a timer:

```rust
// On entering the Until scope:
let mut timers_entry = world
    .entity_mut(entity)
    .get_mut::<EffectTimers>()
    .map(|mut comp| comp)
    .unwrap_or_else(|| {
        world.entity_mut(entity).insert(EffectTimers { timers: Vec::new() });
        world.entity_mut(entity).get_mut::<EffectTimers>().unwrap()
    });
timers_entry.timers.push((duration, duration));
```

(Pseudocode \u2014 actual implementation uses `commands.entity(...).insert(...)` or direct world mutation depending on the walker's context; match the surrounding pattern.)

On scope EXIT (the Until trigger fired OR the Until was canceled), remove the matching entry from `EffectTimers`. Match by `original_duration` and the `source` key if the walker tracks it \u2014 same scheme as the `armed_key` pattern in `conditions/evaluate_conditions/system.rs`. If `EffectTimers.timers` becomes empty, the component is auto-removed by `tick_effect_timers` on the next tick (existing behavior at `tick_timers.rs:43-45`).

Additionally: the `Until` walker tracks which timer entry belongs to which scope. The existing `armed_key` machinery in `evaluate_conditions/system.rs` provides a naming scheme; extend it to include timer-entry keys so cancellation can find the right timer to remove.

### Trigger-emit note

`tick_effect_timers` already emits `EffectTimerExpired` with `original_duration`. The `on_time_expires` bridge walks trees with `Trigger::TimeExpires(original_duration)`. Trees matching that trigger for that duration fire their inner \u2014 existing behavior covered by `bridges/tests.rs::behavior 13`. No change to the tick or bridge needed.

### Consequence for Kickstart and Ricochet

Per user direction, Kickstart and Ricochet remain code-driven with tuning-only RON. They use `commands.fire_effect(...)` / `commands.reverse_effect(...)` / `commands.stamp_effect(...)` directly and manage their countdowns in protocol-specific code. They do NOT regress to RON effect trees.

This remediation wires the capability for ANY future consumer that declares `Until(TimeExpires(duration), ...)` in RON \u2014 not specifically for Kickstart/Ricochet. Without this fix, such a RON declaration would silently no-op.

### Tests

Add `breaker-game/src/effect_v3/walking/tests/until_time_expires_arms_timer.rs` (or wherever Until-walker tests live):

1. Spawn an entity with a `BoundEffects` tree containing `When(NodeStartOccurred, Until(TimeExpires(3.0), Fire(SpeedBoost(2.0))))`.
2. Drive `OnEnter(NodeState::Playing)` (fires `NodeStartOccurred`).
3. Assert the entity has `EffectTimers` with one entry `(OrderedFloat(3.0), OrderedFloat(3.0))`.
4. Assert the entity's `EffectStack<SpeedBoostConfig>` contains one entry with multiplier 2.0.
5. Advance the fixed timestep by 3 seconds.
6. Assert `EffectTimers` is removed (or empty) and `EffectStack<SpeedBoostConfig>` no longer contains the entry (scope reversal fired).

Add a cancellation test `until_time_expires_cancels_on_outer_reversal.rs`:

1. Spawn entity with `Until(TimeExpires(5.0), Fire(...))` nested under a `During(...)` that goes false before 5 seconds elapse.
2. Drive the outer reversal.
3. Assert the timer entry is removed from `EffectTimers` mid-flight.
4. Assert the inner fire is reversed.

### Fallout on existing scenarios

Any RON file that currently declares `Until(TimeExpires(...), ...)` and has silently been broken will start working after this remediation. Search for all such RON declarations:

```
grep -rn "TimeExpires" breaker-game/assets/
```

For each hit, verify the intended behavior is still correct. If an asset was relying on the silent no-op, update the asset. If an asset is genuinely broken and the fix works correctly, great \u2014 the remediation uncovered a latent bug.

The current Kickstart RON (which will be rewritten to tuning-only per `kickstart-code-driven-effects.md`) was the primary known consumer. Once that migration lands in parallel, the RON side is quiet.
