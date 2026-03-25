# Triggers and Effects

Complete reference of the TriggerChain system — the unified model for ALL chip effects and archetype behaviors.

All chip effects — whether passive (applied on selection), triggered (fired on game events), or archetype-defined — are expressed as `TriggerChain` variants. There is no separate `ChipEffect`, `AmpEffect`, or `AugmentEffect` enum.

## Triggers

Triggers wrap an inner chain (another trigger or a leaf effect). When the trigger condition is met, the inner chain is evaluated: if it's a leaf, the effect fires immediately; if it's another trigger, the chain is armed on the bolt for later resolution.

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
| `OnSelected` | Chip was selected on the upgrade screen | N/A — immediate evaluation |

### OnSelected — Passive Effects

`OnSelected` is a special trigger that evaluates immediately when a chip is selected, rather than waiting for a game event. It replaces the old `Amp(AmpEffect::...)` and `Augment(AugmentEffect::...)` wrappers:

```ron
// Old: effects: [Amp(Piercing(1))]
// New:
effects: [OnSelected([Piercing(1)])]

// Old: effects: [Augment(WidthBoost(8.0))]
// New:
effects: [OnSelected([SizeBoost(Breaker, 8.0)])]
```

`OnSelected` takes a `Vec<TriggerChain>` — multiple passive effects can be applied in a single chip selection.

### Trigger Chaining

Triggers can nest arbitrarily deep. Each nesting layer adds one arm-then-resolve step before the leaf effect fires. Examples:
- `OnPerfectBump(Shockwave(...))` — depth 1, fires shockwave on perfect bump
- `OnPerfectBump(OnImpact(Cell, Shockwave(...)))` — depth 2, fires shockwave on cell impact after a perfect bump
- `OnPerfectBump(OnImpact(Cell, OnCellDestroyed(Shockwave(...))))` — depth 3, fires shockwave when the hit cell is destroyed after a perfect-bump cell impact

The evaluate/arm/resolve cycle is depth-agnostic: `evaluate()` peels the outermost trigger layer, `arm_bolt()` pushes the remaining chain onto the bolt's `ArmedTriggers`, and `resolve_armed()` re-evaluates on subsequent triggers — producing either another `Arm` (re-arm with a shorter chain) or `Fire` (execute the leaf). A 5-deep chain would arm 4 times then fire.

### Bolt Context

- **Specific bolt**: The effect targets the bolt that triggered the event. The bolt entity is passed through `EffectFired.bolt`.
- **Global**: No specific bolt. Effects that require a bolt entity (like SpeedBoost targeting Bolt) will no-op. Effects that don't require a bolt (like LoseLife) fire normally.

### Armed Triggers

When a trigger matches but the inner chain is another trigger (not a leaf), the remaining chain is "armed" on the bolt entity via the `ArmedTriggers` component. The armed chain is evaluated when the next matching trigger fires on that specific bolt.

## Effects (Leaves)

Leaf effects are the terminal actions in a trigger chain. They fire via the `EffectFired` event and are handled by dedicated observer systems.

### Triggered Effects

These fire through the bridge system when their trigger condition is met.

| Effect | Parameters | Handler | Description |
|--------|-----------|---------|-------------|
| `Shockwave` | `base_range`, `range_per_level`, `stacks` | `handle_shockwave` | Area damage within range. Effective range = `base_range + (stacks - 1) * range_per_level`. |
| `ChainBolt` | `tether_distance` | `handle_chain_bolt` | Spawns a chain bolt tethered to the triggering bolt via `DistanceConstraint`. |
| `MultiBolt` | `base_count`, `count_per_level`, `stacks` | *(not yet wired)* | Spawns additional bolts. Effective count = `base_count + (stacks - 1) * count_per_level`. |
| `Shield` | `base_duration`, `duration_per_level`, `stacks` | *(not yet wired)* | Temporary shield. Effective duration = `base_duration + (stacks - 1) * duration_per_level`. |
| `LoseLife` | *(none)* | `handle_life_lost` | Decrements `LivesCount`. When lives reach 0, sends `RunLost`. |
| `TimePenalty` | `seconds` | `handle_time_penalty` | Subtracts time from the node timer. |
| `SpawnBolt` | *(none)* | `handle_spawn_bolt` | Spawns one additional bolt. |
| `SpeedBoost` | `target: Target`, `multiplier: f32` | `handle_speed_boost` | Scales velocity of the target by `multiplier`. |
| `RandomEffect` | `Vec<(f32, TriggerChain)>` | `handle_random_effect` | Weighted random selection from a pool of effects. |
| `EntropyEngine` | `counter: u32`, `Vec<(f32, TriggerChain)>` | `handle_entropy_engine` | Counter-gated `RandomEffect` — every Nth trigger, roll from pool. |
| `RampingDamage` | `bonus_per_hit: f32`, `max_bonus: f32` | `handle_ramping_damage` | Stacking damage bonus on cell hits, resets on non-bump breaker impact. |
| `TimedSpeedBurst` | `speed_mult: f32`, `duration_secs: f32` | `handle_timed_speed_burst` | Temporary speed multiplier that decays after duration. |

### Passive Effects (OnSelected Leaves)

These fire immediately when a chip is selected and modify entity components directly.

| Effect | Parameters | Target | Description |
|--------|-----------|--------|-------------|
| `Piercing` | `count: u32` | Bolt | Bolt passes through N cells before stopping |
| `DamageBoost` | `boost: f32` | Bolt | Fractional bonus damage per stack |
| `SpeedBoost` | `target: Target`, `multiplier: f32` | Bolt or Breaker | Percentage-based speed multiplier per stack (e.g., 1.1 = 10% boost) |
| `ChainHit` | `count: u32` | Bolt | Chains to N additional cells on hit |
| `SizeBoost` | `target: Target`, `value: f32` | Bolt (radius) or Breaker (width) | Size increase per stack |
| `Attraction` | `force: f32` | Bolt | Attracts nearby cells |
| `BumpForce` | `force: f32` | Breaker | Flat bump force increase per stack |
| `TiltControl` | `sensitivity: f32` | Breaker | Flat tilt control sensitivity increase per stack |

### Target Enum

Effects that can target multiple entity types use the `Target` enum:

| Target | Behavior |
|--------|----------|
| `Bolt` | Affects the specific triggering bolt entity |
| `Breaker` | Affects the breaker entity |
| `AllBolts` | Affects all bolt entities in play |

`SizeBoost` interpretation varies by target: on `Bolt` it adjusts radius, on `Breaker` it adjusts width.

### Stacking

Shockwave, MultiBolt, and Shield support stacking via `stacks` and `*_per_level` fields. Each stack beyond the first adds the per-level bonus. Stacks are incremented at runtime when the same chip is selected again.

Passive effects (Piercing, DamageBoost, etc.) stack by incrementing the flat component on the entity.

## Archetype Usage

Each archetype defines root trigger chains for bump events:

| Archetype | on_bolt_lost | on_perfect_bump | on_early_bump | on_late_bump |
|-----------|-------------|-----------------|---------------|-------------|
| **Aegis** | LoseLife | SpeedBoost(Bolt, 1.5x) | SpeedBoost(Bolt, 1.1x) | SpeedBoost(Bolt, 1.1x) |
| **Chrono** | TimePenalty(5s) | SpeedBoost(Bolt, 1.5x) | SpeedBoost(Bolt, 1.1x) | SpeedBoost(Bolt, 1.1x) |
| **Prism** | TimePenalty(7s) | SpawnBolt | *(none)* | *(none)* |

Archetypes can also define additional chains in the `chains` field for more complex trigger combinations.
