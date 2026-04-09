# Core Types

All core types live in `effect/core/types/` (directory module: `definitions/enums.rs` holds the enum types; `definitions/fire.rs` and `definitions/reverse.rs` hold the dispatch functions; all exported via `definitions/mod.rs` → `types/mod.rs`).

## EffectType

The full set of fireable effects. Simple effects use bare variants; complex effects use **config struct wrappers** (newtype variants). Each config struct is defined in its per-effect module with `#[derive(Clone, Debug, PartialEq, Deserialize)]`.

```rust
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum EffectType {
    // Stat effects (simple — bare variants)
    SpeedBoost(f32), SizeBoost(f32), DamageBoost(f32),
    BumpForce(f32), QuickStop(f32), FlashStep,
    Piercing(u32), RampingDamage(f32), Vulnerable(f32),

    // Stat effects (config structs)
    Attraction(AttractionConfig), Anchor(AnchorConfig),
    Pulse(PulseConfig), Shield(ShieldConfig), SecondWind,

    // AoE / spawn effects (config structs)
    Shockwave(ShockwaveConfig), Explode(ExplodeConfig),
    ChainLightning(ChainLightningConfig), PiercingBeam(PiercingBeamConfig),
    SpawnBolts(SpawnBoltsConfig), SpawnPhantom(SpawnPhantomConfig),
    ChainBolt(ChainBoltConfig), MirrorProtocol(MirrorConfig),
    TetherBeam(TetherBeamConfig),

    // Penalty / lifecycle
    LoseLife, TimePenalty(f32), Die,

    // Meta effects
    CircuitBreaker(CircuitBreakerConfig), EntropyEngine(EntropyConfig),
    RandomEffect(Vec<(f32, Box<EffectType>)>),  // weighted pool, Box avoids infinite size
}
```

RON syntax uses double parens for config struct variants: `Shockwave((base_range: 24.0, speed: 500.0))`. Simple variants stay clean: `SpeedBoost(1.5)`, `Piercing(3)`.

## ReversibleEffectType

Subset of `EffectType` restricted to effects that can be reversed (undone). Used in `During` and `Until` direct fire positions where the scoped context must reverse the effect on exit.

```rust
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum ReversibleEffectType {
    SpeedBoost(f32), SizeBoost(f32), DamageBoost(f32),
    BumpForce(f32), QuickStop(f32), FlashStep,
    Piercing(u32), Attraction(AttractionConfig), RampingDamage(f32),
    Anchor(AnchorConfig), Vulnerable(f32), Pulse(PulseConfig),
    Shield(ShieldConfig), SecondWind,
}
```

Non-reversible effects (Shockwave, Explode, SpawnBolts, LoseLife, Die, etc.) cannot appear in direct `During`/`Until` fire position. They can appear inside a nested `When` within a scoped context (the listener itself is reversed, not the effect).

## Trigger

```rust
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum Trigger {
    // Local bump triggers (past-tense — "you were bumped")
    PerfectBumped,
    EarlyBumped,
    LateBumped,
    Bumped,

    // Local impact triggers
    Impacted(EntityKind),       // "you were in an impact with X" (both participants)

    // Local death triggers
    Died,                       // "I died"
    Killed(EntityKind),         // "I killed X" (killer perspective)

    // Global bump triggers (Occurred suffix)
    PerfectBumpOccurred,
    EarlyBumpOccurred,
    LateBumpOccurred,
    BumpOccurred,
    BumpWhiffOccurred,
    NoBumpOccurred,

    // Global impact / death / loss triggers
    ImpactOccurred(EntityKind),
    DeathOccurred(EntityKind),
    BoltLostOccurred,

    // Node lifecycle (global)
    NodeStartOccurred,
    NodeEndOccurred,
    NodeTimerThresholdOccurred(f32),    // ratio threshold

    // Timer (special — timer system ticks this)
    TimeExpires(f32),
}
```

