# Triggers and Effects

Complete reference of the EffectNode system — the unified model for ALL chip effects and breaker behaviors.

All chip effects — whether passive (applied on selection), triggered (fired on game events), or breaker-defined — are expressed as `EffectNode` variants. There is no separate `ChipEffect`, `AmpEffect`, or `AugmentEffect` enum.

## EffectNode

```rust
pub enum EffectNode {
    When { trigger: Trigger, then: Vec<EffectNode> },
    Do(Effect),
    Until { until: Trigger, then: Vec<EffectNode> },
    Once(Vec<EffectNode>),
    On { target: Target, then: Vec<EffectNode> },  // mandatory at top level, also used for runtime chain transfer
}
```

### Node Types

| Node | Purpose | Re-fires? |
|------|---------|-----------|
| `When` | Trigger gate — fires children when trigger matches | Yes, each activation |
| `Do` | Leaf effect — terminal action | N/A |
| `Until` | Applies children, auto-removes when `until` trigger fires | No — removed on trigger |
| `Once` | Fires children once ever, then permanently consumed from the chain | No — consumed after first fire |
| `On` | Transfers children onto a target entity's `EffectChains` at evaluation time | N/A — modifies target entity |

**`When`** — trigger gate. Fires children when the trigger condition is met. Re-fires on each subsequent activation. This is the primary mechanism for recurring triggered effects.

**`Do`** — leaf effect. Terminal action in the tree. Executes the effect immediately when reached during evaluation.

**`Until`** — applies children immediately, then auto-removes them when the `until` trigger fires. Used for timed buffs (`TimeExpires(3.0)`), trigger-based removal (`Impact(Breaker)`), or any effect that should end on a specific condition.

**`Once`** — fires children once ever, then permanently consumed from the chain. Used for SecondWind-style one-time saves where the effect fires exactly once per run (or per node) and never again.

**`On`** — transfers children onto a target entity's `EffectChains` at runtime. The `target` is the `Target` enum (`Bolt`, `Breaker`, `Cell`) in RON. The bridge resolves it to a concrete entity from the trigger context (e.g., `On(Cell, ...)` resolves to the cell from `BoltHitCell { bolt, cell }`). Used for effects that dynamically modify other entities' chains, like PowderKeg adding an OnDeath shockwave to a hit cell.

### Examples

```ron
// Shockwave on perfect bump (recurring)
When(trigger: OnPerfectBump, then: [Do(Shockwave(base_range: 48.0, ...))])

// Temporary 2x damage boost for 3 seconds
Until(until: TimeExpires(3.0), then: [Do(DamageBoost(2.0))])

// One-time save: invisible wall that bounces bolt once
Once([Do(SecondWind(...))])

// Nested: perfect bump arms a cell-impact shockwave
When(trigger: OnPerfectBump, then: [
    When(trigger: OnImpact(Cell), then: [
        Do(Shockwave(base_range: 48.0, ...))
    ])
])

// Whiff redemption: after whiff, next cell impact gets 1.5x damage + shockwave
When(trigger: OnBumpWhiff, then: [
    When(trigger: OnImpact(Cell), then: [
        Until(until: OnImpact(Breaker), then: [
            Do(DamageBoost(1.5)),
            Do(Shockwave(base_range: 64.0, ...))
        ])
    ])
])

// PowderKeg: perfect bump → cell hit → add OnDeath shockwave to that cell
When(trigger: OnPerfectBump, then: [
    When(trigger: OnImpact(Cell), then: [
        On(target: Cell, then: [
            When(trigger: OnDeath, then: [Do(Shockwave(base_range: 48.0, ...))])
        ])
    ])
])
```

## Triggers

Triggers are conditions that gate child node evaluation. Used inside `When` and `Until` nodes.

