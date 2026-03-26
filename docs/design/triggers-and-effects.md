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
    On { target: Target, then: Vec<EffectNode> },  // target scope — used by RootEffect
}
```

### Node Types

| Node | Purpose | Re-fires? |
|------|---------|-----------|
| `When` | Trigger gate — fires children when trigger matches | Yes, each activation |
| `Do` | Leaf effect — terminal action | N/A |
| `Until` | Applies children, auto-removes when `until` trigger fires | No — removed on trigger |
| `Once` | Fires children once ever, then permanently consumed from the chain | No — consumed after first fire |
| `On` | Target scope — dispatches children against the specified entity type; used at top level in `RootEffect` | N/A |

**`When`** — trigger gate. Fires children when the trigger condition is met. Re-fires on each subsequent activation. This is the primary mechanism for recurring triggered effects.

**`Do`** — leaf effect. Terminal action in the tree. Executes the effect immediately when reached during evaluation.

**`Until`** — applies children immediately, then auto-removes them when the `until` trigger fires. Used for timed buffs (`TimeExpires(3.0)`), trigger-based removal (`Impact(Breaker)`), or any effect that should end on a specific condition.

**`Once`** — fires children once ever, then permanently consumed from the chain. Used for SecondWind-style one-time saves where the effect fires exactly once per run (or per node) and never again.

**`On`** — target scope. Dispatches children against the entity identified by `target`. Not evaluated by trigger matching — it is resolved at dispatch time. Used as the top-level wrapper in `RootEffect` so every breaker chain explicitly names its target entity. `On` nodes in a chain are unwrapped during dispatch before any trigger evaluation.

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

// PowderKeg concept: perfect bump → cell hit → shockwave on cell destruction
When(trigger: OnPerfectBump, then: [
    When(trigger: OnImpact(Cell), then: [
        When(trigger: OnCellDestroyed, then: [Do(Shockwave(base_range: 48.0, ...))])
    ])
])
```

## Triggers

Triggers are conditions that gate child node evaluation. Used inside `When` and `Until` nodes.

| Trigger (RON name) | Rust variant | Condition | Bolt Context |
|--------------------|-------------|-----------|-------------|
| `OnPerfectBump` | `PerfectBump` | Bump timed within the perfect window | Breaker entity |
| `OnEarlyBump` | `EarlyBump` | Bump pressed before the perfect zone | Breaker entity |
| `OnLateBump` | `LateBump` | Bump pressed after the bolt hit | Breaker entity |
| `OnBump` | `Bump` | Any non-whiff bump (Early, Late, or Perfect) | Breaker entity |
| `OnBumpWhiff` | `BumpWhiff` | Forward bump window expired without contact | Global (no specific bolt) |
| `NoBump` | `NoBump` | Bolt passed the breaker without any bump attempt | Breaker entity |
| `PerfectBumped` | `PerfectBumped` | "I was perfect-bumped" — bolt-perspective trigger | Specific bolt |
| `Bumped` | `Bumped` | "I was bumped" — bolt-perspective, any non-whiff | Specific bolt |
| `EarlyBumped` | `EarlyBumped` | "I was early-bumped" — bolt-perspective | Specific bolt |
| `LateBumped` | `LateBumped` | "I was late-bumped" — bolt-perspective | Specific bolt |
| `OnImpact(Cell)` | `Impact(Cell)` | Bolt hit a cell | Specific bolt from impact |
| `OnImpact(Breaker)` | `Impact(Breaker)` | Bolt bounced off the breaker | Specific bolt from impact |
| `OnImpact(Wall)` | `Impact(Wall)` | Bolt bounced off a wall | Specific bolt from impact |
| `OnCellDestroyed` | `CellDestroyed` | A cell was destroyed | Global (no specific bolt) |
| `OnBoltLost` | `BoltLost` | A bolt was lost (fell off screen) | Global (no specific bolt) |
| `OnDeath` | `Death` | Breaker lost all lives or timer expired | Global |
| `OnSelected` | `Selected` | Chip was selected on the upgrade screen | N/A — immediate evaluation |
| `TimeExpires(f32)` | `TimeExpires(f32)` | Elapsed seconds since the node was applied | N/A — timer-based removal |
| `OnNodeTimerThreshold(f32)` | `NodeTimerThreshold(f32)` | Node timer ratio drops below threshold | Breaker entity |

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

