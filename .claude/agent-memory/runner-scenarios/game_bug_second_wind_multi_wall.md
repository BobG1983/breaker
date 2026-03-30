---
name: SecondWindWallAtMostOne violation — fire() spawns unconditionally (RESOLVED)
description: SecondWind fire() always spawned a wall without checking if one already exists — previously confirmed bug, now RESOLVED
type: project
---

## Bug: second_wind.rs — fire() has no existence check (RESOLVED)

`fire()` in `breaker-game/src/effect/effects/second_wind.rs` unconditionally spawned a `SecondWindWall` entity every time it was called. It did not check whether a wall already existed.

When `SecondWind` was configured as `When(trigger: BoltLost, ...)`, every bolt loss spawned another wall. If a bolt was served after a wall existed but fell to the bottom again before reaching the wall (e.g., stationary breaker, steep angles), a second wall spawned on top of the first. These accumulated over time.

Violated invariant: `SecondWindWallAtMostOne` (global — checked every frame regardless of scenario invariants list).

## Previous failure

- `bolt_lost_second_wind.scenario.ron` — Aegis/Corridor, Chaos(action_prob=0.05), SecondWind on BoltLost
- Previously: 4428 violations, frames 372..4799 — persistent accumulation

## Fix needed (archived)

`fire()` should check if a `SecondWindWall` already exists before spawning a new one, or should despawn existing walls before spawning. The single-use semantics require at most one wall active at a time.

File: `breaker-game/src/effect/effects/second_wind.rs`, function `fire()`

## Status

RESOLVED — `bolt_lost_second_wind` PASS, `second_wind_wall_at_most_one` PASS. Confirmed 2026-03-30 run.
