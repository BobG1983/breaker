# Effect Builder Design

## Architecture: Single Path, Round-Trip

```
     Load:  RON file → deserialize → RawEffectTree → builder methods → Result<ValidEffect, EffectError>
     Code:  ──────────────────────────────────────→ builder methods → Result<ValidEffect, EffectError>
     Save:  ValidEffect → .to_raw() → RawEffectTree → serialize → RON file
```

One builder for everything. RON deserializes into permissive `Raw` structs (derive both `Serialize` + `Deserialize`), then a loader walks the raw tree and calls the builder programmatically. The builder returns `Result` at each step. Content tooling calls the same builder directly with compile-time enforcement.

## Typestate Machine

```
┌─────────────────────────────────────────────────────┐
│ Entry Points                                         │
│                                                       │
│ Effect::when(event)       → TriggerChain<AnyFire>    │
│ Effect::during(condition) → DuringContext             │
│ Effect::until(trigger)    → UntilContext              │
│ Effect::spawned(type)     → SpawnedContext            │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│ TriggerChain<FireConstraint>                         │
│                                                       │
│ .when(event) → TriggerChain<FireConstraint>  (nest)  │
│ .on(target)  → TargetContext<FireConstraint>          │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│ DuringContext (state-scoped: fires on start,         │
│                reverses on end)                       │
│                                                       │
│ .when(event) → TriggerChain<AnyFire>     (nested When│
│                — reversal removes listener, so inner  │
│                can be anything)                       │
│ .on(target)  → TargetContext<ReversibleOnly>  (direct │
│                fire — must be reversible)              │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│ UntilContext (event-scoped: fires immediately,       │
│               reverses when trigger fires)            │
│                                                       │
│ .when(event) → TriggerChain<AnyFire>     (nested When│
│                — same relaxation as During)           │
│ .on(target)  → TargetContext<ReversibleOnly>  (direct │
│                fire — must be reversible)              │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│ TargetContext<AnyFire>                               │
│ .fire(any_effect) → ValidEffect                      │
│ .transfer(inner_tree) → ValidEffect                  │
│                                                       │
│ TargetContext<ReversibleOnly>                         │
│ .fire(reversible_effect) → ValidEffect               │
│ .fire(non_reversible)    → COMPILE ERROR             │
│ .transfer(inner_tree) → ValidEffect                  │
│                                                       │
│ (transfer is always allowed — it stamps BoundEffects, │
│  which is inherently reversible via removal)          │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│ SpawnedContext                                        │
│ .fire(any_effect) → ValidEffect  (implicit target)   │
└─────────────────────────────────────────────────────┘
```

## Rust Types

```rust
// ── Marker types for fire constraint ──
struct AnyFire;
struct ReversibleOnly;

// ── Traits ──
trait Effect {
    fn fire(&self, entity: Entity, source_chip: &str, context: TriggerContext, world: &mut World);
}

trait Reversible: Effect {
    fn reverse(&self, entity: Entity, source_chip: &str, world: &mut World);
}

// ── Entry points ──
impl EffectBuilder {
    fn when(event: impl Into<Trigger>) -> TriggerChain<AnyFire>;
    fn during(condition: impl Into<Condition>) -> DuringContext;
    fn until(trigger: impl Into<Trigger>) -> UntilContext;
    fn spawned(entity_type: EntityType) -> SpawnedContext;
}

// ── TriggerChain ──
struct TriggerChain<C> {
    triggers: Vec<Trigger>,
    _constraint: PhantomData<C>,
}

impl<C> TriggerChain<C> {
    fn when(self, event: impl Into<Trigger>) -> TriggerChain<C>;
    fn on(self, target: impl Into<Target>) -> TargetContext<C>;
}

// ── DuringContext ──
struct DuringContext { condition: Condition }

impl DuringContext {
    fn when(self, event: impl Into<Trigger>) -> DuringTriggerChain; // relaxes to AnyFire
    fn on(self, target: impl Into<Target>) -> TargetContext<ReversibleOnly>; // direct = reversible
}

struct DuringTriggerChain { condition: Condition, triggers: Vec<Trigger> }

impl DuringTriggerChain {
    fn when(self, event: impl Into<Trigger>) -> DuringTriggerChain;
    fn on(self, target: impl Into<Target>) -> TargetContext<AnyFire>; // nested When = any
}

// ── UntilContext (same shape as DuringContext, different semantics) ──
struct UntilContext { trigger: Trigger }

impl UntilContext {
    fn when(self, event: impl Into<Trigger>) -> UntilTriggerChain; // relaxes to AnyFire
    fn on(self, target: impl Into<Target>) -> TargetContext<ReversibleOnly>; // direct = reversible
}

struct UntilTriggerChain { until_trigger: Trigger, triggers: Vec<Trigger> }

impl UntilTriggerChain {
    fn when(self, event: impl Into<Trigger>) -> UntilTriggerChain;
    fn on(self, target: impl Into<Target>) -> TargetContext<AnyFire>; // nested When = any
}

// ── TargetContext ──
impl TargetContext<AnyFire> {
    fn fire(self, effect: impl Effect) -> ValidEffect;
    fn transfer(self, inner: ValidEffect) -> ValidEffect;
    fn transfer_to(self, target: impl Into<Target>, inner: ValidEffect) -> ValidEffect;
}

impl TargetContext<ReversibleOnly> {
    fn fire(self, effect: impl Reversible) -> ValidEffect; // compile error if !Reversible
    fn transfer(self, inner: ValidEffect) -> ValidEffect;
    fn transfer_to(self, target: impl Into<Target>, inner: ValidEffect) -> ValidEffect;
}

// ── SpawnedContext ──
impl SpawnedContext {
    fn fire(self, effect: impl Effect) -> ValidEffect;
}
```