- **Specific bolt**: The effect targets the bolt that triggered the event. The bolt entity is carried in the typed event's `targets` field as `EffectTarget::Entity(entity)`.
- **Global**: No specific bolt. Effects that operate on all bolts (like `SpeedBoost`) query for all bolt entities. Effects that don't require a bolt (like `LoseLife`) fire normally regardless of context.

## Chain Ownership Model

Chains live on the entity whose events trigger them. The `Target` enum in effects handles cross-entity targeting at handler time.

### Three Effect Stores

- **`ActiveEffects`** — global resource (`Vec<(Option<String>, EffectNode)>`). Holds all breaker-definition and triggered-chip chains. Populated by `init_breaker` and `dispatch_chip_effects`. Bridge helpers sweep it for global and breaker-owned triggers (BoltLost, BumpWhiff, NoBump, CellDestroyed).
- **`ArmedEffects`** — component on bolt entities. Partially resolved `When` trees waiting for deeper triggers. Consumed on Fire, replaced on re-Arm.
- **`EffectChains`** — component on individual entities (bolts, cells). Entity-local chains evaluated by `evaluate_entity_chains`. Used for `Once`-wrapped one-shot effects, cell-specific chains, and `On`-node-dispatched sub-chains.

### Which Entity Owns Which Chains

| Trigger | Default Owner | Why |
|---------|--------------|-----|
| PerfectBump, Bump, EarlyBump, LateBump, BumpWhiff, NoBump | Breaker | Bump is a breaker event |
| Impact(Cell), Impact(Wall), Impact(Breaker) | Bolt | Bolt is the impactor |
| Death | The dying entity | Cell death chains on cell, bolt death on bolt |
| NodeTimerThreshold | Breaker | Node-level timer |
| Selected | N/A | Fires immediately |
| TimeExpires | N/A | Managed by Until timer system |

**Global triggers** — `CellDestroyed` and `BoltLost` are global events. During evaluation, bridges call `evaluate_active_chains` which sweeps all chains in `ActiveEffects`. The `bridge_cell_death` bridge reads `RequestCellDestroyed` (entity still alive) so effects can access the cell's components. It also evaluates the cell's own `Death` trigger. Then `cleanup_destroyed_cells` despawns the entity.