**Local vs global**: Local triggers (past-tense, no suffix) fire on the entities involved in the event. Global triggers (`Occurred` suffix) fire on all entities that have matching BoundEffects/StagedEffects entries.

## EntityKind (trigger parameter)

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum EntityKind {
    Cell,
    Bolt,
    Wall,
    Breaker,
    Any,
}
```

Specifies **what type of entity** is involved. Used as the trigger parameter for:
- `Impacted(EntityKind)` and `ImpactOccurred(EntityKind)` — what entity type was in the impact
- `Killed(EntityKind)` — "I killed X" (killer perspective)
- `DeathOccurred(EntityKind)` — "something of type X died"

Includes `Any` to match all entity types. Replaces the old separate `KillTarget` and entity-type `ImpactTarget`/`DeathTarget` enums.

**Not to be confused with** the participant role enums (`ImpactTarget { Impactor, Impactee }`, `DeathTarget { Victim, Killer }`) used in `On(...)` for participant redirect. See [Participant Enums](#participant-enums) below.

## Condition

```rust
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
pub enum Condition {
    NodeActive,
    ShieldActive,
    ComboActive(u32),
}
```

Used by `During(Condition, ...)` to scope effects to a boolean condition.

## EntityType

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum EntityType { Bolt, Cell, Wall, Breaker }
```

Used by `Spawned(EntityType, ...)` — fires when an entity of this type is added.

## Participant Enums

Per-trigger role enums that identify participants in an event. Used with `On(ParticipantTarget, ...)` to redirect fire/stamp/transfer to a specific participant instead of `This`. These are **role** enums (who in the event), not entity-type enums. Entity-type filtering uses `EntityKind` (above).

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BumpTarget { Bolt, Breaker }

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ImpactTarget { Impactor, Impactee }    // participant ROLE — not entity type

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeathTarget { Victim, Killer }         // participant ROLE — not entity type

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoltLostTarget { Bolt, Breaker }
```

**`BumpTarget`** — participants in bump triggers (`PerfectBumped`, `Bumped`, etc. and their `Occurred` variants).

**`ImpactTarget` (participant role)** — `Impactor` / `Impactee` roles in impact triggers. Entity-type filtering on triggers uses `EntityKind`, not this enum.

**`DeathTarget` (participant role)** — `Victim` / `Killer` roles in death triggers. Entity-type filtering on triggers uses `EntityKind`, not this enum.

**`BoltLostTarget`** — participants in bolt-lost triggers.

### ParticipantTarget

Wraps the per-trigger participant enums into a single type for `On(...)`:

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParticipantTarget {
    Bump(BumpTarget),
    Impact(ImpactTarget),       // ImpactTarget = { Impactor, Impactee }
    Death(DeathTarget),         // DeathTarget = { Victim, Killer }
    BoltLost(BoltLostTarget),
}
```

No `This` variant — `Fire` targets `This` implicitly. No entity-type variants — `Route` handles routing at definition level.

## RouteTarget

```rust
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum RouteTarget {
    // Singular
    Bolt, Breaker, Cell, Wall,
    // Plural
    ActiveBolts, EveryBolt, PrimaryBolts, ExtraBolts,
    ActiveCells, EveryCells,
    ActiveWalls, EveryWall,
    ActiveBreakers, EveryBreaker,
}
```

Definition-time entity routing. Required at the root of every `effects: []` entry. Determines which entity (or entities) the tree's `This` resolves to.

## ValidDef

```rust
#[derive(Clone, Debug, PartialEq)]
pub struct ValidDef {
    pub route_target: RouteTarget,
    pub tree: ValidTree,
}
```

Top-level wrapper for breaker/chip/cell definitions. Ensures every effect chain explicitly names its route target. Used in `BreakerDefinition`, `ChipDefinition`, and `CellDefinition` as `effects: Vec<ValidDef>`.

Replaces the old `RootEffect` struct.

