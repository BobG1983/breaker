# Triggers and Effects

Complete reference for the effect system — the unified model for ALL chip effects and breaker behaviors.

All chip effects — whether passive (applied on selection), triggered (fired on game events), or breaker-defined — are expressed as `EffectNode` trees. There is no separate `ChipEffect`, `AmpEffect`, or `AugmentEffect` enum. See [architecture/effects.md](../architecture/effects.md) for the full runtime model.

## EffectNode

```rust
pub enum EffectNode {
    When { trigger: Trigger, then: Vec<EffectNode> },
    Do(Effect),
    Until { until: Trigger, then: Vec<EffectNode> },
    Once(Vec<EffectNode>),
    On { target: Target, then: Vec<EffectNode> },
}
```

### Node Types

| Node | Purpose | Re-fires? |
|------|---------|-----------|
| `When` | Trigger gate — fires children when trigger matches | Yes (in EffectChains), No (in ArmedEffects — consumed) |
| `Do` | Leaf effect — fires on the entity it lives on | N/A |
| `Until` | Applies children immediately, reverses when until-trigger fires | Runtime node in ArmedEffects |
| `Once` | Fires children once, then permanently consumed | No — consumed after first fire |
| `On` | Redirects children to another entity — never fires on self | N/A |

**`When`** — trigger gate. Fires children when the trigger matches. In EffectChains: permanent, re-evaluates every time. In ArmedEffects: consumed after matching (one-shot).

**`Do`** — terminal effect. Fires the effect directly on the entity whose chain is being evaluated. The entity is the implicit target — no targets field.

**`Until`** — duration-scoped buff. When first encountered: fires/arms children immediately. Stays in ArmedEffects as a runtime node. When the until-trigger fires: reverses all children and removes itself. Used for timed buffs (`TimeExpires(3.0)`) and trigger-based removal (`Impacted(Breaker)`).

**`Once`** — one-shot wrapper. Evaluate children against the current trigger. If any match, fire/arm them and consume the Once. If nothing matches, keep it.

**`On`** — entity redirect. Resolves target to entity/entities from trigger context. Bare `Do` children fire directly on the target. Non-Do children push to the target's ArmedEffects. On never fires anything on the current entity.

### Examples

```ron
// Shockwave on wall impact (bolt's chain)
When(trigger: Impacted(Wall), then: [Do(Shockwave(base_range: 48.0, ...))])

// Temporary 2x damage boost for 3 seconds
Until(until: TimeExpires(3.0), then: [Do(DamageBoost(2.0))])

// One-time save: bounces bolt once
Once([When(trigger: BoltLost, then: [Do(SecondWind(invuln_secs: 1.0))])])

// Nested: perfect bump arms a cell-impact shockwave
When(trigger: PerfectBumped, then: [
    When(trigger: Impacted(Cell), then: [
        Do(Shockwave(base_range: 48.0, ...))
    ])
])

// Ricochet Protocol: wall hit arms damage boost until next cell hit
When(trigger: Impacted(Wall), then: [
    Until(until: Impacted(Cell), then: [
        Do(DamageBoost(2.0))
    ])
])

// Wall redirects speed boost to the bolt that hit it
When(trigger: Impacted(Bolt), then: [
    On(target: Bolt, then: [Do(SpeedBoost(multiplier: 1.2))])
])
```

## Triggers

Triggers are conditions that gate child node evaluation. Each trigger has a **scope** (global or targeted) and optional **context entities** that On nodes can resolve.

### Global Triggers

Sweep ALL entities with EffectChains when they fire.

