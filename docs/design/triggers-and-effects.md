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
}
```

### Node Types

| Node | Purpose | Re-fires? |
|------|---------|-----------|
| `When` | Trigger gate — fires children when trigger matches | Yes, each activation |
| `Do` | Leaf effect — terminal action | N/A |
| `Until` | Applies children, auto-removes when `until` trigger fires | No — removed on trigger |
| `Once` | Fires children once ever, then permanently consumed from the chain | No — consumed after first fire |

**`When`** — trigger gate. Fires children when the trigger condition is met. Re-fires on each subsequent activation. This is the primary mechanism for recurring triggered effects.

**`Do`** — leaf effect. Terminal action in the tree. Executes the effect immediately when reached during evaluation.

**`Until`** — applies children immediately, then auto-removes them when the `until` trigger fires. Used for timed buffs (`TimeExpires(3.0)`), trigger-based removal (`OnImpact(Breaker)`), or any effect that should end on a specific condition.

**`Once`** — fires children once ever, then permanently consumed from the chain. Used for SecondWind-style one-time saves where the effect fires exactly once per run (or per node) and never again.

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

## EffectChains Component

`EffectChains(Vec<EffectNode>)` replaces both the old `ActiveEffects` resource and `ArmedEffects` component. Any entity — bolts, cells, breakers — can have `EffectChains`. Bridge systems evaluate each entity's chains based on incoming triggers.

- **Breaker entity**: carries breaker-defined chains (from the breaker definition) and chip-granted chains
- **Bolt entity**: carries armed chains (partially-resolved trigger trees awaiting the next matching trigger)
- **Cell entity**: can carry per-cell effect chains if needed (future use)

When a `When` node matches a trigger but the inner chain requires further triggers, the remaining chain is pushed onto the appropriate entity's `EffectChains` for later evaluation.

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

Buffs stack **multiplicatively**. Each buff is independent. Removal divides out the buff's contribution.

All multipliers use the **1.x standard**: 2.0 = 2x (double), 0.5 = 50% (half). This applies to `SpeedBoost`, `DamageBoost`, `TimedSpeedBurst`, and any other multiplicative effect.

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
