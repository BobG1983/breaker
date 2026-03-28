# Core Types

All core types live in `effect/core/types.rs`.

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
    Attraction(AttractionType, f32),
    TiltControl(f32),
    LoseLife,
    TimePenalty { seconds: f32 },
    SpawnBolts { #[serde(default = "one")] count: u32, #[serde(default)] lifespan: Option<f32>, #[serde(default)] inherit: bool },
    MultiBolt { base_count: u32, count_per_level: u32, stacks: u32 },
    ChainBolt { tether_distance: f32 },
    Shield { base_duration: f32, duration_per_level: f32, stacks: u32 },
    ChainLightning { arcs: u32, range: f32, damage_mult: f32 },
    PiercingBeam { damage_mult: f32, width: f32 },
    Pulse { base_range: f32, range_per_level: f32, stacks: u32, speed: f32 },
    SecondWind { invuln_secs: f32 },
    SpawnPhantom { duration: f32, max_active: u32 },
    GravityWell { strength: f32, duration: f32, radius: f32, max: u32 },
    RandomEffect(Vec<(f32, EffectNode)>),
    EntropyEngine { threshold: u32, pool: Vec<(f32, EffectNode)> },
    RampingDamage { bonus_per_hit: f32 },
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

## fire() and reverse()

The enum has `fire()` and `reverse()` methods. Each match arm destructures the variant and calls the per-module function:

```rust
impl EffectKind {
    pub(crate) fn fire(&self, entity: Entity, world: &mut World) {
        match self {
            Self::Shockwave { base_range, range_per_level, stacks, speed } => {
                shockwave::fire(entity, *base_range, *range_per_level, *stacks, *speed, world)
            }
            Self::SpeedBoost { multiplier } => {
                speed_boost::fire(entity, *multiplier, world)
            }
            Self::DamageBoost(value) => {
                damage_boost::fire(entity, *value, world)
            }
            Self::LoseLife => {
                life_lost::fire(entity, world)
            }
            // ... one arm per variant
        }
    }

    pub(crate) fn reverse(&self, entity: Entity, world: &mut World) {
        match self {
            Self::Shockwave { .. } => shockwave::reverse(entity, world),
            Self::SpeedBoost { multiplier } => speed_boost::reverse(entity, *multiplier, world),
            Self::DamageBoost(value) => damage_boost::reverse(entity, *value, world),
            Self::LoseLife => life_lost::reverse(entity, world),
            // ... ALL variants — every effect defines reverse
        }
    }
}
```

## Per-Effect Modules

Each effect module (`effect/effects/<name>.rs`) defines free functions:

```rust
// effect/effects/speed_boost.rs

pub(crate) fn fire(entity: Entity, multiplier: f32, world: &mut World) {
    // query entity for Velocity2D, BoltBaseSpeed, BoltMaxSpeed
    // scale velocity, push to ActiveSpeedBoosts
}

pub(crate) fn reverse(entity: Entity, multiplier: f32, world: &mut World) {
    // remove matching entry from ActiveSpeedBoosts
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(FixedUpdate, apply_speed_boosts.run_if(in_state(PlayingState::Active)));
}
```

The module is self-contained: fire, reverse, components, runtime systems, registration. All logic lives in the module — the enum match is mechanical dispatch only.

## Why No Trait

An Effect trait would require each effect to be a **separate struct** wrapping its params. The enum would then be `Shockwave(ShockwaveParams)` — a newtype variant. RON would serialize this as `Shockwave(ShockwaveParams(base_range: 24.0))` or `Shockwave((base_range: 24.0))` — double-wrapped and ugly.

Inline fields on the enum give clean RON. The exhaustive match gives compile-time enforcement (add a variant → compiler forces you to add fire/reverse arms). The trait doesn't add value beyond what the match already provides.