## Validated Type Tree (what the builder produces)

The builder produces `ValidEffect` — a separate type tree from the raw RON types. Structural enforcement: `During`/`Until` can only contain reversible effects in direct `Fire` position. Participant targets are per-trigger enums.

```rust
// ── Shared leaf enums (same for Raw and Valid) ──

enum Trigger {
    PerfectBumped, EarlyBumped, LateBumped, Bumped,
    Impacted(ImpactTarget), Died, Killed(KillTarget),
    PerfectBumpOccurred, EarlyBumpOccurred, LateBumpOccurred, BumpOccurred,
    BumpWhiffOccurred, NoBumpOccurred, ImpactOccurred(ImpactTarget),
    DeathOccurred(DeathTarget), BoltLostOccurred,
    NodeStartOccurred, NodeEndOccurred, NodeTimerThresholdOccurred(f32),
    TimeExpires(f32),
}

enum Condition { NodeActive, NodePlaying }
enum EntityType { Bolt, Cell, Wall, Breaker }
enum ImpactTarget { Cell, Bolt, Wall, Breaker }
enum KillTarget { Cell, Bolt, Wall, Breaker, Any }
enum DeathTarget { Cell, Bolt, Wall, Breaker, Any }

// ── Per-trigger participant enums ──

enum PerfectBumpedTarget { Bolt, Breaker }
enum EarlyBumpedTarget { Bolt, Breaker }
enum LateBumpedTarget { Bolt, Breaker }
enum BumpedTarget { Bolt, Breaker }
enum ImpactedTarget { Impactor, Target }
enum DiedTarget { Victim, Killer }
enum KilledTarget { Killer, Victim }
enum BumpOccurredTarget { Bolt, Breaker }
enum ImpactOccurredTarget { Bolt, Cell, Wall, Breaker }  // depends on ImpactTarget
enum DeathOccurredTarget { Entity, Killer }
enum BoltLostOccurredTarget { Bolt, Breaker }
// NodeStartOccurred, NodeEndOccurred, etc. — no participants

// ── ValidTarget: all possible targets, structurally typed ──

enum ValidTarget {
    // Entity reference
    This,

    // Aggregate targets
    EveryBolt, ActiveBolts, PrimaryBolts, ExtraBolts,
    EveryCell, ActiveCells,
    EveryWall, ActiveWalls,
    EveryBreaker, ActiveBreakers,

    // Per-trigger participants (structurally enforced)
    PerfectBumped(PerfectBumpedTarget),
    EarlyBumped(EarlyBumpedTarget),
    LateBumped(LateBumpedTarget),
    Bumped(BumpedTarget),
    Impacted(ImpactedTarget),
    Died(DiedTarget),
    Killed(KilledTarget),
    BumpOccurred(BumpOccurredTarget),
    ImpactOccurred(ImpactOccurredTarget),
    DeathOccurred(DeathOccurredTarget),
    BoltLostOccurred(BoltLostOccurredTarget),
}

// ── Effect types: full set + reversible subset ──

enum EffectType {
    SpeedBoost(f32), SizeBoost(f32), DamageBoost(f32),
    BumpForce(f32), QuickStop(f32), FlashStep,
    Piercing(u32), Attraction(AttractionConfig), RampingDamage(f32),
    Anchor(AnchorConfig), Vulnerable(f32), Pulse(PulseConfig),
    Shield(ShieldConfig), SecondWind,
    Shockwave(ShockwaveConfig), Explode(ExplodeConfig),
    ChainLightning(ChainLightningConfig), PiercingBeam(PiercingBeamConfig),
    SpawnBolts(SpawnBoltsConfig), SpawnPhantom(SpawnPhantomConfig),
    ChainBolt(ChainBoltConfig), MirrorProtocol(MirrorConfig),
    TetherBeam(TetherBeamConfig),
    LoseLife, TimePenalty(f32), Die,
    CircuitBreaker(CircuitBreakerConfig), EntropyEngine(EntropyConfig),
    RandomEffect,
}

enum ReversibleEffectType {
    SpeedBoost(f32), SizeBoost(f32), DamageBoost(f32),
    BumpForce(f32), QuickStop(f32), FlashStep,
    Piercing(u32), Attraction(AttractionConfig), RampingDamage(f32),
    Anchor(AnchorConfig), Vulnerable(f32), Pulse(PulseConfig),
    Shield(ShieldConfig), SecondWind,
}

// ── Validated tree structure ──

enum ValidEffect {
    When(Trigger, ValidInner),
    During(Condition, ValidScopedInner),
    Until(Trigger, ValidScopedInner),
    Spawned(EntityType, ValidTerminal),
}

enum ValidInner {
    When(Trigger, Box<ValidInner>),        // nested triggers
    On(ValidTarget, ValidTerminal),         // target + terminal
}

enum ValidScopedInner {
    When(Trigger, Box<ValidInner>),        // nested When → relaxed (any effect OK)
    On(ValidTarget, ValidScopedTerminal),  // direct → reversible only
}

enum ValidTerminal {
    Fire(EffectType),                       // any effect
    Transfer(Box<ValidEffect>),             // stamp inner tree
}

enum ValidScopedTerminal {
    Fire(ReversibleEffectType),             // reversible only
    Transfer(Box<ValidEffect>),             // transfer always OK
}
```

