---
name: SecondWindWallAtMostOne violation — fire() spawns unconditionally
description: SecondWind fire() always spawns a wall without checking if one already exists, causing wall accumulation on rapid BoltLost cycles
type: project
---

## Bug: second_wind.rs — fire() has no existance check

`fire()` in `breaker-game/src/effect/effects/second_wind.rs` unconditionally spawns a `SecondWindWall` entity every time it is called. It does not check whether a wall already exists.

When `SecondWind` is configured as `When(trigger: BoltLost, ...)`, every bolt loss spawns another wall. If a bolt is served after a wall exists but falls to the bottom again before reaching the wall (e.g., stationary breaker, steep angles), a second wall spawns on top of the first. These accumulate over time.

Violated invariant: `SecondWindWallAtMostOne` (global — checked every frame regardless of scenario invariants list).

## Scenario

- `bolt_lost_second_wind.scenario.ron` — Aegis/Corridor, Chaos(action_prob=0.05), SecondWind on BoltLost
- 4428 violations, frames 372..4799 — persistent accumulation
- Not transient — violations fire for nearly the entire run after wall count exceeds 1

## Fix needed

`fire()` should check if a `SecondWindWall` already exists before spawning a new one, or should despawn existing walls before spawning. The single-use semantics require at most one wall active at a time.

File: `breaker-game/src/effect/effects/second_wind.rs`, function `fire()`

Confirmed 2026-03-30.
