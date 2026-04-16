---
name: speed-boost-checker-ordering
description: One-frame lag between EffectStack<SpeedBoostConfig> update and bolt speed — root cause and the two-option fix
type: project
---

## Root Cause: `EffectStack<SpeedBoostConfig>` Updated After `apply_velocity_formula` Runs

When a speed-boost effect fires on a collision frame, the checker sees the new
multiplier but the bolt's velocity still reflects the old one. This is a genuine
one-frame lag, not a false positive.

**Why:** `fire()` / `reverse()` in `speed_boost.rs` now call `recalculate_velocity()` INLINE
(added in feature/chip-evolution-ecosystem). This means speed_boost.rs now implements a
variant of Option B (velocity recalculated after EffectStack<SpeedBoostConfig> changes). However, the
commands are still queued via `FireEffectCommand` and applied *after the bridge returns*.
The bridge systems run in `EffectSystems::Bridge`, which is ordered **after**
`BoltSystems::CellCollision`, `BoltSystems::WallCollision`, and
`BoltSystems::BreakerCollision`. Each collision system calls `apply_velocity_formula`
before the bridge runs. The `recalculate_velocity()` inline call runs WITHIN the command
apply phase — so velocity IS updated immediately when the boost fires, resolving the lag.

**Why:** The checkers run `.before(BoltSystems::BoltLost)` and have no explicit ordering
relative to `EffectSystems::Bridge`. In practice Bevy runs the checkers in the same
FixedUpdate pass, and the checkers can run after the bridge commands flush —
so the checker sees `EffectStack<SpeedBoostConfig>` updated to the new multiplier while the
bolt velocity is still the pre-bridge value.

## Full ordering chain (single collision frame)

```
FixedUpdate:
  1. BoltSystems::CellCollision    — calls apply_velocity_formula (old multiplier)
     BoltSystems::WallCollision    — calls apply_velocity_formula (old multiplier)
     BoltSystems::BreakerCollision — calls apply_velocity_formula (old multiplier)
  2. EffectSystems::Bridge         — bridge_impact_* runs, queues FireEffectCommand
     [command flush after bridge]  — FireEffectCommand::apply() pushes onto
                                      EffectStack<SpeedBoostConfig> (NOW UPDATED)
  3. checkers_a chain              — check_bolt_speed_accurate reads new EffectStack<SpeedBoostConfig>,
                                      but bolt velocity is still old-multiplier value
                                      => VIOLATION fires
```

Ordering constraint from bolt/plugin.rs:
- bridge is `.after(BoltSystems::CellCollision)` (impact/system.rs line 267)
- checkers have `.before(BoltSystems::BoltLost)` but no `.before(EffectSystems::Bridge)`

## Two fix options

**Option A — Checkers run before the bridge**
Add `.before(EffectSystems::Bridge)` to the checker chain in
`breaker-scenario-runner/src/lifecycle/systems/plugin.rs`. The checkers then see the old
multiplier and the old velocity — consistent. Cost: very minor, checkers miss any
violations introduced by the bridge in the same frame (acceptable, they fire every frame).

**Option B — Collision systems re-apply formula after the bridge**
Add a dedicated `normalize_bolt_speeds` system in `BoltSystems` that runs
`.after(EffectSystems::Bridge)` and calls `apply_velocity_formula` on every active bolt.
This is the "ground-truth" fix: bolt speed is always consistent with `EffectStack<SpeedBoostConfig>`
by the time checkers run. Cost: one extra query pass per frame over all bolts.

Option A is simpler and sufficient for checker accuracy. Option B is the correct
behavioral fix if the game logic requires the new speed to take effect immediately
(e.g., a speed-boost chip fires on a wall hit and the bolt should immediately travel
faster that same frame).
