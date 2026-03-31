# Core Types

All core types live in `effect/core/types/` (directory module: `definitions/enums.rs` holds the enum types; `definitions/fire.rs` and `definitions/reverse.rs` hold the dispatch methods; all exported via `definitions/mod.rs` → `types/mod.rs`).

## Trigger

```rust
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub enum Trigger {
    // Bump triggers (global)
    PerfectBump,
    EarlyBump,
    LateBump,
    Bump,
    BumpWhiff,
    NoBump,

    // Bump triggers (targeted — bolt perspective)
    PerfectBumped,
    EarlyBumped,
    LateBumped,
    Bumped,

    // Impact triggers
    Impact(ImpactTarget),       // global — "an impact involving X happened"
    Impacted(ImpactTarget),     // targeted — "you were in an impact with X" (both participants)

    // Death triggers
    Death,                      // global — "something died"
    Died,                       // targeted — "I died"

    // Destruction / loss
    BoltLost,                   // global
    CellDestroyed,              // global

    // Node lifecycle
    NodeStart,                  // global
    NodeEnd,                    // global

    // Timer
    NodeTimerThreshold(f32),    // global — ratio threshold
    TimeExpires(f32),           // special — timer system ticks this
}
```

## ImpactTarget

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum ImpactTarget {
    Cell,
    Bolt,
    Wall,
    Breaker,
}
```

Used by `Impact(ImpactTarget)` and `Impacted(ImpactTarget)` trigger variants.

## Target

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum Target {
    Bolt,       // singular — context-sensitive at runtime, primary bolt at dispatch
    AllBolts,   // plural — all bolt entities (desugared at dispatch time)
    Breaker,    // the breaker entity
    Cell,       // singular — context-sensitive at runtime
    AllCells,   // plural — all cell entities (desugared at dispatch time)
    Wall,       // singular — context-sensitive at runtime
    AllWalls,   // plural — all wall entities (desugared at dispatch time)
}
```

See [Target Resolution](targets.md) for how each variant resolves at dispatch vs runtime.

## RootEffect

```rust
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum RootEffect {
    On { target: Target, then: Vec<EffectNode> },
}

impl From<RootEffect> for EffectNode {
    fn from(r: RootEffect) -> Self {
        let RootEffect::On { target, then } = r;
        EffectNode::On { target, permanent: false, then }
    }
}
```

Top-level wrapper for breaker/chip/cell definitions. Ensures every effect chain explicitly names its target. Used in `BreakerDefinition`, `ChipDefinition`, and `CellDefinition` as `effects: Vec<RootEffect>`.

## EffectNode

```rust
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum EffectNode {
    When { trigger: Trigger, then: Vec<EffectNode> },
    Do(EffectKind),
    Once(Vec<EffectNode>),
    On { target: Target, #[serde(default)] permanent: bool, then: Vec<EffectNode> },
    Until { trigger: Trigger, then: Vec<EffectNode> },
    Reverse { effects: Vec<EffectKind>, chains: Vec<EffectNode> },  // internal only, not in RON
}
```

See [Node Types](node_types.md) for detailed semantics of each variant.

## EffectKind

The enum uses **inline fields** on each variant for clean RON deserialization:

```rust
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum EffectKind {
    Shockwave { base_range: f32, range_per_level: f32, stacks: u32, speed: f32 },
    SpeedBoost { multiplier: f32 },
    DamageBoost(f32),
    Piercing(u32),
    SizeBoost(f32),
    BumpForce(f32),
    Attraction {
        attraction_type: AttractionType,
        force: f32,
        #[serde(default)] max_force: Option<f32>,  // clamps velocity delta per tick
    },
    LoseLife,
    TimePenalty { seconds: f32 },
    SpawnBolts { #[serde(default = "one")] count: u32, #[serde(default)] lifespan: Option<f32>, #[serde(default)] inherit: bool },
    ChainBolt { tether_distance: f32 },
    Shield { stacks: u32 },                        // stacks become charge count; no duration
    ChainLightning {
        arcs: u32,
        range: f32,
        damage_mult: f32,
        #[serde(default = "default_chain_lightning_arc_speed")] arc_speed: f32,  // default 200.0
    },
    PiercingBeam { damage_mult: f32, width: f32 },
    Pulse {
        base_range: f32,
        range_per_level: f32,
        stacks: u32,
        speed: f32,
        #[serde(default = "default_pulse_interval")] interval: f32,  // default 0.5
    },
    SecondWind,           // unit variant — no fields
    SpawnPhantom { duration: f32, max_active: u32 },
    GravityWell { strength: f32, duration: f32, radius: f32, max: u32 },
    RandomEffect(Vec<(f32, EffectNode)>),
    EntropyEngine { max_effects: u32, pool: Vec<(f32, EffectNode)> },   // note: max_effects, not threshold
    RampingDamage { damage_per_trigger: f32 },
    Explode { range: f32, damage_mult: f32 },
    QuickStop { multiplier: f32 },
    TetherBeam {
        damage_mult: f32,
        #[serde(default)] chain: bool,  // if true: chain mode — connects all bolts instead of spawning new ones
    },
    MirrorProtocol {
        #[serde(default)] inherit: bool,  // if true: spawned bolt gets parent's BoundEffects
    },
    Anchor { bump_force_multiplier: f32, perfect_window_multiplier: f32, plant_delay: f32 },
    FlashStep,              // unit variant — no fields
    CircuitBreaker {
        bumps_required: u32,
        #[serde(default = "one")] spawn_count: u32,
        #[serde(default)] inherit: bool,
        shockwave_range: f32,
        shockwave_speed: f32,
    },
}
```