| Trigger | RON name | Condition | Context |
|---------|----------|-----------|---------|
| `PerfectBump` | `PerfectBump` | Perfect-timed bump | bolt |
| `EarlyBump` | `EarlyBump` | Early bump | bolt |
| `LateBump` | `LateBump` | Late bump | bolt |
| `Bump` | `Bump` | Any non-whiff bump | bolt |
| `BumpWhiff` | `BumpWhiff` | Forward bump window expired without contact | (none) |
| `NoBump` | `NoBump` | Bolt passed breaker without bump attempt | bolt |
| `Death` | `Death` | Something died | dying entity |
| `BoltLost` | `BoltLost` | Bolt fell off screen | (none) |
| `CellDestroyed` | `CellDestroyed` | A cell was destroyed | cell position |
| `NodeTimerThreshold(f32)` | `NodeTimerThreshold(0.25)` | Node timer ratio drops below threshold | (none) |

### Targeted Triggers

Evaluate ONLY the specific entity involved.

| Trigger | RON name | Evaluated on | Context |
|---------|----------|-------------|---------|
| `PerfectBumped` | `PerfectBumped` | The bolt | bolt |
| `EarlyBumped` | `EarlyBumped` | The bolt | bolt |
| `LateBumped` | `LateBumped` | The bolt | bolt |
| `Bumped` | `Bumped` | The bolt | bolt |
| `Impacted(Cell)` | `Impacted(Cell)` | The entity that hit a cell | both entities |
| `Impacted(Bolt)` | `Impacted(Bolt)` | The entity hit by a bolt | both entities |
| `Impacted(Wall)` | `Impacted(Wall)` | The entity that hit a wall | both entities |
| `Impacted(Breaker)` | `Impacted(Breaker)` | The entity that hit a breaker | both entities |
| `Died` | `Died` | The dying entity | dying entity |

### Special Triggers

| Trigger | Behavior |
|---------|----------|
| `Selected` | Fires at dispatch time (chip selection). Used for passives: `When(Selected, [Do(Piercing(1))])` |
| `TimeExpires(f32)` | Timer system ticks Until entries in ArmedEffects. Not used in When nodes. |

### Impacted Fires Both Directions

When a bolt hits a cell, the bridge fires TWO triggers:
- `Impacted(Cell)` on the bolt — "I hit a cell"
- `Impacted(Bolt)` on the cell — "I was hit by a bolt"

Both entities receive context about the other. Any entity type can be on either side as mechanics expand (moving cells, breaker collisions, etc.).

### Bump / Bumped Split

Bump triggers split by perspective to clarify entity ownership:

| Trigger | Scope | Perspective |
|---------|-------|------------|
| `PerfectBump` | Global | "A perfect bump happened" |
| `PerfectBumped` | Targeted (bolt) | "I was perfect bumped" |
| `BumpWhiff` | Global | "I whiffed" (breaker perspective) |
| `NoBump` | Global | "Bolt hit me without bump input" |

### Death / Died Split

| Trigger | Scope | Perspective |
|---------|-------|------------|
| `Death` | Global | "Something died" — sweep all entities |
| `Died` | Targeted | "I died" — on the dying entity |

## Effects (Leaves)

Leaf effects are the terminal `Do(...)` actions. Each effect is a direct function call on the entity — no typed events, no observer indirection. The entity is the implicit target.

### Triggered Effects

| Effect | Parameters | Description |
|--------|-----------|-------------|
| `Shockwave` | `base_range`, `range_per_level`, `stacks`, `speed` | Expanding ring of area damage at entity position |
| `ChainBolt` | `tether_distance` | Spawns a chain bolt tethered to the entity |
| `SpawnBolts` | `count`, `lifespan`, `inherit` | Spawns additional bolts. `inherit: true` copies parent's EffectChains. |
| `MultiBolt` | `base_count`, `count_per_level`, `stacks` | Spawns bolts. Count = `base_count + (stacks - 1) * count_per_level` |
| `Shield` | `base_duration`, `duration_per_level`, `stacks` | Temporary shield. Duration = `base + (stacks - 1) * per_level` |
| `LoseLife` | *(none)* | Decrements LivesCount. Sends RunLost when 0 |
| `TimePenalty` | `seconds` | Subtracts time from node timer |
| `SpeedBoost` | `multiplier` | Scales entity's speed by multiplier |
| `RandomEffect` | `Vec<(f32, EffectNode)>` | Weighted random selection from pool |
| `EntropyEngine` | `threshold`, `Vec<(f32, EffectNode)>` | Counter-gated random — every Nth trigger, roll from pool |
| `RampingDamage` | `bonus_per_hit` | Stacking damage bonus on cell hits. No max cap. |
| `SecondWind` | `invuln_secs` | Spawns invisible bottom wall, bounces bolt once |
| `ChainLightning` | `arcs`, `range`, `damage_mult` | Arc damage jumping between nearby cells |
| `PiercingBeam` | `damage_mult`, `width` | Beam through all cells in velocity direction |
| `SpawnPhantom` | `duration`, `max_active` | Temporary phantom bolt with infinite piercing |
| `GravityWell` | `strength`, `duration`, `radius`, `max` | Attracts bolts within radius |
| `Pulse` | `base_range`, `range_per_level`, `stacks`, `speed` | Shockwave at every active bolt position simultaneously |