`ChipDefinition.effects` is `Vec<EffectNode>` (not a restricted wrapper). `BreakerDefinition` uses `effects: Vec<RootEffect>` — see the [RootEffect and Breaker Definition](#rooteffect-and-breaker-definition) section below for the full format and RON example.

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

When a chip is selected, `dispatch_chip_effects` processes each `EffectNode` in `ChipDefinition.effects`:
- **`When(trigger: OnSelected, ...)` at top level**: evaluates immediately, fires passive effects via `fire_passive_event`.
- **Other `When`/`Until`/`Once` nodes**: pushed to `ActiveEffects` for evaluation by bridge systems on matching game events.

### Evaluation Routing

Different triggers evaluate different entity sets:
- **Entity-specific triggers** (Impact, Bump, Death): evaluate only the relevant entity's EffectChains + ArmedEffects.
- **Global triggers** (CellDestroyed, BoltLost): sweep ALL entities with EffectChains and evaluate matching chains wherever they live.

### Arm Routing

When evaluation produces an Arm result and the inner trigger belongs to a different entity type, the armed chain moves to that entity's `ArmedEffects`. Example: `When(PerfectBump, [When(Impact(Wall), [Do(Shockwave)])])` — first Arm stays on breaker, second Arm moves to bolt (Impact is a bolt event).

## Effects (Leaves)

Leaf effects are the terminal `Do(...)` actions in an EffectNode tree. Bridge systems call `fire_typed_event` which dispatches the appropriate typed event (e.g., `ShockwaveFired`, `LoseLifeFired`). Each effect's observer handler in `effect/effects/` receives the typed event and executes the game action.

### Triggered Effects

These fire through the bridge system when their trigger condition is met.

| Effect | Parameters | Handler | Description |
|--------|-----------|---------|-------------|
| `Shockwave` | `base_range`, `range_per_level`, `stacks`, `speed` | `handle_shockwave` | Expanding ring of area damage. Effective range = `base_range + (stacks - 1) * range_per_level`. `speed` controls expansion rate in world units/sec. |
| `ChainBolt` | `tether_distance` | `handle_chain_bolt` | Spawns a chain bolt tethered to the triggering bolt via `DistanceConstraint`. |
| `SpawnBolts` | `count`, `lifespan`, `inherit` | `handle_spawn_bolts` | Spawns `count` additional bolts. Serde defaults: `lifespan: None` (permanent), `inherit: false` (no effect inheritance), `count: 1`. |
| `MultiBolt` | `base_count`, `count_per_level`, `stacks` | `handle_multi_bolt` | Spawns additional bolts. Effective count = `base_count + (stacks - 1) * count_per_level`. |
| `Shield` | `base_duration`, `duration_per_level`, `stacks` | `handle_shield` | Temporary shield. Effective duration = `base_duration + (stacks - 1) * duration_per_level`. |
| `LoseLife` | *(none)* | `handle_life_lost` | Decrements `LivesCount`. When lives reach 0, sends `RunLost`. |
| `TimePenalty` | `seconds` | `handle_time_penalty` | Subtracts time from the node timer. |
| `SpeedBoost` | `multiplier: f32` | `handle_speed_boost` | Scales the bolt velocity by `multiplier`. Uses 1.x format: 2.0 = 2x speed, 0.5 = 50% speed. Target is resolved from the `On` node or event context. |
| `RandomEffect` | `Vec<(f32, EffectNode)>` | `handle_random_effect` | Weighted random selection from a pool of effects. |
| `EntropyEngine` | `counter: u32`, `Vec<(f32, EffectNode)>` | `handle_entropy_engine` | Counter-gated `RandomEffect` — every Nth trigger, roll from pool. |
| `RampingDamage` | `bonus_per_hit: f32` | `handle_ramping_damage` | Stacking damage bonus on cell hits, resets on non-bump breaker impact. No maximum cap. |
| `SecondWind` | `invuln_secs: f32` | `handle_second_wind` | Spawns an invisible bottom wall that bounces the bolt once. Despawned after first hit. |
| `ChainLightning` | `arcs: u32`, `range: f32`, `damage_mult: f32` | `handle_chain_lightning` | Arc damage jumping between nearby cells using greedy nearest-neighbor traversal. |
| `PiercingBeam` | `damage_mult: f32`, `width: f32` | `handle_piercing_beam` | Fires a beam through all cells in the bolt's current velocity direction. |
| `SpawnPhantom` | `duration: f32`, `max_active: u32` | `handle_spawn_phantom` | Spawns a temporary phantom bolt with infinite piercing and a lifespan timer. |
| `GravityWell` | `strength: f32`, `duration: f32`, `radius: f32`, `max: u32` | `handle_gravity_well` | Spawns a gravity well entity that attracts bolts within radius for the given duration. |
| `Pulse` | `base_range: f32`, `range_per_level: f32`, `stacks: u32`, `speed: f32` | `handle_pulse` | Fires a shockwave at every active bolt position simultaneously. Functionally equivalent to Shockwave but targets all bolts. |

### Passive Effects (OnSelected Leaves)

These fire immediately when a chip is selected and modify entity components directly.

| Effect | Parameters | Target | Description |
|--------|-----------|--------|-------------|
| `Piercing` | `count: u32` | Bolt | Bolt passes through N cells before stopping |
| `DamageBoost` | `boost: f32` | Bolt | Fractional bonus damage per stack. Uses 1.x format: 2.0 = 2x damage. |
| `SpeedBoost` | `multiplier: f32` | Bolt | Multiplicative speed multiplier per stack (e.g., 1.1 = 10% boost). 2.0 = 2x speed. Passive apply fires `SpeedBoostApplied`; triggered form fires `SpeedBoostFired`. |
| `ChainHit` | `count: u32` | Bolt | Chains to N additional cells on hit |
| `SizeBoost` | `value: f32` | Bolt (radius) or Breaker (width) | Size increase per stack. Dispatched as `SizeBoostApplied` — bolt handler (`bolt_size_boost`) and breaker handler (`width_boost`) both receive it and apply to their respective entities. |
| `Attraction` | `type: AttractionType`, `force: f32` | Bolt | Attracts toward nearest entity of the given type. See Attraction section below. |
| `BumpForce` | `force: f32` | Breaker | Flat bump force increase per stack |
| `TiltControl` | `sensitivity: f32` | Breaker | Flat tilt control sensitivity increase per stack |

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

`Target` is used in `On { target, then }` nodes to scope effect dispatch to a specific entity type:

| Target | Behavior |
|--------|----------|
| `Bolt` | The specific bolt that triggered the event |
| `Breaker` | The breaker entity |
| `AllBolts` | All bolt entities currently in play |
| `Cell` | The specific cell entity that was hit |
| `Wall` | The wall entity that was hit |
| `AllCells` | All cell entities in the current node |

`SizeBoost` interpretation varies by context: `bolt_size_boost` handler applies to bolt radius; `width_boost` handler applies to breaker width.

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

## RootEffect and Breaker Definition

`RootEffect` is a top-level wrapper enum that constrains breaker definitions so every chain explicitly names its target entity before any trigger matching:

```rust
pub enum RootEffect {
    On { target: Target, then: Vec<EffectNode> }
}
```

`BreakerDefinition` has a single `effects: Vec<RootEffect>` field. Each entry is `On(target: ..., then: [...])`. At dispatch time `RootEffect` converts to `EffectNode::On`, establishing the entity context for child trigger evaluation.

## Breaker Usage

Each breaker defines root effect chains using `effects: Vec<RootEffect>`. All entries are `On(target, then: [When(trigger, then: [effect])])` style:

| Breaker | Effect chains |
|---------|--------------|
| **Aegis** | `On(Breaker, When(OnBoltLost, Do(LoseLife)))` · `On(Bolt, When(PerfectBumped, Do(SpeedBoost(1.5))))` · `On(Bolt, When(EarlyBumped, Do(SpeedBoost(1.1))))` · `On(Bolt, When(LateBumped, Do(SpeedBoost(1.1))))` |
| **Chrono** | `On(Breaker, When(OnBoltLost, Do(TimePenalty(5.0))))` · `On(Bolt, When(PerfectBumped, Do(SpeedBoost(1.5))))` · `On(Bolt, When(EarlyBumped, Do(SpeedBoost(1.1))))` · `On(Bolt, When(LateBumped, Do(SpeedBoost(1.1))))` |
| **Prism** | `On(Breaker, When(OnBoltLost, Do(TimePenalty(7.0))))` · `On(Breaker, When(PerfectBump, Do(SpawnBolts())))` |

All entries in `effects` are loaded into `ActiveEffects` at run start by `init_breaker`. Breakers may have any number of entries; there are no special named fields.

Example — Aegis RON:
```ron
(
    name: "Aegis",
    stat_overrides: (),
    life_pool: Some(3),
    effects: [
        On(target: Breaker, then: [When(trigger: OnBoltLost, then: [Do(LoseLife)])]),
        On(target: Bolt, then: [When(trigger: PerfectBumped, then: [Do(SpeedBoost(multiplier: 1.5))])]),
        On(target: Bolt, then: [When(trigger: EarlyBumped, then: [Do(SpeedBoost(multiplier: 1.1))])]),
        On(target: Bolt, then: [When(trigger: LateBumped, then: [Do(SpeedBoost(multiplier: 1.1))])]),
    ],
)
```