This gives clean RON: `Shockwave(base_range: 24.0, speed: 400.0)` — no double-name wrapping.

## AttractionType

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum AttractionType {
    Cell,
    Wall,
    Breaker,
}
```

## EffectSourceChip

```rust
/// Deferred chip attribution stored on spawned effect entities (shockwave,
/// pulse ring, explode request, chain lightning chain, piercing beam request,
/// tether beam). Damage-application systems read this to populate
/// DamageCell.source_chip.
#[derive(Component, Debug, Clone, Default)]
pub struct EffectSourceChip(pub Option<String>);

impl EffectSourceChip {
    /// Create from a chip name: empty string → EffectSourceChip(None), non-empty → Some(owned).
    pub fn new(source_chip: &str) -> Self { ... }
    /// Extract the chip attribution for DamageCell.source_chip.
    pub fn source_chip(&self) -> Option<String> { ... }
}

/// Convert a source_chip string into Option<String>.
/// Empty string → None; non-empty → Some(s.to_string()).
pub fn chip_attribution(source_chip: &str) -> Option<String> { ... }
```

Lives in `effect/core/types/definitions/enums.rs`. Used by AoE/spawn effects that need to carry chip attribution
from dispatch time to damage-application time (since those effects damage cells on a later tick).

## fire() and reverse()

The enum has `fire()` and `reverse()` methods on `EffectKind`. Both take a `source_chip: &str`
parameter for chip attribution. Each match arm destructures the variant and calls the per-module
free function:

```rust
impl EffectKind {
    pub(crate) fn fire(&self, entity: Entity, source_chip: &str, world: &mut World) {
        match self {
            Self::Shockwave { base_range, range_per_level, stacks, speed } => {
                shockwave::fire(entity, *base_range, *range_per_level, *stacks, *speed, source_chip, world)
            }
            Self::SpeedBoost { multiplier } => speed_boost::fire(entity, *multiplier, source_chip, world),
            Self::DamageBoost(v) => damage_boost::fire(entity, *v, source_chip, world),
            Self::LoseLife => life_lost::fire(entity, source_chip, world),
            Self::SecondWind => second_wind::fire(entity, source_chip, world),
            // ... one arm per variant; falls through to fire_aoe_and_spawn / fire_utility_and_spawn / fire_breaker_effects
        }
    }

    pub(crate) fn reverse(&self, entity: Entity, source_chip: &str, world: &mut World) {
        match self {
            Self::Shockwave { .. } => shockwave::reverse(entity, source_chip, world),
            Self::SpeedBoost { multiplier } => speed_boost::reverse(entity, *multiplier, source_chip, world),
            Self::DamageBoost(v) => damage_boost::reverse(entity, *v, source_chip, world),
            Self::LoseLife => life_lost::reverse(entity, source_chip, world),
            Self::SecondWind => second_wind::reverse(entity, source_chip, world),
            // ... falls through to reverse_aoe_and_spawn / reverse_utility / reverse_breaker_effects — ALL variants covered
        }
    }
}
```

The `fire` match is split across **four** private methods (`fire`, `fire_aoe_and_spawn`,
`fire_utility_and_spawn`, `fire_breaker_effects`) purely for line count. The `reverse` match is
split across **four** private methods (`reverse`, `reverse_aoe_and_spawn`, `reverse_utility`,
`reverse_breaker_effects`). All splits are exhaustive.

## Per-Effect Modules

Each effect module (`effect/effects/<name>.rs` or `effect/effects/<name>/` directory module) defines free functions and any active-state components:

```rust
// effect/effects/speed_boost.rs

// Active state component (tracks applied multipliers on the entity)
pub struct ActiveSpeedBoosts(pub Vec<f32>);

pub(crate) fn fire(entity: Entity, multiplier: f32, source_chip: &str, world: &mut World) {
    // push multiplier to ActiveSpeedBoosts component
    // source_chip is accepted for API uniformity but not used by stat effects
}

pub(crate) fn reverse(entity: Entity, multiplier: f32, source_chip: &str, world: &mut World) {
    // remove matching entry from ActiveSpeedBoosts
}

pub(crate) fn register(app: &mut App) {
    // simple stat effects have no runtime systems to register;
    // effects with runtime behavior (shockwave tick, tether beam, etc.) add systems here
}
```

The module is self-contained: fire, reverse, components, runtime systems, registration. All logic lives in the module — the enum match is mechanical dispatch only.

## Why No Trait

An Effect trait would require each effect to be a **separate struct** wrapping its params. The enum would then be `Shockwave(ShockwaveParams)` — a newtype variant. RON would serialize this as `Shockwave(ShockwaveParams(base_range: 24.0))` or `Shockwave((base_range: 24.0))` — double-wrapped and ugly.

Inline fields on the enum give clean RON. The exhaustive match gives compile-time enforcement (add a variant → compiler forces you to add fire/reverse arms). The trait doesn't add value beyond what the match already provides.
