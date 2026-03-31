---
name: EffectiveSpeedConsistent / SizeBoostInRange state-gate mismatch
description: recalculate_speed and recalculate_size run only in PlayingState::Active, but fire()/reverse() can update Active* components in any state (e.g. node transitions). Invariant checkers see a one-frame divergence between Active* and Effective* whenever a boost changes state outside PlayingState::Active.
type: project
---

`recalculate_speed` and `recalculate_size` are registered with:
```rust
app.configure_sets(FixedUpdate, EffectSystems::Recalculate.run_if(in_state(PlayingState::Active)));
```
(`breaker-game/src/effect/plugin.rs`)

But `fire()` and `reverse()` in `speed_boost.rs` and `size_boost.rs` push/pop directly onto `ActiveSpeedBoosts`/`ActiveSizeBoosts` via world access. These are called from the effect dispatch system which can fire during node transitions, `Until(NodeEnd)` cleanup, chip select, and other non-Active states.

The invariant checkers (`check_effective_speed_consistent`, `check_size_boost_in_range`) are gated only on `playing_gate` (i.e., `stats.entered_playing`), NOT on `PlayingState::Active`. So in any frame where `fire()`/`reverse()` fires outside `PlayingState::Active`, the checker sees `EffectiveSpeedMultiplier` diverged from the product of `ActiveSpeedBoosts` — because `recalculate_speed` did not run.

**Evidence:** `node_end_speed_purge` scenario fires `EffectiveSpeedConsistent` x27 at frames 127..19474, using `Until(NodeEnd)` scoped `SpeedBoost(1.3)`. On each node transition, `reverse()` cleans up the boost but `recalculate_speed` skips (wrong state) — leaving `EffectiveSpeedMultiplier` stale until the next `PlayingState::Active` tick.

**Downstream:** `BoltSpeedInRange` fires as a downstream consequence — once `EffectiveSpeedMultiplier` is stale from a missed recalculation, bolt speeds may exceed or fall below the [min, max] clamp range.

**Affected scenarios:** ~38 gameplay scenarios including `node_end_speed_purge`, `breaker_impact_trigger_chaos`, `aegis_speed_bounce`, `damage_boost_until_reversal`, `entity_scale_collision_chaos`, etc.

**Fix location:** `breaker-game/src/effect/plugin.rs` — `EffectSystems::Recalculate` should not be gated on `PlayingState::Active`, OR `recalculate_speed`/`recalculate_size` should be called inline after `fire()`/`reverse()`.