| Trigger | Condition | Bolt Context |
|---------|-----------|-------------|
| `OnPerfectBump` | Bump timed within the perfect window | Specific bolt from bump |
| `OnEarlyBump` | Bump pressed before the perfect zone | Specific bolt from bump |
| `OnLateBump` | Bump pressed after the bolt hit | Specific bolt from bump |
| `OnBump` | Any non-whiff bump (Early, Late, or Perfect) | Specific bolt from bump |
| `OnBumpWhiff` | Forward bump window expired without contact | Global (no specific bolt) |
| `OnImpact(Cell)` | Bolt hit a cell | Specific bolt from impact |
| `OnImpact(Breaker)` | Bolt bounced off the breaker | Specific bolt from impact |
| `OnImpact(Wall)` | Bolt bounced off a wall | Specific bolt from impact |
| `OnCellDestroyed` | A cell was destroyed | Global (no specific bolt) |
| `OnBoltLost` | A bolt was lost (fell off screen) | Global (no specific bolt) |
| `OnDeath` | Breaker lost all lives or timer expired | Global |
| `OnSelected` | Chip was selected on the upgrade screen | N/A — immediate evaluation |
| `TimeExpires(f32)` | Elapsed seconds since the node was applied | N/A — timer-based removal |

### OnSelected — Passive Effects

`OnSelected` is a special trigger used in `When` nodes that evaluates immediately when a chip is selected, rather than waiting for a game event:

```ron
// Passive piercing
When(trigger: OnSelected, then: [Do(Piercing(1))])

// Passive breaker width boost
When(trigger: OnSelected, then: [Do(SizeBoost(Breaker, 8.0))])

// Multiple passive effects in one selection
When(trigger: OnSelected, then: [
    Do(SizeBoost(Breaker, 6.0)),
    Do(BumpForce(8.0))
])
```

### Trigger Chaining

`When` nodes can nest arbitrarily deep. Each nesting layer adds one arm-then-resolve step before the leaf effect fires. Examples:
- `When(OnPerfectBump, [Do(Shockwave(...))])` — depth 1, fires shockwave on perfect bump
- `When(OnPerfectBump, [When(OnImpact(Cell), [Do(Shockwave(...))])])` — depth 2, fires shockwave on cell impact after a perfect bump
- `When(OnPerfectBump, [When(OnImpact(Cell), [When(OnCellDestroyed, [Do(Shockwave(...))])])])` — depth 3, fires shockwave when the hit cell is destroyed after a perfect-bump cell impact

The evaluate/arm/resolve cycle is depth-agnostic: `evaluate()` peels the outermost trigger layer, `arm_bolt()` pushes the remaining chain onto the bolt's `EffectChains`, and `resolve_armed()` re-evaluates on subsequent triggers — producing either another `Arm` (re-arm with a shorter chain) or `Fire` (execute the leaf). A 5-deep chain would arm 4 times then fire.

### Bolt Context

- **Specific bolt**: The effect targets the bolt that triggered the event. The bolt entity is passed through `EffectFired.bolt`.
- **Global**: No specific bolt. Effects that require a bolt entity (like `SpeedBoost` targeting `Bolt`) will no-op. Effects that don't require a bolt (like `LoseLife`) fire normally.

## Chain Ownership Model

Chains live on the entity whose events trigger them. The `Target` enum in effects handles cross-entity targeting at handler time. `On` enables runtime chain transfer to other entities.

### Two Stores Per Entity

- **`EffectChains`** — permanent source of truth. Populated by breaker init, chip dispatch, and `On` node evaluation. Never modified by trigger evaluation.
- **`ArmedEffects`** — temporary working set. Partially resolved `When` trees waiting for deeper triggers. Consumed on Fire, replaced on re-Arm.

### Which Entity Owns Which Chains

| Trigger | Default Owner | Why |
|---------|--------------|-----|
| PerfectBump, Bump, EarlyBump, LateBump, BumpWhiff, NoBump | Breaker | Bump is a breaker event |
| Impact(Cell), Impact(Wall), Impact(Breaker) | Bolt | Bolt is the impactor |
| Death | The dying entity | Cell death chains on cell, bolt death on bolt |
| NodeTimerThreshold | Breaker | Node-level timer |
| Selected | N/A | Fires immediately |
| TimeExpires | N/A | Managed by Until timer system |

