# Core Types

All effect-system types live in `breaker-game/src/effect_v3/types/`. Each type sits in its own file (`effect_type.rs`, `tree.rs`, `root_node.rs`, etc.) and is re-exported from `types/mod.rs`. The supporting traits live in `effect_v3/traits/` (`fireable.rs`, `reversible.rs`, `passive_effect.rs`).

## EffectType

Every effect in the game. Each variant **wraps a per-effect config struct** that implements the [`Fireable`](#fireable-and-reversible-traits) trait. The enum is the dispatch layer; the config struct is the implementation. There are no bare-scalar variants — even single-field configs (`SpeedBoostConfig { multiplier: OrderedFloat<f32> }`) are wrapped.

```rust
pub enum EffectType {
    SpeedBoost(SpeedBoostConfig),
    SizeBoost(SizeBoostConfig),
    DamageBoost(DamageBoostConfig),
    BumpForce(BumpForceConfig),
    QuickStop(QuickStopConfig),
    FlashStep(FlashStepConfig),
    Piercing(PiercingConfig),
    Vulnerable(VulnerableConfig),
    RampingDamage(RampingDamageConfig),
    Attraction(AttractionConfig),
    Anchor(AnchorConfig),
    Pulse(PulseConfig),
    Shield(ShieldConfig),
    SecondWind(SecondWindConfig),
    Shockwave(ShockwaveConfig),
    Explode(ExplodeConfig),
    ChainLightning(ChainLightningConfig),
    PiercingBeam(PiercingBeamConfig),
    SpawnBolts(SpawnBoltsConfig),
    SpawnPhantom(SpawnPhantomConfig),
    ChainBolt(ChainBoltConfig),
    MirrorProtocol(MirrorConfig),
    TetherBeam(TetherBeamConfig),
    GravityWell(GravityWellConfig),
    LoseLife(LoseLifeConfig),
    TimePenalty(TimePenaltyConfig),
    Die(DieConfig),
    CircuitBreaker(CircuitBreakerConfig),
    EntropyEngine(EntropyConfig),
    RandomEffect(RandomEffectConfig),
}
```

All variants derive `Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize`. Float fields inside config structs use `OrderedFloat<f32>` so the whole enum can derive `Hash` and `Eq`.

RON syntax wraps the config struct: `Fire(SpeedBoost(multiplier: 1.5))`, `Fire(Shockwave(base_range: 24.0, range_per_level: 6.0, stacks: 1, speed: 400.0))`.

The variant count is checked by `fire_dispatch_does_not_panic_for_any_effect_type_variant` in `dispatch/fire_dispatch.rs` — that test asserts `types.len() == 30`, so adding a variant requires updating it.

## ReversibleEffectType

The 16-variant subset of `EffectType` whose configs implement [`Reversible`](#fireable-and-reversible-traits). Used in `ScopedTree::Fire` (the only `Fire` position inside `During`/`Until` scoped contexts) and in `commands.reverse_effect()`.

```rust
pub enum ReversibleEffectType {
    SpeedBoost(SpeedBoostConfig),  SizeBoost(SizeBoostConfig),
    DamageBoost(DamageBoostConfig), BumpForce(BumpForceConfig),
    QuickStop(QuickStopConfig),     FlashStep(FlashStepConfig),
    Piercing(PiercingConfig),       Vulnerable(VulnerableConfig),
    RampingDamage(RampingDamageConfig),
    Attraction(AttractionConfig),   Anchor(AnchorConfig),
    Pulse(PulseConfig),             Shield(ShieldConfig),
    SecondWind(SecondWindConfig),
    CircuitBreaker(CircuitBreakerConfig),
    EntropyEngine(EntropyConfig),
}
```

Two conversions sit alongside the enum:

- `impl From<ReversibleEffectType> for EffectType` — widening, infallible (every reversible variant has an `EffectType` twin).
- `impl TryFrom<EffectType> for ReversibleEffectType` — narrowing, returns `Err(())` for non-reversible variants (`Shockwave`, `Explode`, `LoseLife`, `Die`, etc.).

Non-reversible effects can still appear inside a scoped context, but only nested under a `When` (where reversibility is not required because the listener — not the effect — is what gets removed). See `node_types.md` for the structural rules.

## Tree

The recursive effect tree. `Tree` is what gets stored inside `BoundEffects` and `StagedEffects`. It is also the payload of every `Stamp`/`Spawn` root node.

```rust
pub enum Tree {
    Fire(EffectType),
    When(Trigger, Box<Self>),
    Once(Trigger, Box<Self>),
    During(Condition, Box<ScopedTree>),
    Until(Trigger, Box<ScopedTree>),
    Sequence(Vec<Terminal>),
    On(ParticipantTarget, Terminal),
}
```

Variant semantics live in `node_types.md`. Walker entry points live in `walking/walk_effects/system.rs` (`walk_bound_effects`, `walk_staged_effects`, `evaluate_tree`).

## ScopedTree

The restricted tree that appears inside `During`/`Until`. Direct `Fire` is reversible-only; nested `When` re-opens the gate to the full `Tree`.

```rust
pub enum ScopedTree {
    Fire(ReversibleEffectType),
    Sequence(Vec<ReversibleEffectType>),
    When(Trigger, Box<Tree>),
    On(ParticipantTarget, ScopedTerminal),
    During(Condition, Box<Self>),
}
```

The structural restriction is the only thing that distinguishes `ScopedTree` from `Tree`. The reversibility constraint is enforced at the type level — the loader cannot construct a `ScopedTree::Fire(non_reversible)` because `ScopedTree::Fire` only takes `ReversibleEffectType`.

## Terminal and ScopedTerminal

Leaf operations used by `Tree::Sequence` and `Tree::On` (and their scoped equivalents).

```rust
pub enum Terminal {
    Fire(EffectType),
    Route(RouteType, Box<Tree>),
}

pub enum ScopedTerminal {
    Fire(ReversibleEffectType),
    Route(RouteType, Box<Tree>),
}

impl From<ScopedTerminal> for Terminal { /* widens Fire variant */ }
```

`Route(RouteType, Tree)` installs a tree onto a participant entity at runtime. `RouteType` controls permanence:

```rust
pub enum RouteType {
    Bound,   // → target's BoundEffects (re-arms each match)
    Staged,  // → target's StagedEffects (consumed on first match)
}
```

## RootNode

Top-level entry point for a chip/breaker/cell `effects:` list in RON. A `RootNode` either installs a tree onto entities that exist now, or watches for spawns of a given kind.

```rust
pub enum RootNode {
    Stamp(StampTarget, Tree),
    Spawn(EntityKind, Tree),
}
```

`Stamp` is dispatched by `chips/systems/dispatch_chip_effects` (and the breaker/cell equivalents). `Spawn` is registered into `SpawnStampRegistry` and applied later by `stamp_spawned_*` watcher systems when matching entities are added. See `dispatch.md`.

## StampTarget

Identifies which entities a `Stamp` root installs onto. Used at root level only — not at runtime trigger redirection (that's `ParticipantTarget`).

```rust
pub enum StampTarget {
    Bolt,                // primary bolt entity
    Breaker,             // primary breaker entity
    ActiveBolts,         // all bolts that exist right now
    EveryBolt,           // existing + future bolts (registered via SpawnStampRegistry)
    PrimaryBolts,        // bolts marked with PrimaryBolt
    ExtraBolts,          // bolts marked with ExtraBolt
    ActiveCells,         // all cells that exist right now
    EveryCell,           // existing + future cells
    ActiveWalls,         // all walls that exist right now
    EveryWall,           // existing + future walls
    ActiveBreakers,      // all breakers that exist right now
    EveryBreaker,        // existing + future breakers
}
```

The `Active*` and `Every*` distinction matters for chips equipped between nodes: cells/walls don't exist yet at chip-select time, so `Active*` is a no-op while `Every*` registers a `SpawnStampRegistry` entry that fires on the next spawn. See `dispatch.md` for the actual dispatch flow.

## Trigger

A game event that gates effect tree evaluation. Local triggers fire on specific participating entities; global triggers (suffix `Occurred`) fire on every entity that has effects bound or staged.

```rust
pub enum Trigger {
    // Local bump (fires on bolt + breaker)
    PerfectBumped, EarlyBumped, LateBumped, Bumped,

    // Global bump (fires on all entities)
    PerfectBumpOccurred, EarlyBumpOccurred, LateBumpOccurred,
    BumpOccurred, BumpWhiffOccurred, NoBumpOccurred,

    // Impact (collision)
    Impacted(EntityKind),                // local — this entity collided
    ImpactOccurred(EntityKind),          // global — a collision happened

    // Death
    Died,                                // local — this entity died
    Killed(EntityKind),                  // local — this entity killed something
    DeathOccurred(EntityKind),           // global — something of kind X died

    // Loss / lifecycle
    BoltLostOccurred,
    NodeStartOccurred,
    NodeEndOccurred,
    NodeTimerThresholdOccurred(OrderedFloat<f32>),

    // Self
    TimeExpires(OrderedFloat<f32>),      // owner-only countdown
}
```

All `f32` payloads are `OrderedFloat<f32>` so `Trigger` can derive `Hash` and `Eq` and be used as a dictionary key during equality comparison in walkers.

## Condition

A state predicate evaluated each frame by `evaluate_conditions` for `During` nodes. Conditions are stateful (start/end), not edge-triggered like triggers.

```rust
pub enum Condition {
    NodeActive,             // node state machine in Playing
    ShieldActive,           // at least one ShieldWall entity exists
    ComboActive(u32),       // perfect-bump streak ≥ N
}
```

Each condition has a free predicate function in `effect_v3/conditions/` (`is_node_active`, `is_shield_active`, `is_combo_active`) called by `evaluate_condition`.

## EntityKind

Classifies entity *types* for trigger matching and for `Spawn` root nodes. Distinct from `ParticipantTarget` (which classifies *roles*).

```rust
pub enum EntityKind {
    Cell, Bolt, Wall, Breaker, Any,
}
```

`Any` is valid in trigger payloads (`Impacted(Any)` matches any collision) but not in `Spawn(Any, ...)` — spawn watchers are per-kind.

## ParticipantTarget

Identifies a *role* in a trigger event. Used by `Tree::On` to redirect a terminal to a specific participant. Wraps per-trigger role enums.

```rust
pub enum BumpTarget       { Bolt, Breaker }
pub enum ImpactTarget     { Impactor, Impactee }
pub enum DeathTarget      { Victim, Killer }
pub enum BoltLostTarget   { Bolt, Breaker }

pub enum ParticipantTarget {
    Bump(BumpTarget),
    Impact(ImpactTarget),
    Death(DeathTarget),
    BoltLost(BoltLostTarget),
}
```

Resolution happens in `walking/on/system.rs` via `resolve_participant`, which pattern-matches `(ParticipantTarget, TriggerContext)` and pulls the `Entity` out. Resolution returns `Option<Entity>` — a `Death(Killer)` against an environmental death resolves to `None` and the `On` is silently skipped.

## TriggerContext

Carries the entities involved in a trigger event so `On` nodes can resolve `ParticipantTarget` values during walking. Bridge systems build a `TriggerContext` from their source message before calling `walk_bound_effects` / `walk_staged_effects`.

```rust
pub enum TriggerContext {
    Bump     { bolt: Option<Entity>, breaker: Entity },
    Impact   { impactor: Entity,     impactee: Entity },
    Death    { victim: Entity,       killer: Option<Entity> },
    BoltLost { bolt: Entity,         breaker: Entity },
    None,
}
```

`Bump.bolt` is optional because `BumpWhiffOccurred` and `NoBumpOccurred` fire without a participating bolt. `Death.killer` is optional because environmental deaths have no killer. `None` is used by triggers with no participants (`NodeStartOccurred`, `NodeEndOccurred`, `NodeTimerThresholdOccurred`, `TimeExpires`).

There is no `depth` field. Recursion limiting, if needed, is handled by the walker, not the context.

## EffectSourceChip

Marker component placed on spawned effect entities (shockwave rings, pulse emitters, gravity wells, etc.) so that downstream damage-application systems can attribute damage back to the chip that caused it. Lives in `effect_v3/components/effect_source_chip.rs`.

```rust
#[derive(Component, Debug, Clone)]
pub struct EffectSourceChip(pub Option<String>);

impl EffectSourceChip {
    pub fn from_source(source: &str) -> Self {
        Self((!source.is_empty()).then(|| source.to_owned()))
    }
}
```

Empty source strings map to `None` (effect not chip-sourced — e.g. cascade effects). Non-empty sources map to `Some(name)`.

## Fireable and Reversible traits

Live in `effect_v3/traits/`. Every config struct in `EffectType` implements `Fireable`; every config struct in `ReversibleEffectType` additionally implements `Reversible`.

```rust
pub trait Fireable {
    fn fire(&self, entity: Entity, source: &str, world: &mut World);

    fn register(_app: &mut App) {}
}

pub trait Reversible: Fireable {
    fn reverse(&self, entity: Entity, source: &str, world: &mut World);

    fn reverse_all_by_source(&self, entity: Entity, source: &str, world: &mut World) {
        self.reverse(entity, source, world);
    }
}
```

`Fireable::register(app)` is the per-effect plugin hook. The default is a no-op. Effects with tick systems, cleanup systems, or reset systems override it. `EffectV3Plugin::build` calls `XxxConfig::register(&mut app)` for all 30 configs unconditionally so that "I added a system but forgot to wire it" never silently drops a system.

`reverse_all_by_source` is the override point for stack-based passives (`SpeedBoostConfig`, `PiercingConfig`, etc.) — the default reverses one instance, but stack effects override it to remove every entry whose source matches via `EffectStack::retain_by_source`. Singleton effects (`Shield`, `Pulse`, etc.) keep the default because there is at most one active instance.

## Dispatch functions

`fire_dispatch` and `reverse_dispatch` are free functions in `effect_v3/dispatch/` that match on the enum variant and call the corresponding config's trait method.

```rust
// dispatch/fire_dispatch.rs
pub fn fire_dispatch(effect: &EffectType, entity: Entity, source: &str, world: &mut World) {
    match effect {
        EffectType::SpeedBoost(config) => config.fire(entity, source, world),
        EffectType::SizeBoost(config)  => config.fire(entity, source, world),
        // ... one arm per variant
    }
}

// dispatch/reverse_dispatch/system.rs
pub fn reverse_dispatch(effect: &ReversibleEffectType, entity: Entity, source: &str, world: &mut World) {
    match effect {
        ReversibleEffectType::SpeedBoost(config) => config.reverse(entity, source, world),
        // ... one arm per reversible variant
    }
}

pub fn fire_reversible_dispatch(effect: &ReversibleEffectType, ...) { /* mirrors fire_dispatch */ }

pub fn reverse_all_by_source_dispatch(effect: &ReversibleEffectType, ...) {
    // calls config.reverse_all_by_source — for stack-based passives
}
```

These are the only places the enum-to-trait jump happens. The walker queues `FireEffectCommand` / `ReverseEffectCommand` (defined in `commands/`); those commands call `fire_dispatch` / `reverse_dispatch` from inside `Command::apply` where `&mut World` is available.

## Why no `Effect` trait dispatch instead of an enum

Trait-object dispatch would require each effect to be a separate boxed type. The enum + match pattern gives:

- **Compile-time exhaustiveness** — the `match` in `fire_dispatch` is exhaustive; adding a variant forces you to add the arm.
- **No allocation** — `EffectType` is a stack-resident value, no `Box<dyn Effect>` per fire.
- **Hash + Eq** — enum variants compare structurally; trait objects don't.
- **Serde for free** — RON serialization round-trips through the enum without custom adapters.

The `Fireable` trait still exists, but it's the per-config implementation contract, not the dispatch layer. The enum is the dispatch layer.