## Raw Types (RON schema — permissive, for serde)

```rust
#[derive(Serialize, Deserialize)]
enum RawEffect {
    When(Trigger, Box<RawInner>),
    During(Condition, Box<RawInner>),
    Until(Trigger, Box<RawInner>),
    Spawned(EntityType, Box<RawTerminal>),
}

#[derive(Serialize, Deserialize)]
enum RawInner {
    When(Trigger, Box<RawInner>),
    On(RawTarget, Box<RawTerminal>),
}

#[derive(Serialize, Deserialize)]
enum RawTerminal {
    Fire(EffectType),                       // any effect — validation checks reversibility
    Transfer(Box<RawEffect>),
}

#[derive(Serialize, Deserialize)]
enum RawTarget {
    This,
    EveryBolt, ActiveBolts, PrimaryBolts, ExtraBolts,
    EveryCell, ActiveCells,
    EveryWall, ActiveWalls,
    EveryBreaker, ActiveBreakers,
    // Participants — permissive, validated by loader
    Bolt, Breaker, Cell, Wall,
    Impactor, Target, Victim, Killer, Entity,
}
```

Raw uses flat participant names (`Bolt`, `Victim`, etc.) — the loader validates they match the trigger context and maps to the per-trigger `ValidTarget` variant.

## RON → Valid (loader)