**Global triggers** — `CellDestroyed` and `BoltLost` are global events. During evaluation, bridges sweep ALL entities with `EffectChains` and evaluate matching chains wherever they live. The `CellDestroyed` bridge reads `RequestCellDestroyed` (entity still alive) so effects can access the cell's components. It also evaluates the cell's own `Death` trigger. Then writes `CellDestroyedAt` as aftermath for location-only consumers. Chip authors control which entity the chain lives on using `On`:

```ron
// CellDestroyed chain on bolt (Cascade — shockwave at bolt position)
On(target: Bolt, then: [When(trigger: CellDestroyed, then: [Do(Shockwave(...))])])

// CellDestroyed chain on breaker (scoring/global reaction)
On(target: Breaker, then: [When(trigger: CellDestroyed, then: [Do(SpawnBolts(...))])])
```

Effects are pure data — they describe WHAT to do, not WHO to do it to. `Target` enum (`Bolt`, `Breaker`, `Cell`, `Wall`, `AllBolts`, `AllCells`) lives only on `On`, not on effects.

`On` is **mandatory** at the top level of every chip and breaker definition. Enforced at compile time via `RootEffect` — a single-variant enum wrapper that deserializes identically to `EffectNode::On` in RON:

```rust
pub enum RootEffect {
    On { target: Target, then: Vec<EffectNode> },
}

// ChipDefinition.effects: Vec<RootEffect>
// BreakerDefinition.effects: Vec<RootEffect>
```

Bare `When(...)` or `Do(...)` at the top level is a compile error.

Example — Aegis breaker effects:
```ron
effects: [
    On(Breaker, [When(trigger: BoltLost, then: [Do(LoseLife)])]),
    On(Bolt, [When(trigger: PerfectBumped, then: [Do(SpeedBoost(1.5))])]),
    On(Bolt, [When(trigger: Bumped, then: [Do(SpeedBoost(1.1))])]),
]
```

### Bump / Bumped Trigger Split

Bump triggers are split by perspective to clarify entity ownership:

| Trigger | Perspective | Owner entity |
|---------|-----------|-------------|
| `PerfectBump` | "A perfect bump happened" | Breaker |
| `PerfectBumped` | "I was perfect bumped" | Bolt |
| `Bump` / `Bumped` | Any non-whiff bump | Breaker / Bolt |
| `EarlyBump` / `EarlyBumped` | Early bump | Breaker / Bolt |
| `LateBump` / `LateBumped` | Late bump | Breaker / Bolt |
| `BumpWhiff` | "I whiffed" | Breaker only |
| `NoBump` | "Bolt hit me without bump" | Breaker only |

This eliminates cross-entity arm routing for the common bump→bolt-effect case.

### Chip Dispatch

When a chip is selected, each chain is routed to an entity:
- **`On(target, children)` at top level**: push children to the specified target entity. This gives chip authors explicit control over chain ownership.
- **`When(trigger, ...)` at top level**: push to the entity that owns the trigger (see table above). `When(PerfectBump, ...)` → breaker. `When(Impact(Cell), ...)` → all bolts.
- **`When(Selected, ...)`**: fires immediately, not stored.

### Evaluation Routing

Different triggers evaluate different entity sets:
- **Entity-specific triggers** (Impact, Bump, Death): evaluate only the relevant entity's EffectChains + ArmedEffects.
- **Global triggers** (CellDestroyed, BoltLost): sweep ALL entities with EffectChains and evaluate matching chains wherever they live.

### Arm Routing

When evaluation produces an Arm result and the inner trigger belongs to a different entity type, the armed chain moves to that entity's `ArmedEffects`. Example: `When(PerfectBump, [When(Impact(Wall), [Do(Shockwave)])])` — first Arm stays on breaker, second Arm moves to bolt (Impact is a bolt event).

### Runtime Chain Transfer (`On`)

`On(target, children)` pushes children onto a target entity's `EffectChains` at evaluation time. The target entity is resolved from the trigger context. This is how effects like PowderKeg dynamically add chains to other entities:

```ron
// PowderKeg: perfect bump → cell hit → add OnDeath shockwave to THAT cell
When(trigger: PerfectBump, then: [
    When(trigger: Impact(Cell), then: [
        On(target: Cell, then: [
            When(trigger: Death, then: [Do(Shockwave(...))])
        ])
    ])
])
```