### Passive Effects (Selected Trigger)

Applied immediately when a chip is selected via `When(Selected, [Do(...)])`.

| Effect | Parameters | Description |
|--------|-----------|-------------|
| `Piercing` | `count: u32` | Bolt passes through N cells before stopping |
| `DamageBoost` | `boost: f32` | Multiplicative damage bonus (1.x format) |
| `SpeedBoost` | `multiplier: f32` | Multiplicative speed bonus (1.x format) |
| `ChainHit` | `count: u32` | Chains to N additional cells on hit |
| `SizeBoost` | `value: f32` | Size increase (radius for bolts, width for breakers) |
| `Attraction` | `type`, `force` | Attracts toward nearest entity of given type |
| `BumpForce` | `force: f32` | Flat bump force increase |
| `TiltControl` | `sensitivity: f32` | Flat tilt control sensitivity increase |

### Attraction

`Attraction(AttractionType, f32)` where `AttractionType` is `Cell`, `Wall`, or `Breaker`.

- Bolt attracts toward the **nearest** entity of the specified type
- **Nearest wins**: if multiple types active, closest target determines pull direction
- **Type deactivation**: attraction toward a type deactivates on hit, reactivates on bounce off a non-attracted type

### Buff Stacking

Passive effects that modify stats are tracked in per-entity vecs. Each application pushes an entry; reversal removes one. The actual stat is **recalculated from the vec** every tick.

| Effect | Stacking | Recalculation |
|--------|----------|---------------|
| `SpeedBoost` | Multiplicative | `base_speed * product(boosts)`, clamped `[min, max]` |
| `DamageBoost` | Multiplicative | `base_damage * product(boosts)` |
| `Piercing` | Additive | `sum(pierce_counts)` |
| `SizeBoost` | Additive | `base_size + sum(boosts)` |
| `BumpForce` | Additive | `base_force + sum(boosts)` |

All multipliers use the **1.x standard**: 2.0 = 2x (double), 0.5 = 50% (half).

### Reversal

Each reversible effect defines its own reverse function. The `Effect` enum has a `reverse()` method that dispatches to the effect's reverse function — one line per variant. Non-reversible effects (Shockwave, SpawnBolts, LoseLife, etc.) are no-ops.

Until nodes store their children and call `reverse()` on each when the until-trigger fires. The Until system never imports effect internals — reversal logic lives entirely in each effect's module.

## Target Enum

`Target` is used in `On { target, then }` nodes to scope where chains are dispatched:

| Target | At dispatch | At runtime |
|--------|------------|-----------|
| `Bolt` | Primary bolt | Bolt from trigger context |
| `AllBolts` | All bolt entities | All bolt entities |
| `Breaker` | Breaker entity | Breaker entity |
| `Cell` | No-op | Cell from trigger context |
| `AllCells` | No-op | All cell entities |
| `Wall` | No-op | Wall from trigger context |

## RootEffect and Breaker Definition

`RootEffect` constrains top-level effect declarations so every chain explicitly names its target:

```rust
pub enum RootEffect {
    On { target: Target, then: Vec<EffectNode> }
}
```

