---
name: Effect RON syntax patterns
description: Exact RON syntax for initial_effects, EffectKind variants, and trigger chains used in scenario files
type: reference
---

## RootEffect wrapper

Every entry in `initial_effects` must be wrapped in `On(target: ..., then: [...])`.

```ron
initial_effects: [
    On(target: Bolt, then: [
        When(trigger: PerfectBumped, then: [
            Do(SpeedBoost(multiplier: 1.3)),
        ]),
    ]),
],
```

## Permanent (no trigger) effects

To apply an effect unconditionally for the whole run, omit the `When` and put `Do(...)` directly inside the `On`:

```ron
initial_effects: [
    On(target: Bolt, then: [
        Do(Attraction(attraction_type: Cell, force: 600.0, max_force: Some(400.0))),
    ]),
],
```

## Duration-scoped effects (Until)

```ron
Until(trigger: TimeExpires(0.5), then: [
    Do(Pulse(base_range: 48.0, range_per_level: 0.0, stacks: 1, speed: 300.0, interval: 0.2)),
]),
```

## Key EffectKind field notes

- `SpawnBolts`: uses `count` (not `base_count`), plus optional `lifespan` and `inherit`
- `Pulse`: `interval` has serde default of 0.5 — can be omitted
- `Attraction`: `max_force` has serde default of `None` — use `Some(N)` to cap
- `Shield`: targets `Breaker`, not `Bolt`
- `SecondWind`: no fields — just `Do(SecondWind)`
- `GravityWell`: `strength`, `duration`, `radius`, `max`
- `Explode`: `range`, `damage_mult`
- `SpawnPhantom`: `duration`, `max_active`

## Target values

`Bolt`, `AllBolts`, `Breaker`, `Cell`, `AllCells`, `Wall`, `AllWalls`

## Trigger values (common ones)

`NodeStart`, `NodeEnd`, `BoltLost`, `CellDestroyed`, `PerfectBumped`, `Bumped`,
`Impacted(Cell)`, `Impacted(Wall)`, `Impacted(Breaker)`, `TimeExpires(N)`