## Effects (Leaves)

Leaf effects are the terminal `Do(...)` actions in an EffectNode tree. They fire via the `EffectFired` event and are handled by dedicated observer systems.

### Triggered Effects

These fire through the bridge system when their trigger condition is met.

| Effect | Parameters | Handler | Description |
|--------|-----------|---------|-------------|
| `Shockwave` | `base_range`, `range_per_level`, `stacks` | `handle_shockwave` | Area damage within range. Effective range = `base_range + (stacks - 1) * range_per_level`. |
| `ChainBolt` | `tether_distance` | `handle_chain_bolt` | Spawns a chain bolt tethered to the triggering bolt via `DistanceConstraint`. |
| `SpawnBolts` | `count`, `lifespan`, `inherit` | `handle_spawn_bolts` | Spawns `count` additional bolts. Serde defaults: `lifespan: None` (permanent), `inherit: false` (no effect inheritance), `count: 1`. |
| `MultiBolt` | `base_count`, `count_per_level`, `stacks` | *(not yet wired)* | Spawns additional bolts. Effective count = `base_count + (stacks - 1) * count_per_level`. |
| `Shield` | `base_duration`, `duration_per_level`, `stacks` | *(not yet wired)* | Temporary shield. Effective duration = `base_duration + (stacks - 1) * duration_per_level`. |
| `LoseLife` | *(none)* | `handle_life_lost` | Decrements `LivesCount`. When lives reach 0, sends `RunLost`. |
| `TimePenalty` | `seconds` | `handle_time_penalty` | Subtracts time from the node timer. |
| `SpeedBoost` | `target: Target`, `multiplier: f32` | `handle_speed_boost` | Scales velocity of the target by `multiplier`. Uses 1.x format: 2.0 = 2x speed, 0.5 = 50% speed. |
| `RandomEffect` | `Vec<(f32, EffectNode)>` | `handle_random_effect` | Weighted random selection from a pool of effects. |
| `EntropyEngine` | `counter: u32`, `Vec<(f32, EffectNode)>` | `handle_entropy_engine` | Counter-gated `RandomEffect` — every Nth trigger, roll from pool. |
| `RampingDamage` | `bonus_per_hit: f32`, `max_bonus: f32` | `handle_ramping_damage` | Stacking damage bonus on cell hits, resets on non-bump breaker impact. |
| `TimedSpeedBurst` | `speed_mult: f32`, `duration_secs: f32` | `handle_timed_speed_burst` | Temporary speed multiplier that decays after duration. |
| `SecondWind` | `invuln_secs: f32` | `handle_second_wind` | Invisible wall that bounces the bolt once per node. Applied to the breaker's `EffectChains`. Consumed after use via `Once`. |
| `ChainLightning` | `arcs: u32`, `range: f32`, `damage_mult: f32` | `handle_chain_lightning` | Arc damage jumping between nearby cells. |
| `PiercingBeam` | `damage_mult: f32`, `width: f32` | `handle_piercing_beam` | Piercing beam through all cells in the bolt's path. |

### Passive Effects (OnSelected Leaves)

These fire immediately when a chip is selected and modify entity components directly.

| Effect | Parameters | Target | Description |
|--------|-----------|--------|-------------|
| `Piercing` | `count: u32` | Bolt | Bolt passes through N cells before stopping |
| `DamageBoost` | `boost: f32` | Bolt | Fractional bonus damage per stack. Uses 1.x format: 2.0 = 2x damage. |
| `SpeedBoost` | `target: Target`, `multiplier: f32` | Bolt or Breaker | Multiplicative speed multiplier per stack (e.g., 1.1 = 10% boost). 2.0 = 2x speed. |
| `ChainHit` | `count: u32` | Bolt | Chains to N additional cells on hit |
| `SizeBoost` | `target: Target`, `value: f32` | Bolt (radius) or Breaker (width) | Size increase per stack |
| `Attraction` | `type: AttractionType`, `force: f32` | Bolt | Attracts toward nearest entity of the given type. See Attraction section below. |
| `BumpForce` | `force: f32` | Breaker | Flat bump force increase per stack |
| `TiltControl` | `sensitivity: f32` | Breaker | Flat tilt control sensitivity increase per stack |
| `TimePressureBoost` | `speed_mult: f32`, `threshold_pct: f32` | Bolt | Conditional: when timer < threshold, bolt speed multiplied. |