```rust
fn load_effect(raw: &RawEffect) -> Result<ValidEffect, EffectError> {
    match raw {
        RawEffect::When(trigger, inner) => {
            let valid_inner = load_inner(trigger, inner)?;
            Ok(ValidEffect::When(*trigger, valid_inner))
        }
        RawEffect::During(condition, inner) => {
            let valid_inner = load_scoped_inner(inner)?;
            Ok(ValidEffect::During(*condition, valid_inner))
        }
        RawEffect::Until(trigger, inner) => {
            let valid_inner = load_scoped_inner(inner)?;
            Ok(ValidEffect::Until(*trigger, valid_inner))
        }
        RawEffect::Spawned(entity_type, terminal) => {
            let valid_term = load_terminal(terminal)?;
            Ok(ValidEffect::Spawned(*entity_type, valid_term))
        }
    }
}

fn load_inner(trigger_ctx: &Trigger, raw: &RawInner) -> Result<ValidInner, EffectError> {
    match raw {
        RawInner::When(trigger, next) => {
            let inner = load_inner(trigger, next)?;
            Ok(ValidInner::When(*trigger, Box::new(inner)))
        }
        RawInner::On(raw_target, terminal) => {
            let target = validate_target(trigger_ctx, raw_target)?; // checks participant validity
            let term = load_terminal(terminal)?;
            Ok(ValidInner::On(target, term))
        }
    }
}

fn load_scoped_inner(raw: &RawInner) -> Result<ValidScopedInner, EffectError> {
    match raw {
        RawInner::When(trigger, next) => {
            // Nested When → relaxed (any effect OK)
            let inner = load_inner(trigger, next)?;
            Ok(ValidScopedInner::When(*trigger, Box::new(inner)))
        }
        RawInner::On(raw_target, terminal) => {
            // Direct → must be reversible
            let target = validate_target_no_context(raw_target)?;
            let term = load_scoped_terminal(terminal)?;
            Ok(ValidScopedInner::On(target, term))
        }
    }
}

fn load_scoped_terminal(raw: &RawTerminal) -> Result<ValidScopedTerminal, EffectError> {
    match raw {
        RawTerminal::Fire(effect) => {
            let reversible = to_reversible(effect)?; // Err if not reversible
            Ok(ValidScopedTerminal::Fire(reversible))
        }
        RawTerminal::Transfer(inner) => {
            let valid = load_effect(inner)?;
            Ok(ValidScopedTerminal::Transfer(Box::new(valid)))
        }
    }
}
```

## Valid → Raw (round-trip for serialization)

```rust
impl ValidEffect {
    fn to_raw(&self) -> RawEffect {
        match self {
            ValidEffect::When(t, inner) => RawEffect::When(*t, Box::new(inner.to_raw())),
            ValidEffect::During(c, inner) => RawEffect::During(*c, Box::new(inner.to_raw_inner())),
            ValidEffect::Until(t, inner) => RawEffect::Until(*t, Box::new(inner.to_raw_inner())),
            ValidEffect::Spawned(e, term) => RawEffect::Spawned(*e, Box::new(term.to_raw())),
        }
    }
}

// ValidTarget → RawTarget: flatten per-trigger enums back to flat names
impl ValidTarget {
    fn to_raw(&self) -> RawTarget {
        match self {
            ValidTarget::This => RawTarget::This,
            ValidTarget::EveryBolt => RawTarget::EveryBolt,
            ValidTarget::Died(DiedTarget::Victim) => RawTarget::Victim,
            ValidTarget::Died(DiedTarget::Killer) => RawTarget::Killer,
            ValidTarget::PerfectBumped(PerfectBumpedTarget::Bolt) => RawTarget::Bolt,
            // ... etc
        }
    }
}
```

## Builder Usage Examples

```rust
// Simple: when bumped, speed boost on self
EffectBuilder::when(PerfectBumped)
    .on(This)
    .fire(SpeedBoost { multiplier: 1.5 })?;

// Scoped: speed boost for the whole node, reversed at teardown
EffectBuilder::during(NodeRunning)
    .on(EveryBolt)
    .fire(SpeedBoost { multiplier: 1.3 })?;

// Won't compile: Explode is not Reversible
// EffectBuilder::during(NodeRunning)
//     .on(EveryBolt)
//     .fire(Explode { range: 50.0, damage: 10.0 })

// During + nested When: non-reversible is OK (During reverses the listener)
EffectBuilder::during(NodeRunning)
    .when(PerfectBumped)
    .on(This)
    .fire(Explode { range: 50.0, damage: 10.0 })?;

// Until: speed boost until I die (fires immediately, reverses on death)
EffectBuilder::until(Died)
    .on(This)
    .fire(SpeedBoost { multiplier: 1.5 })?;

// Until: shield until a bolt is lost (global trigger as reversal)
EffectBuilder::until(BoltLostOccurred)
    .on(This)
    .fire(Shield { charges: 1 })?;

// Until + nested When: non-reversible OK (same relaxation as During)
EffectBuilder::until(Died)
    .when(PerfectBumped)
    .on(This)
    .fire(Explode { range: 50.0, damage: 10.0 })?;

// On bolt spawn, apply piercing
EffectBuilder::spawned(Bolt)
    .fire(Piercing { count: 3 })?;

// Nested triggers: perfect bump then cell impact
EffectBuilder::when(PerfectBumped)
    .when(Impacted(Cell))
    .on(This)
    .fire(ChainBolt { tether_distance: 120.0 })?;

// Transfer: powder keg — "when I hit a cell, stamp 'when you die, explode' on it"
EffectBuilder::when(Impacted(Cell))
    .on(Impacted::Target)
    .transfer(
        EffectBuilder::when(Died)
            .on(This)
            .fire(Explode { range: 48.0, damage: 10.0 })?
    )?;

// Kill reward: "when I kill a cell, boost my speed"
EffectBuilder::when(Killed(Cell))
    .on(This)
    .fire(SpeedBoost { multiplier: 1.3 })?;

// Named participants: "when a bump occurs, do something to the breaker"
EffectBuilder::when(PerfectBumped)
    .on(PerfectBumped::Breaker)
    .fire(FlashStep)?;
```