## ValidTree

The validated tree structure produced by the builder. Structural enforcement: `During`/`Until` can only contain reversible effects in direct `Fire` position.

```rust
#[derive(Clone, Debug, PartialEq)]
pub enum ValidTree {
    Fire(EffectType),                               // fire on This (implicit)
    Sequence(Vec<ValidTree>),                        // multiple children at same level
    When(Trigger, Box<ValidTree>),                   // trigger → subtree
    Once(Trigger, Box<ValidTree>),                   // same as When, self-removes after first match
    During(Condition, ValidScopedTree),              // condition-scoped (fire on start, reverse on end)
    Until(Trigger, ValidScopedTree),                 // event-scoped (fire immediately, reverse when trigger fires)
    Spawned(EntityType, Box<ValidTree>),             // fires on entity add
    On(ParticipantTarget, ValidTerminal),            // redirect to non-This target
}

#[derive(Clone, Debug, PartialEq)]
pub enum ValidScopedTree {
    Fire(ReversibleEffectType),                      // direct fire — reversible only
    Sequence(Vec<ReversibleEffectType>),             // multiple reversible effects
    When(Trigger, Box<ValidTree>),                   // nested When → relaxed (any effect OK)
    On(ParticipantTarget, ValidScopedTerminal),      // redirect — reversible only
}

#[derive(Clone, Debug, PartialEq)]
pub enum ValidTerminal {
    Fire(EffectType),                               // any effect, immediate
    Stamp(Box<ValidTree>),                           // permanent → target's BoundEffects
    Transfer(Box<ValidTree>),                        // one-shot → target's StagedEffects
    Reverse(ReversibleEffectType),                   // internal only — generated by During/Until desugaring
}

#[derive(Clone, Debug, PartialEq)]
pub enum ValidScopedTerminal {
    Fire(ReversibleEffectType),                      // reversible only
    Stamp(Box<ValidTree>),                           // stamp always OK (removable via BoundEffects cleanup)
    Transfer(Box<ValidTree>),                        // transfer always OK
}
```

`Reverse` never appears in RON. It is generated internally when `During`/`Until` desugar their reversal entries into `StagedEffects`:

```
During(NodeActive, Fire(SpeedBoost(1.3)))
  → fires SpeedBoost immediately
  → stages: When(NodeEndOccurred, Reverse(SpeedBoost(1.3)))
```

## TriggerContext

Typed per-trigger context structs. Each trigger concept has its own struct with named fields, wrapped in an enum:

```rust
pub enum TriggerContext {
    Bump(BumpContext),
    Impact(ImpactContext),
    Death(DeathContext),
    BoltLost(BoltLostContext),
    None,
}

// Chip attribution comes from BoundEntry.source, not TriggerContext.
pub struct BumpContext { pub bolt: Entity, pub breaker: Entity, pub depth: u32 }
pub struct ImpactContext { pub impactor: Entity, pub impactee: Entity, pub depth: u32 }
pub struct DeathContext { pub victim: Entity, pub killer: Option<Entity>, pub depth: u32 }
pub struct BoltLostContext { pub bolt: Entity, pub breaker: Entity, pub depth: u32 }
```

`ParticipantTarget` resolves against `TriggerContext` at dispatch time: `BumpTarget::Bolt` extracts `BumpContext.bolt`, `DeathTarget::Killer` extracts `DeathContext.killer`, etc.

## GameEntity

