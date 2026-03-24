# Triggers and Effects

Complete reference of the TriggerChain system — triggers (when), effects (what), and chip effects (passive).

## Triggers

Triggers wrap an inner chain (another trigger or a leaf effect). When the trigger condition is met, the inner chain is evaluated: if it's a leaf, the effect fires immediately; if it's another trigger, the chain is armed on the bolt for later resolution.

| Trigger | Condition | Bolt Context |
|---------|-----------|-------------|
| `OnPerfectBump` | Bump timed within the perfect window | Specific bolt from bump |
| `OnEarlyBump` | Bump pressed before the perfect zone | Specific bolt from bump |
| `OnLateBump` | Bump pressed after the bolt hit | Specific bolt from bump |
| `OnBumpSuccess` | Any non-whiff bump (Early, Late, or Perfect) | Specific bolt from bump |
| `OnBumpWhiff` | Forward bump window expired without contact | Global (no specific bolt) |
| `OnImpact(Cell)` | Bolt hit a cell | Specific bolt from impact |
| `OnImpact(Breaker)` | Bolt bounced off the breaker | Specific bolt from impact |
| `OnImpact(Wall)` | Bolt bounced off a wall | Specific bolt from impact |
| `OnCellDestroyed` | A cell was destroyed | Global (no specific bolt) |
| `OnBoltLost` | A bolt was lost (fell off screen) | Global (no specific bolt) |

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

| Effect | Parameters | Handler | Description |
|--------|-----------|---------|-------------|
| `Shockwave` | `base_range`, `range_per_level`, `stacks` | `handle_shockwave` | Area damage to all non-locked cells within range. Spawns an expanding ring entity; `shockwave_collision` queries the quadtree each tick. Damage = `BASE_BOLT_DAMAGE * (1.0 + DamageBoost)`. Effective range = `base_range + (stacks - 1) * range_per_level`. |
| `ChainBolt` | `tether_distance` | `handle_chain_bolt` | Spawns a chain bolt tethered to the triggering bolt via `SpawnChainBolt` message → `spawn_chain_bolt`. The tethered bolt is constrained to `tether_distance` from its anchor via `DistanceConstraint`. Despawned when anchor is lost. |
| `MultiBolt` | `base_count`, `count_per_level`, `stacks` | *(not yet wired)* | Spawns additional bolts. Effective count = `base_count + (stacks - 1) * count_per_level`. |
| `Shield` | `base_duration`, `duration_per_level`, `stacks` | *(not yet wired)* | Temporary shield protecting the breaker. Effective duration = `base_duration + (stacks - 1) * duration_per_level`. |
| `LoseLife` | *(none)* | `handle_life_lost` | Decrements `LivesCount` on the breaker. When lives reach 0, sends `RunLost`. |
| `TimePenalty` | `seconds` | `handle_time_penalty` | Subtracts time from the node timer. |
| `SpawnBolt` | *(none)* | `handle_spawn_bolt` | Spawns one additional bolt into play. |
| `SpeedBoost` | `target`, `multiplier` | `handle_speed_boost` | Scales velocity of the target by `multiplier`, clamped within `[BoltBaseSpeed + amp_boost, BoltMaxSpeed + amp_boost]`. |

### SpeedBoost Targets

| Target | Behavior |
|--------|----------|
| `Bolt` | Scales the specific triggering bolt's velocity |
| `Breaker` | *(future — no-op)* |
| `AllBolts` | *(future — no-op)* |

### Stacking

Shockwave, MultiBolt, and Shield support stacking via `stacks` and `*_per_level` fields. Each stack beyond the first adds the per-level bonus. Stacks are incremented at runtime when the same overclock chip is selected again.

## Chip Effects (Passive)

Chip effects are applied when a chip is selected during the upgrade screen. They modify entity components directly rather than firing through the trigger chain system.

### Amp Effects (Bolt Passives)

Applied to bolt entities. Stack additively.

| Effect | Parameter | Description |
|--------|-----------|-------------|
| `Piercing` | `count: u32` | Bolt passes through N cells before stopping |
| `DamageBoost` | `boost: f32` | Fractional bonus damage per stack. `damage = BASE_BOLT_DAMAGE * (1.0 + boost)` |
| `SpeedBoost` | `flat_speed: f32` | Adds flat speed to bolt's base and max speed per stack |
| `ChainHit` | `count: u32` | Bolt chains to N additional cells on hit |
| `SizeBoost` | `fraction: f32` | Increases bolt radius by a fraction per stack |

### Augment Effects (Breaker Passives)

Applied to the breaker entity. Stack additively.

| Effect | Parameter | Description |
|--------|-----------|-------------|
| `WidthBoost` | `flat_width: f32` | Adds flat width to the breaker per stack |
| `SpeedBoost` | `flat_speed: f32` | Adds flat speed to breaker max speed per stack |
| `BumpForce` | `force: f32` | Adds flat bump force per stack |
| `TiltControl` | `sensitivity: f32` | Adds flat tilt control sensitivity per stack |

### Overclock Effects (Triggered Abilities)

Overclocks are trigger chains. When selected as a chip, the chain is pushed to `ActiveChains` and evaluated by the bridge systems on each matching event. See **Triggers** and **Effects** above.

## Archetype Usage

Each archetype defines root trigger chains for bump events:

| Archetype | on_bolt_lost | on_perfect_bump | on_early_bump | on_late_bump |
|-----------|-------------|-----------------|---------------|-------------|
| **Aegis** | LoseLife | SpeedBoost(Bolt, 1.5x) | SpeedBoost(Bolt, 1.1x) | SpeedBoost(Bolt, 1.1x) |
| **Chrono** | TimePenalty(5s) | SpeedBoost(Bolt, 1.5x) | SpeedBoost(Bolt, 1.1x) | SpeedBoost(Bolt, 1.1x) |
| **Prism** | TimePenalty(7s) | SpawnBolt | *(none)* | *(none)* |

Archetypes can also define additional chains in the `chains` field for more complex trigger combinations.
