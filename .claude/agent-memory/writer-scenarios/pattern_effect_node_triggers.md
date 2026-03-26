---
name: EffectNode trigger syntax and initial_overclocks conventions
description: RON trigger names, initial_overclocks field semantics, and how to exercise effect system in scenarios
type: reference
---

## initial_overclocks Field

`initial_overclocks: Option<Vec<EffectNode>>` — pushed directly into breaker `EffectChains` (NOT `ActiveEffects`).

**Critical**: bare `Do(...)` at top level of `EffectChains` never fires — `evaluate_node` only matches `When` nodes. Always wrap in a `When(trigger: ...)` gate.

**For passive effects** (Piercing, DamageBoost, SpeedBoost, SizeBoost) that should be active during gameplay, wrap in a trigger that fires frequently:
```ron
When(trigger: OnBump, then: [Do(Piercing(1)), Do(DamageBoost(0.5))])
```

## Trigger Serde Names (canonical)

These names go in RON `trigger:` fields. Source of truth: `breaker-game/src/effect/definition.rs`.

| RON name | Rust variant | Perspective |
|----------|-------------|-------------|
| `OnPerfectBump` | `PerfectBump` | Breaker |
| `OnBump` | `Bump` | Breaker |
| `OnEarlyBump` | `EarlyBump` | Breaker |
| `OnLateBump` | `LateBump` | Breaker |
| `OnBumpWhiff` | `BumpWhiff` | Breaker |
| `OnImpact(Cell/Breaker/Wall)` | `Impact(...)` | Bolt |
| `OnCellDestroyed` | `CellDestroyed` | Global |
| `OnBoltLost` | `BoltLost` | Global |
| `OnDeath` | `Death` | Global |
| `OnNodeTimerThreshold(f32)` | `NodeTimerThreshold(f32)` | Breaker |
| `PerfectBumped` | `PerfectBumped` | Bolt (bolt-perspective) |
| `Bumped` | `Bumped` | Bolt |
| `EarlyBumped` | `EarlyBumped` | Bolt |
| `LateBumped` | `LateBumped` | Bolt |
| `Impacted(Cell/Breaker/Wall)` | `Impacted(...)` | Bolt (bolt-perspective impact) |
| `DestroyedCell` | `DestroyedCell` | Bolt (bolt-perspective cell destroyed) |
| `TimeExpires(f32)` | `TimeExpires(f32)` | Until node only |
| `NoBump` | `NoBump` | Breaker |

## Effect Field Syntax

```ron
Do(SpeedBoost(multiplier: 1.3))               // struct field
Do(DamageBoost(0.5))                          // tuple
Do(Piercing(1))                               // tuple u32
Do(SizeBoost(0.2))                            // tuple f32
Do(Shockwave(base_range: 48.0, range_per_level: 0.0, stacks: 1, speed: 400.0))
Do(ChainBolt(tether_distance: 120.0))
Do(SpawnBolts(count: 1, lifespan: Some(3.0), inherit: true))
Do(SpawnBolts())                              // all defaults (count:1, lifespan:None, inherit:false)
Do(MultiBolt(base_count: 1, count_per_level: 0, stacks: 1))
Do(EntropyEngine(threshold: 3, pool: [(0.5, Do(...)), (0.5, Do(...))]))
Do(RandomEffect([(0.5, Do(...)), (0.5, Do(...))]))
```

## Working Trigger Chains for initial_overclocks

These patterns are verified working in scenario runner (bolt-perspective triggers work in EffectChains):

```ron
// Wall bounce speed boost (Impacted is bolt-perspective)
When(trigger: Impacted(Wall), then: [Do(SpeedBoost(multiplier: 1.1))])

// Perfect bump chain -> cell impact -> destroyed cell (4-deep)
When(trigger: PerfectBumped, then: [
    When(trigger: Impacted(Cell), then: [
        When(trigger: DestroyedCell, then: [Do(Shockwave(...))])
    ])
])

// Global cell destruction trigger
When(trigger: DestroyedCell, then: [Do(Shockwave(...))])

// Breaker-perspective bump (OnBump)
When(trigger: OnBump, then: [Do(RandomEffect(...))])
```

## Available Layouts

Names are case-sensitive strings matching `name:` in `.node.ron` files:
- `"Dense"` — 20-col dense grid, maximum cell density
- `"Corridor"` — tall narrow layout, maximum wall bounces
- `"Scatter"` — sparse cells at varied positions
- `"Fortress"` — fortress-style layout
- `"Gauntlet"` — gauntlet layout
- `"BossArena"` — boss arena layout

## max_bolt_count Guidelines

Default is 8 (InvariantParams default). When SpawnBolts or MultiBolt are involved:
- SpawnBolts(count: 1) with lifespan: use 12–16 (overlap window)
- MultiBolt(base_count: 1): use 12–20 (no lifespan, accumulates)
- Supernova-style (MultiBolt + chain): use 20+