```rust
/// Marker trait for entity types that participate in the death pipeline.
/// Used as the generic bound on KillYourself<T> and Destroyed<T>.
trait GameEntity: Component {}
impl GameEntity for Bolt {}
impl GameEntity for Cell {}
impl GameEntity for Wall {}
impl GameEntity for Breaker {}
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

Lives in `effect/core/types/definitions/enums.rs`. Used by AoE/spawn effects that carry chip attribution from dispatch time to damage-application time (since those effects damage cells on a later tick).

## dispatch_fire() and dispatch_reverse()

Free functions that match on `EffectType` / `ReversibleEffectType` and delegate to per-effect module functions. Each takes a `source_chip: &str` parameter for chip attribution.

```rust
pub(crate) fn dispatch_fire(
    effect: &EffectType,
    entity: Entity,
    source_chip: &str,
    context: &TriggerContext,
    world: &mut World,
) {
    match effect {
        EffectType::Shockwave(config) => shockwave::fire(entity, config, source_chip, world),
        EffectType::SpeedBoost(multiplier) => speed_boost::fire(entity, *multiplier, source_chip, world),
        EffectType::DamageBoost(v) => damage_boost::fire(entity, *v, source_chip, world),
        EffectType::LoseLife => life_lost::fire(entity, source_chip, world),
        // ... one arm per variant
    }
}

pub(crate) fn dispatch_reverse(
    effect: &ReversibleEffectType,
    entity: Entity,
    source_chip: &str,
    world: &mut World,
) {
    match effect {
        ReversibleEffectType::SpeedBoost(multiplier) => speed_boost::reverse(entity, *multiplier, source_chip, world),
        ReversibleEffectType::DamageBoost(v) => damage_boost::reverse(entity, *v, source_chip, world),
        // ... one arm per variant (all reversible variants covered)
    }
}
```

The `dispatch_fire` match is split across multiple private functions purely for line count. Both dispatches are exhaustive — add a variant and the compiler forces you to add arms.

## Per-Effect Modules

Each effect module (`effect/effects/<name>.rs` or `effect/effects/<name>/` directory module) defines free functions and any active-state components:

```rust
// effect/effects/speed_boost.rs

// Active state component (tracks applied multipliers on the entity)
pub struct ActiveSpeedBoosts(pub Vec<f32>);

pub(crate) fn fire(entity: Entity, multiplier: f32, source_chip: &str, context: &TriggerContext, world: &mut World) {
    // push multiplier to ActiveSpeedBoosts, recalculate velocity
    // source_chip accepted for API uniformity but not used by stat effects
    // context accepted for API uniformity — effects that deal damage use it for attribution
}

pub(crate) fn reverse(entity: Entity, multiplier: f32, source_chip: &str, world: &mut World) {
    // remove matching entry from ActiveSpeedBoosts
}

pub(crate) fn register(app: &mut App) {
    // simple stat effects have no runtime systems;
    // effects with runtime behavior (shockwave tick, tether beam, etc.) add systems here
}
```

The module is self-contained: fire, reverse, components, runtime systems, registration. All logic lives in the module — the enum match in `dispatch_fire`/`dispatch_reverse` is mechanical dispatch only.

## Config Structs

Each complex effect has a config struct defined in its per-effect module. Config structs derive `Clone, Debug, PartialEq, Deserialize`. They are not listed exhaustively here — see the individual effect modules under `effect/effects/`. Examples:

- `ShockwaveConfig { base_range: f32, range_per_level: f32, stacks: u32, speed: f32 }`
- `ExplodeConfig { range: f32, damage: f32 }`
- `AttractionConfig { attraction_type: ..., force: f32, max_force: Option<f32> }`
- `SpawnBoltsConfig { count: u32, lifespan: Option<f32>, inherit: bool }`

RON: `Shockwave((base_range: 24.0, range_per_level: 0.0, stacks: 1, speed: 400.0))` — the double parens are the newtype variant wrapping the struct.

## Why No Trait

An Effect trait would require each effect to be a **separate struct** implementing the trait. The enum would then need dynamic dispatch or a second layer of indirection. The exhaustive match on `EffectType` gives compile-time enforcement (add a variant, compiler forces you to add fire/reverse arms) without trait boilerplate. Config struct variants give acceptable RON: `Shockwave((base_range: 24.0))` — one layer of parens for the newtype. The trait does not add value beyond what the match already provides.