Both `BreakerDefinition` and `ChipDefinition` use `effects: Vec<RootEffect>`.

### Breaker Examples

```ron
// Aegis — life-based, speed boosts on bump
(
    name: "Aegis",
    stat_overrides: (),
    life_pool: Some(3),
    effects: [
        On(target: Breaker, then: [When(trigger: BoltLost, then: [Do(LoseLife)])]),
        On(target: Bolt, then: [When(trigger: PerfectBumped, then: [Do(SpeedBoost(multiplier: 1.5))])]),
        On(target: Bolt, then: [When(trigger: EarlyBumped, then: [Do(SpeedBoost(multiplier: 1.1))])]),
        On(target: Bolt, then: [When(trigger: LateBumped, then: [Do(SpeedBoost(multiplier: 1.1))])]),
    ],
)

// Chrono — timer-based, penalty on bolt lost
(
    name: "Chrono",
    stat_overrides: (),
    life_pool: None,
    effects: [
        On(target: Breaker, then: [When(trigger: BoltLost, then: [Do(TimePenalty(seconds: 5.0))])]),
        On(target: Bolt, then: [When(trigger: PerfectBumped, then: [Do(SpeedBoost(multiplier: 1.5))])]),
        On(target: Bolt, then: [When(trigger: EarlyBumped, then: [Do(SpeedBoost(multiplier: 1.1))])]),
        On(target: Bolt, then: [When(trigger: LateBumped, then: [Do(SpeedBoost(multiplier: 1.1))])]),
    ],
)

// Prism — spawns bolts on perfect bump
(
    name: "Prism",
    stat_overrides: (),
    life_pool: None,
    effects: [
        On(target: Breaker, then: [When(trigger: BoltLost, then: [Do(TimePenalty(seconds: 7.0))])]),
        On(target: Breaker, then: [When(trigger: PerfectBump, then: [Do(SpawnBolts())])]),
    ],
)
```

### Chip Examples

```ron
// Surge — permanent speed boost on perfect bump
(
    name: "Surge",
    max_taken: 3,
    common: Some((
        prefix: "Basic",
        effects: [
            On(target: Bolt, then: [
                When(trigger: PerfectBumped, then: [Do(SpeedBoost(multiplier: 1.2))])
            ]),
        ],
    )),
    ...
)

// Overclock — temporary speed boost
(
    name: "Overclock",
    max_taken: 2,
    common: Some((
        prefix: "Basic",
        effects: [
            On(target: Bolt, then: [
                When(trigger: PerfectBumped, then: [
                    Until(until: TimeExpires(2.0), then: [Do(SpeedBoost(multiplier: 1.3))])
                ])
            ]),
        ],
    )),
    ...
)

// Piercing Shot — passive
(
    name: "Piercing Shot",
    max_taken: 3,
    common: Some((
        prefix: "Basic",
        effects: [
            On(target: Bolt, then: [When(trigger: Selected, then: [Do(Piercing(1))])])
        ],
    )),
    ...
)

// Last Stand — breaker speed boost on bolt lost
(
    name: "Last Stand",
    max_taken: 1,
    common: Some((
        prefix: "Minor",
        effects: [
            On(target: Breaker, then: [
                When(trigger: BoltLost, then: [Do(SpeedBoost(multiplier: 1.15))])
            ]),
        ],
    )),
    ...
)

// Ricochet Protocol — wall hit arms damage boost until cell hit
(
    name: "Ricochet Protocol",
    max_taken: 1,
    legendary: Some((
        prefix: "",
        effects: [
            On(target: Bolt, then: [
                When(trigger: Impacted(Wall), then: [
                    Until(until: Impacted(Cell), then: [Do(DamageBoost(2.0))])
                ])
            ]),
        ],
    )),
)
```

### Serde Defaults

`EffectNode` fields use serde defaults to minimize RON verbosity:
- `Option<T>` defaults to `None`
- `bool` defaults to `false`
- `u32` defaults to `1`