### Attraction

Redesigned in C7. `Attraction(AttractionType, f32)` where `AttractionType` is one of `Cell`, `Wall`, or `Breaker`.

- Bolt is attracted toward the **nearest** entity of the specified type
- **Nearest wins**: if multiple attraction types are active, the closest target of any type determines the pull direction
- **Type deactivation**: attraction toward a type deactivates on hit with that type, reactivates on bounce off a non-attracted type
- Force value is the attraction strength (higher = stronger pull)

```ron
// Attract toward nearest cell
Do(Attraction(Cell, 1.0))

// Attract toward nearest wall
Do(Attraction(Wall, 0.5))
```

### Target Enum

Effects that can target multiple entity types use the `Target` enum:

| Target | Behavior |
|--------|----------|
| `Bolt` | Affects the specific triggering bolt entity |
| `Breaker` | Affects the breaker entity |
| `AllBolts` | Affects all bolt entities in play |

`SizeBoost` interpretation varies by target: on `Bolt` it adjusts radius, on `Breaker` it adjusts width.

### Buff Stacking

Passive effects that modify stats (SpeedBoost, DamageBoost, Piercing, SizeBoost, BumpForce) are tracked in per-entity vecs. Each application pushes an entry; each removal removes one entry. The actual stat is **recalculated from the vec** every tick — no incremental mutation, no imprecision from clamping.

| Effect | Stacking | Recalculation |
|--------|----------|---------------|
| `SpeedBoost` | Multiplicative | `base_speed * product(boosts)`, clamped to `[min, max]` |
| `DamageBoost` | Multiplicative | `base_damage * product(boosts)` |
| `Piercing` | Additive | `sum(pierce_counts)` |
| `SizeBoost` | Additive | `base_size + sum(boosts)` |
| `BumpForce` | Additive | `base_force + sum(boosts)` |

**Until removal**: When an `Until` node expires, it fires a removal message (e.g., `RemoveSpeedBoost`) for each passive child it applied. The effect's own cleanup system removes the matching entry from the vec. Recalculation picks up the change automatically next tick. Until has zero knowledge of effect internals.

**Non-passive children in Until**: `When`/`Once` nodes nested inside `Until` are armed triggers — they live inside the Until container and are evaluated by bridges while the Until is alive. When the Until expires, the container is removed and the armed triggers are gone. No removal message needed.

All multipliers use the **1.x standard**: 2.0 = 2x (double), 0.5 = 50% (half). This applies to `SpeedBoost`, `DamageBoost`, and any other multiplicative effect.

### Serde Defaults

`EffectNode` fields use serde defaults to minimize RON verbosity:
- `Option<T>` defaults to `None`
- `bool` defaults to `false`
- `u32` defaults to `1`

RON files only need to specify non-default values.

## Breaker Usage

Each breaker defines root effect chains for bump events:

| Breaker | on_bolt_lost | on_perfect_bump | on_early_bump | on_late_bump |
|---------|-------------|-----------------|---------------|-------------|
| **Aegis** | `Do(LoseLife)` | `Do(SpeedBoost(Bolt, 1.5))` | `Do(SpeedBoost(Bolt, 1.1))` | `Do(SpeedBoost(Bolt, 1.1))` |
| **Chrono** | `Do(TimePenalty(5.0))` | `Do(SpeedBoost(Bolt, 1.5))` | `Do(SpeedBoost(Bolt, 1.1))` | `Do(SpeedBoost(Bolt, 1.1))` |
| **Prism** | `Do(TimePenalty(7.0))` | `Do(SpawnBolts { count: 1 })` | *(none)* | *(none)* |

Breakers can also define additional chains in the `chains` field for more complex trigger combinations. These chains are loaded into the breaker entity's `EffectChains` at run start.