## RON Format

```ron
// Same vocabulary as builder
(
    name: "Aegis Protocol",
    effects: [
        // Scoped: every bolt gets speed boost for the node
        During(NodeRunning, On(EveryBolt, Fire(SpeedBoost(1.3)))),

        // Primary bolts: shockwave on perfect bump
        When(PerfectBumped, On(PrimaryBolts, Fire(Shockwave(
            base_range: 96.0, range_per_level: 0.0, stacks: 1, speed: 400.0,
        )))),

        // Kill reward
        When(Killed(Cell), On(This, Fire(SpeedBoost(1.3)))),

        // Transfer: stamp "when you die, explode" on impacted cell
        When(Impacted(Cell), On(Impacted::Target, Transfer(
            When(Died, On(This, Fire(Explode(range: 48.0, damage: 10.0))))
        ))),

        // Event-scoped: speed boost until I die
        Until(Died, On(This, Fire(SpeedBoost(1.5)))),

        // On bolt spawn
        Spawned(Bolt, Fire(Piercing)),

        // Global: when any death occurs
        When(DeathOccurred(Cell), On(This, Fire(Shockwave(
            base_range: 32.0, range_per_level: 0.0, stacks: 1, speed: 300.0,
        )))),
    ],
)
```

## RON Loader (walks raw tree, calls builder)

```rust
fn load_effect(raw: &RawEffect) -> Result<ValidEffect, EffectError> {
    match raw {
        RawEffect::When(event, inner) => {
            let chain = EffectBuilder::when(event.try_into()?);
            load_inner(chain, inner)
        }
        RawEffect::During(condition, inner) => {
            let ctx = EffectBuilder::during(condition.try_into()?);
            load_during_inner(ctx, inner)
        }
        RawEffect::Spawned(entity_type, inner) => {
            let ctx = EffectBuilder::spawned(entity_type.try_into()?);
            load_fire(ctx, inner)
        }
    }
}
```

## Reversible Marker Trait

```rust
trait Reversible: Effect {
    fn reverse(&self, entity: Entity, source_chip: &str, world: &mut World);
}

// Implements Reversible:
impl Reversible for SpeedBoost { ... }
impl Reversible for SizeBoost { ... }
impl Reversible for DamageBoost { ... }
impl Reversible for BumpForce { ... }
impl Reversible for QuickStop { ... }
impl Reversible for FlashStep { ... }
impl Reversible for Piercing { ... }
impl Reversible for Attraction { ... }
impl Reversible for RampingDamage { ... }
impl Reversible for Anchor { ... }
impl Reversible for Vulnerable { ... }
impl Reversible for Pulse { ... }
impl Reversible for Shield { ... }
impl Reversible for SecondWind { ... }

// Does NOT implement Reversible:
// Shockwave, Explode, ChainLightning, PiercingBeam,
// SpawnBolts, SpawnPhantom, ChainBolt, MirrorProtocol,
// LoseLife, TimePenalty, Die, CircuitBreaker,
// EntropyEngine, RandomEffect, TetherBeam (standard mode)
```

## Validation Rules

| Rule | Example violation | Caught by |
|------|-------------------|-----------|
| `During` directly wrapping `Fire(X)` — X must be `Reversible` | `During(NodeRunning, On(This, Fire(Explode)))` | Compile error (builder) / `Err(NonReversibleInDuring)` (RON loader) |
| `Spawned` has implicit target — no `On()` | `Spawned(Bolt, On(Breaker, Fire(...)))` | Compile error (no `.on()` on SpawnedContext) / `Err(SpawnedCannotHaveExplicitTarget)` |
| `Spawned(Bolt)` + `Fire(SpawnBolts)` = direct loop | | `Err(SpawnLoop)` (RON loader) / runtime recursion depth limit |
| Indirect spawn loops | A spawns B spawns A | Runtime recursion depth limit (safety net) |
| Named participant not available on trigger | `When(NodeStartOccurred, On(NodeStartOccurred::Bolt, ...))` — NodeStart has no participants | Compile error (associated type) / `Err(InvalidParticipant)` |
