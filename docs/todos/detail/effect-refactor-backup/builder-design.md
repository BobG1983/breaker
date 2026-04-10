# Effect Builder Design

## Architecture: Single Path, Round-Trip

```
     Load:  RON file → deserialize → RawDef → builder methods → Result<ValidDef, EffectError>
     Code:  ──────────────────────────────→ builder methods → Result<ValidDef, EffectError>
     Save:  ValidDef → .to_raw() → RawDef → serialize → RON file
```

One builder for everything. RON deserializes into permissive `Raw` structs (derive both `Serialize` + `Deserialize`), then a loader walks the raw tree and calls the builder programmatically. The builder returns `Result` at each step. Content tooling calls the same builder directly with compile-time enforcement.

Note: `ValidDef` was previously called `ValidEffect`, and `RawDef` was previously called `RawEffectTree`. Renamed for clarity — "Def" better describes the definition-level wrapper.

## Typestate Machine

Two entry points: `EffectDef` for definition-level entries (Stamp required), `EffectTree` for inner trees and Route payloads.

```
┌─────────────────────────────────────────────────────┐
│ Definition Entry Point (Stamp required at root)      │
│                                                       │
│ EffectDef::stamp(target) → StampContext              │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│ StampContext (sets This for the subtree)              │
│                                                       │
│ .fire(effect)             → ValidDef                 │
│ .sequence(trees)          → ValidDef                 │
│ .when(trigger)            → TriggerChain<AnyFire>    │
│ .once(trigger)            → TriggerChain<AnyFire>    │
│ .during(condition)        → DuringContext             │
│ .until(trigger)           → UntilContext              │
│ .spawned(type)            → SpawnedContext            │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│ Inner Tree Entry Point (for Route payloads)          │
│                                                       │
│ EffectTree::when(trigger)     → TriggerChain<AnyFire>│
│ EffectTree::once(trigger)     → TriggerChain<AnyFire>│
│ EffectTree::during(condition) → DuringContext         │
│ EffectTree::until(trigger)    → UntilContext          │
│ EffectTree::fire(effect)      → ValidTree            │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│ TriggerChain<FireConstraint>                         │
│                                                       │
│ .when(event)   → TriggerChain<FireConstraint> (nest) │
│ .fire(effect)  → ValidTree/ValidDef (targets This)   │
│ .sequence()    → ValidTree/ValidDef (Sequence node)  │
│ .on(target)    → TargetContext<FireConstraint>        │
│                  (only for non-This targets)          │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│ DuringContext (state-scoped: fires on start,         │
│                reverses on end)                       │
│                                                       │
│ .when(event) → TriggerChain<AnyFire>     (nested When│
│                — reversal removes listener, so inner  │
│                can be anything)                       │
│ .fire(reversible_effect) → ValidTree/ValidDef        │
│                (direct fire — must be reversible,     │
│                 targets This implicitly)              │
│ .on(target)  → TargetContext<ReversibleOnly>          │
│                (only for non-This targets)            │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│ UntilContext (event-scoped: fires immediately,       │
│               reverses when trigger fires)            │
│                                                       │
│ .when(event) → TriggerChain<AnyFire>     (nested When│
│                — same relaxation as During)           │
│ .fire(reversible_effect) → ValidTree/ValidDef        │
│                (direct fire — must be reversible,     │
│                 targets This implicitly)              │
│ .on(target)  → TargetContext<ReversibleOnly>          │
│                (only for non-This targets)            │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│ TargetContext<AnyFire>                               │
│ .fire(any_effect) → ValidTree/ValidDef               │
│ .route(inner_tree) → ValidTree/ValidDef              │
│                                                       │
│ TargetContext<ReversibleOnly>                         │
│ .fire(reversible_effect) → ValidTree/ValidDef        │
│ .fire(non_reversible)    → COMPILE ERROR             │
│ .route(inner_tree) → ValidTree/ValidDef              │
│                                                       │
│ (route is always allowed — it adds to StagedEffects,  │
│  which is inherently one-shot)                        │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│ SpawnedContext                                        │
│ .fire(any_effect) → ValidTree/ValidDef (implicit tgt)│
│ .stamp(tree)      → ValidTree/ValidDef               │
│ .route(tree)      → ValidTree/ValidDef               │
└─────────────────────────────────────────────────────┘
```

### Key rules
- `Fire` always targets `This` (the entity Stamp stamps to, or the entity BoundEffects lives on inside a Stamp/Route payload)
- `On` is ONLY for non-This targets: trigger participants (`BumpTarget::Bolt`) — redirects Fire/Stamp/Route to that entity
- `On(This, ...)` never appears — it's always just `Fire(...)` directly
- `Stamp` is required at definition root — you cannot have a bare `When(...)` in an `effects: []` list
- `Stamp` (terminal) = permanent add to target's BoundEffects at runtime
- `Route` (terminal) = one-shot add to target's StagedEffects at runtime

## Rust Types

```rust
// ── Marker types for fire constraint ──
struct AnyFire;
struct ReversibleOnly;

// ── Traits ──
trait Effect {
    fn fire(&self, entity: Entity, source_chip: &str, context: &TriggerContext, world: &mut World);
}

trait Reversible: Effect {
    fn reverse(&self, entity: Entity, source_chip: &str, world: &mut World);
}

// ── Definition entry point (Stamp required) ──
impl EffectDef {
    fn stamp(target: impl Into<Target>) -> StampContext;
}

// ── StampContext (sets This for the subtree) ──
struct StampContext { target: Target }

impl StampContext {
    fn fire(self, effect: impl Effect) -> ValidDef;                     // passive effect
    fn sequence(self, trees: Vec<ValidTree>) -> ValidDef;               // multiple children
    fn when(self, trigger: impl Into<Trigger>) -> TriggerChain<AnyFire>;
    fn once(self, trigger: impl Into<Trigger>) -> TriggerChain<AnyFire>;
    fn during(self, condition: impl Into<Condition>) -> DuringContext;
    fn until(self, trigger: impl Into<Trigger>) -> UntilContext;
    fn spawned(self, entity_kind: EntityKind) -> SpawnedContext;
}

// ── Inner tree entry point (for Route payloads) ──
impl EffectTree {
    fn when(trigger: impl Into<Trigger>) -> TriggerChain<AnyFire>;
    fn once(trigger: impl Into<Trigger>) -> TriggerChain<AnyFire>;
    fn during(condition: impl Into<Condition>) -> DuringContext;
    fn until(trigger: impl Into<Trigger>) -> UntilContext;
    fn fire(effect: impl Effect) -> ValidTree;                          // direct fire on This
}

// ── TriggerChain ──
struct TriggerChain<C> {
    triggers: Vec<Trigger>,
    _constraint: PhantomData<C>,
}

impl<C> TriggerChain<C> {
    fn when(self, event: impl Into<Trigger>) -> TriggerChain<C>;       // nest triggers
    fn fire(self, effect: impl Effect) -> ValidDef;                     // fire on This (implicit)
    fn sequence(self, trees: Vec<ValidTree>) -> ValidDef;               // Sequence node
    fn on(self, target: impl Into<ParticipantTarget>) -> TargetContext<C>; // non-This target only
}

// ── DuringContext ──
struct DuringContext { condition: Condition }

impl DuringContext {
    fn when(self, event: impl Into<Trigger>) -> DuringTriggerChain;    // relaxes to AnyFire
    fn fire(self, effect: impl Reversible) -> ValidDef;                 // direct = reversible, targets This
    fn on(self, target: impl Into<ParticipantTarget>) -> TargetContext<ReversibleOnly>;
}

struct DuringTriggerChain { condition: Condition, triggers: Vec<Trigger> }

impl DuringTriggerChain {
    fn when(self, event: impl Into<Trigger>) -> DuringTriggerChain;
    fn fire(self, effect: impl Effect) -> ValidDef;                     // nested When = any
    fn on(self, target: impl Into<ParticipantTarget>) -> TargetContext<AnyFire>;
}

// ── UntilContext (same shape as DuringContext, different semantics) ──
struct UntilContext { trigger: Trigger }

impl UntilContext {
    fn when(self, event: impl Into<Trigger>) -> UntilTriggerChain;     // relaxes to AnyFire
    fn fire(self, effect: impl Reversible) -> ValidDef;                 // direct = reversible, targets This
    fn on(self, target: impl Into<ParticipantTarget>) -> TargetContext<ReversibleOnly>;
}

struct UntilTriggerChain { until_trigger: Trigger, triggers: Vec<Trigger> }

impl UntilTriggerChain {
    fn when(self, event: impl Into<Trigger>) -> UntilTriggerChain;
    fn fire(self, effect: impl Effect) -> ValidDef;                     // nested When = any
    fn on(self, target: impl Into<ParticipantTarget>) -> TargetContext<AnyFire>;
}

// ── TargetContext (only reached via .on() for non-This targets) ──
impl TargetContext<AnyFire> {
    fn fire(self, effect: impl Effect) -> ValidDef;
    fn stamp(self, inner: ValidTree) -> ValidDef;                       // permanent → BoundEffects
    fn route(self, inner: ValidTree) -> ValidDef;                       // one-shot → StagedEffects
}

impl TargetContext<ReversibleOnly> {
    fn fire(self, effect: impl Reversible) -> ValidDef;                 // compile error if !Reversible
    fn stamp(self, inner: ValidTree) -> ValidDef;                       // permanent → BoundEffects
    fn route(self, inner: ValidTree) -> ValidDef;                       // one-shot → StagedEffects
}

// ── SpawnedContext ──
impl SpawnedContext {
    fn fire(self, effect: impl Effect) -> ValidDef;                     // implicit target
    fn stamp(self, inner: ValidTree) -> ValidDef;                       // permanent → BoundEffects
    fn route(self, inner: ValidTree) -> ValidDef;                       // one-shot → StagedEffects
}
```

## Validated Type Tree (what the builder produces)

The builder produces `ValidDef` — a separate type tree from the raw RON types. Structural enforcement: `During`/`Until` can only contain reversible effects in direct `Fire` position. Participant targets are per-trigger enums.

```rust
// ── Shared leaf enums (same for Raw and Valid) ──

enum Trigger {
    PerfectBumped, EarlyBumped, LateBumped, Bumped,
    Impacted(EntityKind), Died, Killed(EntityKind),
    PerfectBumpOccurred, EarlyBumpOccurred, LateBumpOccurred, BumpOccurred,
    BumpWhiffOccurred, NoBumpOccurred, ImpactOccurred(EntityKind),
    DeathOccurred(EntityKind), BoltLostOccurred,
    NodeStartOccurred, NodeEndOccurred, NodeTimerThresholdOccurred(f32),
    TimeExpires(f32),
}

enum Condition { NodeActive, ShieldActive, ComboActive(u32) }
enum EntityKind { Cell, Bolt, Wall, Breaker, Any }   // trigger parameter AND Spawned entity type

// ── Target: definition-time entity routing (no This, no participants) ──

enum Target {
    Bolt, Breaker, Cell, Wall,
    ActiveBolts, EveryBolt, PrimaryBolts, ExtraBolts,
    ActiveCells, EveryCells,
    ActiveWalls, EveryWall,
    ActiveBreakers, EveryBreaker,
}

// ── Shared participant enums (for On() participant redirect) ──
// Grouped by concept. Triggers sharing an enum live in the same folder.
// These are ROLE enums (who in the event), NOT entity-type enums.
// Entity-type filtering on triggers uses EntityKind (above).

enum BumpTarget { Bolt, Breaker }          // triggers/bump/
// Used by: PerfectBumped, EarlyBumped, LateBumped, Bumped,
//          PerfectBumpOccurred, BumpOccurred, BumpWhiffOccurred, NoBumpOccurred

enum ImpactTarget { Impactor, Impactee }   // triggers/impact/
// Used by: Impacted, ImpactOccurred — participant ROLE redirect

enum DeathTarget { Victim, Killer }        // triggers/death/
// Used by: Died, Killed, DeathOccurred — participant ROLE redirect

enum BoltLostTarget { Bolt, Breaker }      // triggers/bolt_lost/
// Used by: BoltLostOccurred

// NodeStartOccurred, NodeEndOccurred, NodeTimerThresholdOccurred — no participants

// ── ParticipantTarget: runtime redirect (On target, non-This only) ──
// Shared enums — multiple triggers map to the same concept.

enum ParticipantTarget {
    Bump(BumpTarget),           // triggers/bump/
    Impact(ImpactTarget),       // triggers/impact/
    Death(DeathTarget),         // triggers/death/
    BoltLost(BoltLostTarget),   // triggers/bolt_lost/
}
// No This variant (Fire targets This implicitly).
// No entity types (Route handles routing at definition level).

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
    RandomEffect(Vec<(f32, Box<EffectType>)>),
}

enum ReversibleEffectType {
    SpeedBoost(f32), SizeBoost(f32), DamageBoost(f32),
    BumpForce(f32), QuickStop(f32), FlashStep,
    Piercing(u32), Attraction(AttractionConfig), RampingDamage(f32),
    Anchor(AnchorConfig), Vulnerable(f32), Pulse(PulseConfig),
    Shield(ShieldConfig), SecondWind,
}

// ── Validated tree structure ──

// Definition-level: Stamp wraps every top-level entry
struct ValidDef {
    target: Target,
    tree: ValidTree,
}

// Inner tree (lives inside Stamp, and inside Stamp/Route payloads)
enum ValidTree {
    Fire(EffectType),                               // fire on This (implicit)
    Sequence(Vec<ValidTree>),                        // multiple children at same level
    When(Trigger, Box<ValidTree>),
    Once(Trigger, Box<ValidTree>),                  // same as When, self-removes after first match
    During(Condition, ValidScopedTree),
    Until(Trigger, ValidScopedTree),
    Spawned(EntityKind, Box<ValidTree>),
    On(ParticipantTarget, ValidTerminal),            // redirect to non-This target
}

enum ValidScopedTree {
    Fire(ReversibleEffectType),                      // direct fire in scoped context — reversible only
    Sequence(Vec<ReversibleEffectType>),             // multiple reversible effects
    When(Trigger, Box<ValidTree>),                   // nested When → relaxed (any effect OK)
    On(ParticipantTarget, ValidScopedTerminal),      // direct to non-This — reversible only
}

enum ValidTerminal {
    Fire(EffectType),                               // any effect, immediate
    Stamp(Box<ValidTree>),                           // permanent → target's BoundEffects
    Route(Box<ValidTree>),                           // one-shot → target's StagedEffects
    Reverse(ReversibleEffectType),                   // internal only — generated by During/Until
}

enum ValidScopedTerminal {
    Fire(ReversibleEffectType),                      // reversible only
    Stamp(Box<ValidTree>),                           // stamp always OK (removable via BoundEffects cleanup)
    Route(Box<ValidTree>),                           // route always OK
}

// Note: Reverse never appears in RawEffect/RON. It's generated internally when
// During/Until desugar their reversal entries into StagedEffects:
//   During(NodeActive, Fire(SpeedBoost))
//   → fires SpeedBoost immediately
//   → stages: When(NodeEndOccurred, Reverse(SpeedBoost))

```

**Derive requirements:** All validated types (`ValidTree`, `ValidDef`, `EffectType`, `ReversibleEffectType`, `Trigger`, `Condition`, participant enums) require `Clone`, `Debug`, `PartialEq`. Config structs additionally require `Deserialize`. Raw types require `Serialize` + `Deserialize`.

## Raw Types (RON schema — permissive, for serde)

```rust
// Definition-level: Stamp wraps every top-level entry
#[derive(Serialize, Deserialize)]
struct RawDef {
    target: RawTarget,
    tree: RawTree,
}

#[derive(Serialize, Deserialize)]
enum RawTarget {
    Bolt, Breaker, Cell, Wall,
    ActiveBolts, EveryBolt, PrimaryBolts, ExtraBolts,
    ActiveCells, EveryCells,
    ActiveWalls, EveryWall,
    ActiveBreakers, EveryBreaker,
}

// Inner tree (lives inside Stamp, and inside Stamp/Route payloads)
#[derive(Serialize, Deserialize)]
enum RawTree {
    Fire(EffectType),                       // fire on This (implicit)
    Sequence(Vec<RawTree>),                 // multiple children at same level
    When(Trigger, Box<RawTree>),
    Once(Trigger, Box<RawTree>),
    During(Condition, Box<RawTree>),
    Until(Trigger, Box<RawTree>),
    Spawned(EntityKind, Box<RawTree>),
    On(RawParticipant, Box<RawTerminal>),   // redirect to non-This target
}

#[derive(Serialize, Deserialize)]
enum RawTerminal {
    Fire(EffectType),                       // any effect — validation checks reversibility
    Stamp(Box<RawTree>),                    // permanent → target's BoundEffects
    Route(Box<RawTree>),                    // one-shot → target's StagedEffects
}

#[derive(Serialize, Deserialize)]
enum RawParticipant {
    // Fully qualified — matches ParticipantTarget exactly
    BumpTarget(BumpTarget),
    ImpactTarget(ImpactTarget),
    DeathTarget(DeathTarget),
    BoltLostTarget(BoltLostTarget),
}
```

RON uses fully qualified participant names: `On(BumpTarget::Bolt, ...)`, `On(ImpactTarget::Impactee, ...)`. RawParticipant matches ParticipantTarget — no flat names, no ambiguity. No `This` in On — Fire targets This implicitly. No entity types in On — Route handles routing at definition level.

## RON → Valid (loader)

```rust
fn load_def(raw: &RawDef) -> Result<ValidDef, EffectError> {
    let target = validate_target(&raw.target)?;
    let tree = load_tree(&raw.tree, None)?; // no trigger context at root
    Ok(ValidDef { target, tree })
}

fn load_tree(raw: &RawTree, trigger_ctx: Option<&Trigger>) -> Result<ValidTree, EffectError> {
    match raw {
        RawTree::Fire(effect) => Ok(ValidTree::Fire(effect.clone())),
        RawTree::When(trigger, inner) => {
            let tree = load_tree(inner, Some(trigger))?;
            Ok(ValidTree::When(*trigger, Box::new(tree)))
        }
        RawTree::Once(trigger, inner) => {
            let tree = load_tree(inner, Some(trigger))?;
            Ok(ValidTree::Once(*trigger, Box::new(tree)))
        }
        RawTree::During(condition, inner) => {
            let scoped = load_scoped_tree(inner, trigger_ctx)?;
            Ok(ValidTree::During(*condition, scoped))
        }
        RawTree::Until(trigger, inner) => {
            let scoped = load_scoped_tree(inner, trigger_ctx)?;
            Ok(ValidTree::Until(*trigger, scoped))
        }
        RawTree::Spawned(entity_type, inner) => {
            let tree = load_tree(inner, None)?;
            Ok(ValidTree::Spawned(*entity_type, Box::new(tree)))
        }
        RawTree::On(participant, terminal) => {
            let target = validate_participant(trigger_ctx, participant)?;
            let term = load_terminal(terminal)?;
            Ok(ValidTree::On(target, term))
        }
        RawTree::Sequence(children) => {
            let trees: Result<Vec<_>, _> = children.iter()
                .map(|c| load_tree(c, trigger_ctx))
                .collect();
            Ok(ValidTree::Sequence(trees?))
        }
    }
}

fn load_scoped_tree(raw: &RawTree, trigger_ctx: Option<&Trigger>) -> Result<ValidScopedTree, EffectError> {
    match raw {
        RawTree::Fire(effect) => {
            let reversible = to_reversible(effect)?; // Err if not reversible
            Ok(ValidScopedTree::Fire(reversible))
        }
        RawTree::When(trigger, inner) => {
            // Nested When → relaxed (any effect OK)
            let tree = load_tree(inner, Some(trigger))?;
            Ok(ValidScopedTree::When(*trigger, Box::new(tree)))
        }
        RawTree::On(participant, terminal) => {
            let target = validate_participant(trigger_ctx, participant)?;
            let term = load_scoped_terminal(terminal)?;
            Ok(ValidScopedTree::On(target, term))
        }
        RawTree::Sequence(children) => {
            let effects: Result<Vec<_>, _> = children.iter()
                .map(|c| match c {
                    RawTree::Fire(effect) => to_reversible(effect),
                    _ => Err(EffectError::InvalidInScopedContext),
                })
                .collect();
            Ok(ValidScopedTree::Sequence(effects?))
        }
        _ => Err(EffectError::InvalidInScopedContext),
    }
}

fn load_terminal(raw: &RawTerminal) -> Result<ValidTerminal, EffectError> {
    match raw {
        RawTerminal::Fire(effect) => Ok(ValidTerminal::Fire(effect.clone())),
        RawTerminal::Stamp(inner) => {
            let tree = load_tree(inner, None)?; // Stamp payload has no trigger context
            Ok(ValidTerminal::Stamp(Box::new(tree)))
        }
        RawTerminal::Route(inner) => {
            let tree = load_tree(inner, None)?; // Route payload has no trigger context
            Ok(ValidTerminal::Route(Box::new(tree)))
        }
    }
}
```

## Valid → Raw (round-trip for serialization)

```rust
impl ValidDef {
    fn to_raw(&self) -> RawDef {
        RawDef {
            target: self.target.to_raw(),
            tree: self.tree.to_raw(),
        }
    }
}

impl ValidTree {
    fn to_raw(&self) -> RawTree {
        match self {
            ValidTree::Fire(e) => RawTree::Fire(e.clone()),
            ValidTree::Sequence(children) => RawTree::Sequence(children.iter().map(|c| c.to_raw()).collect()),
            ValidTree::When(t, inner) => RawTree::When(*t, Box::new(inner.to_raw())),
            ValidTree::Once(t, inner) => RawTree::Once(*t, Box::new(inner.to_raw())),
            ValidTree::During(c, inner) => RawTree::During(*c, Box::new(inner.to_raw_tree())),
            ValidTree::Until(t, inner) => RawTree::Until(*t, Box::new(inner.to_raw_tree())),
            ValidTree::Spawned(k, inner) => RawTree::Spawned(*k, Box::new(inner.to_raw())),
            ValidTree::On(target, term) => RawTree::On(target.to_raw(), Box::new(term.to_raw())),
        }
    }
}

// ParticipantTarget → RawParticipant: direct mapping (same structure)
impl ParticipantTarget {
    fn to_raw(&self) -> RawParticipant {
        match self {
            ParticipantTarget::Bump(t) => RawParticipant::BumpTarget(*t),
            ParticipantTarget::Impact(t) => RawParticipant::ImpactTarget(*t),
            ParticipantTarget::Death(t) => RawParticipant::DeathTarget(*t),
            ParticipantTarget::BoltLost(t) => RawParticipant::BoltLostTarget(*t),
        }
    }
}
```

## Builder Usage Examples

```rust
// Simple passive: damage boost on bolt
EffectDef::stamp(Bolt)
    .fire(DamageBoost { multiplier: 3.0 })?;

// Triggered: when bumped, speed boost (Fire targets This = bolt)
EffectDef::stamp(Bolt)
    .when(PerfectBumped)
    .fire(SpeedBoost { multiplier: 1.5 })?;

// Scoped: speed boost for the whole node, reversed at teardown
EffectDef::stamp(EveryBolt)
    .during(NodeActive)
    .fire(SpeedBoost { multiplier: 1.3 })?;

// Won't compile: Explode is not Reversible
// EffectDef::stamp(Bolt)
//     .during(NodeActive)
//     .fire(Explode { range: 50.0, damage: 10.0 })

// During + nested When: non-reversible is OK (During reverses the listener)
EffectDef::stamp(Bolt)
    .during(NodeActive)
    .when(PerfectBumped)
    .fire(Explode { range: 50.0, damage: 10.0 })?;

// Until: speed boost until I die (fires immediately, reverses on death)
EffectDef::stamp(Bolt)
    .until(Died)
    .fire(SpeedBoost { multiplier: 1.5 })?;

// Until + nested When: non-reversible OK (same relaxation as During)
EffectDef::stamp(Bolt)
    .until(Died)
    .when(PerfectBumped)
    .fire(Explode { range: 50.0, damage: 10.0 })?;

// Nested triggers: perfect bump then cell impact then fire on This
EffectDef::stamp(Bolt)
    .when(PerfectBumped)
    .when(Impacted(Cell))
    .fire(ChainBolt { tether_distance: 120.0 })?;

// Route: powder keg — "when I hit a cell, route 'when you die, explode' onto it"
EffectDef::stamp(Bolt)
    .when(Impacted(Cell))
    .on(ImpactTarget::Impactee)
    .route(
        EffectTree::when(Died)
            .fire(Explode { range: 48.0, damage: 10.0 })?
    )?;

// Kill reward: "when I kill a cell, boost my speed" (Fire targets This = bolt)
EffectDef::stamp(Bolt)
    .when(Killed(Cell))
    .fire(SpeedBoost { multiplier: 1.3 })?;

// Breaker bolt effect: stamp speed boost onto every bolt
EffectDef::stamp(EveryBolt)
    .when(PerfectBumped)
    .fire(SpeedBoost { multiplier: 1.5 })?;

// Mixed-target chip: bolt gets damage, breaker gets penalty
EffectDef::stamp(Bolt)
    .fire(DamageBoost { multiplier: 3.0 })?;
EffectDef::stamp(Breaker)
    .when(BoltLostOccurred)
    .fire(LoseLife)?;

// Named participants: redirect fire to a trigger participant
EffectDef::stamp(Breaker)
    .when(PerfectBumped)
    .on(BumpTarget::Bolt)
    .fire(FlashStep)?;
```

## RON Format

```ron
// Same vocabulary as builder. Route required at root of every entry.
(
    name: "Example Chip",
    effects: [
        // Simple passive: bolt gets damage boost
        Stamp(Bolt, Fire(DamageBoost(3.0))),

        // Scoped: every bolt gets speed boost for the node
        Stamp(EveryBolt, During(NodeActive, Fire(SpeedBoost(1.3)))),

        // Triggered: kill reward (Fire targets This = bolt)
        Stamp(Bolt, When(Killed(Cell), Fire(SpeedBoost(1.3)))),

        // Route (one-shot): "when you die, explode" onto impacted cell
        Stamp(Bolt, When(Impacted(Cell), On(ImpactTarget::Impactee, Route(
            When(Died, Fire(Explode(range: 48.0, damage: 10.0)))
        )))),

        // Stamp (permanent): "always explode on death" onto impacted cell
        Stamp(Bolt, When(Impacted(Cell), On(ImpactTarget::Impactee, Stamp(
            When(Died, Fire(Explode(range: 48.0, damage: 10.0)))
        )))),

        // Event-scoped: speed boost until I die
        Stamp(Bolt, Until(Died, Fire(SpeedBoost(1.5)))),

        // On bolt spawn
        Stamp(Bolt, Spawned(Bolt, Fire(Piercing))),

        // Mixed target: breaker effect in same chip
        Stamp(Breaker, When(BoltLostOccurred, Fire(LoseLife))),

        // Global: when any death occurs, shockwave on This
        Stamp(Bolt, When(DeathOccurred(Cell), Fire(Shockwave(
            base_range: 32.0, range_per_level: 0.0, stacks: 1, speed: 300.0,
        )))),
    ],
)
```

## `This` Semantics

`This` is implicit — it's the entity `Route` routes to. `Fire(effect)` always fires on `This`. You never write `This` in the RON or builder; it's determined by the `Route` target:

- `Stamp(Bolt, ...)` → `This` = the bolt entity
- `Stamp(Breaker, ...)` → `This` = the breaker entity
- `Stamp(EveryBolt, ...)` → `This` = each bolt entity individually
- Inside a `Stamp`/`Route` payload → `This` = the entity the tree was added to

`On` only appears when you need to redirect away from `This` to a trigger participant. For example, `On(ImpactTarget::Impactee, Route(...))` redirects the Route to the impact target instead of This.

## Spawned + Stamp/Route Pattern

`Spawned(EntityKind, ...)` fires on entity add with an implicit target. To add a *scoped* effect on the new entity, use `Stamp` (permanent) or `Route` (one-shot):

```ron
// Every future bolt permanently gets damage boost until it dies
Stamp(Bolt, Spawned(Bolt, Stamp(
    Until(Died, Fire(DamageBoost(multiplier: 1.5)))
)))
```

Builder:
```rust
EffectDef::stamp(Bolt)
    .spawned(Bolt)
    .route(
        EffectTree::until(Died)
            .fire(DamageBoost { multiplier: 1.5 })?
    )?;
```

`Spawned` + `Fire` = fire-and-forget effect on the new entity.
`Spawned` + `Route` = arm an effect tree in the new entity's StagedEffects (one-shot, consumed when triggered).

## Stamp vs Route Semantics

| | Destination | Permanence | When | Re-arms |
|---|---|---|---|---|
| **Stamp** (definition + terminal) | BoundEffects | Permanent — part of the entity's identity | Definition/load time or runtime | Yes — triggers re-arm after firing |
| **Route** (terminal) | StagedEffects | Temporary — consumed when triggered | Runtime (trigger fires) | No — one-shot, consumed on match |

**Stamp** (definition-level) = definition-level routing. "This tree goes to this entity type's BoundEffects." Required at root of every `effects: []` entry. `Stamp(Bolt, ...)`, `Stamp(EveryBolt, ...)`, etc.

**Stamp** (terminal) = runtime permanent add. `On(ImpactTarget::Impactee, Stamp(When(Died, Fire(Explode))))` permanently adds "explode on death" to the target cell's BoundEffects. Re-arms — survives multiple deaths/lives.

**Route** (terminal) = runtime one-shot. `On(ImpactTarget::Impactee, Route(When(Died, Fire(Explode))))` arms a one-shot listener in the target cell's StagedEffects. Cell dies, explode fires, entry consumed. Hit the cell again to re-route.

This distinction is load-bearing: choosing Stamp vs Route for the same inner tree gives fundamentally different gameplay behavior.

## Real RON Migration Examples

### Aegis Breaker
```ron
// ── Current ──
(
    name: "Aegis",
    life_pool: Some(3),
    effects: [
        On(target: Bolt, then: [When(trigger: BoltLost, then: [Do(LoseLife)])]),
        On(target: Bolt, then: [When(trigger: PerfectBumped, then: [Do(SpeedBoost(multiplier: 1.5))])]),
        On(target: Bolt, then: [When(trigger: EarlyBumped, then: [Do(SpeedBoost(multiplier: 1.1))])]),
        On(target: Bolt, then: [When(trigger: LateBumped, then: [Do(SpeedBoost(multiplier: 1.1))])]),
    ],
)

// ── New ──
(
    name: "Aegis",
    life_pool: Some(3),
    effects: [
        Stamp(Breaker, When(BoltLostOccurred, Fire(LoseLife))),
        Stamp(EveryBolt, When(PerfectBumped, Fire(SpeedBoost(multiplier: 1.5)))),
        Stamp(EveryBolt, When(EarlyBumped, Fire(SpeedBoost(multiplier: 1.1)))),
        Stamp(EveryBolt, When(LateBumped, Fire(SpeedBoost(multiplier: 1.1)))),
    ],
)
```

### Powder Keg (transfer)
```ron
// ── Current ──
(
    name: "Powder Keg",
    legendary: (
        prefix: "",
        effects: [
            On(target: Bolt, then: [
                When(trigger: Impacted(Cell), then: [
                    On(target: Cell, then: [
                        When(trigger: Died, then: [
                            Do(Explode(range: 48.0, damage: 10.0)),
                        ]),
                    ]),
                ]),
            ]),
        ],
    ),
)

// ── New ──
(
    name: "Powder Keg",
    legendary: (
        prefix: "",
        effects: [
            Stamp(Bolt, When(Impacted(Cell), On(ImpactTarget::Impactee, Route(
                When(Died, Fire(Explode(range: 48.0, damage: 10.0)))
            )))),
        ],
    ),
)
```

### Circuit Breaker (evolution)
```ron
// ── Current ──
(
    name: "Circuit Breaker",
    effects: [
        On(target: Bolt, then: [
            When(trigger: PerfectBumped, then: [
                Do(CircuitBreaker(bumps_required: 3, spawn_count: 1, inherit: true,
                    shockwave_range: 160.0, shockwave_speed: 500.0)),
            ]),
        ]),
    ],
)

// ── New ──
(
    name: "Circuit Breaker",
    effects: [
        Stamp(Bolt, When(PerfectBumped, Fire(CircuitBreaker(
            bumps_required: 3, spawn_count: 1, inherit: true,
            shockwave_range: 160.0, shockwave_speed: 500.0,
        )))),
    ],
)
```

### Entropy Engine (kill trigger)
```ron
// ── Current ──
(
    name: "Entropy Engine",
    effects: [
        On(target: Bolt, then: [
            When(trigger: CellDestroyed, then: [
                Do(EntropyEngine(max_effects: 3, pool: [
                    (0.3, Do(SpawnBolts())),
                    (0.25, Do(Shockwave(base_range: 48.0, range_per_level: 0.0, stacks: 1, speed: 400.0))),
                    (0.25, Do(ChainBolt(tether_distance: 120.0))),
                    (0.20, Do(SpeedBoost(multiplier: 1.3))),
                ])),
            ]),
        ]),
    ],
)

// ── New ──
(
    name: "Entropy Engine",
    effects: [
        Stamp(Bolt, When(Killed(Cell), Fire(EntropyEngine(
            max_effects: 3,
            pool: [
                (0.3, SpawnBolts()),
                (0.25, Shockwave(base_range: 48.0, range_per_level: 0.0, stacks: 1, speed: 400.0)),
                (0.25, ChainBolt(tether_distance: 120.0)),
                (0.20, SpeedBoost(multiplier: 1.3)),
            ],
        )))),
    ],
)
```

## RON Loader (walks raw tree, calls builder)

```rust
fn load_def_via_builder(raw: &RawDef) -> Result<ValidDef, EffectError> {
    let ctx = EffectDef::stamp(raw.target.try_into()?);
    load_tree_via_builder(ctx, &raw.tree)
}

fn load_tree_via_builder(ctx: StampContext, raw: &RawTree) -> Result<ValidDef, EffectError> {
    match raw {
        RawTree::Fire(effect) => ctx.fire(effect.try_into()?),
        RawTree::When(trigger, inner) => {
            let chain = ctx.when(trigger.try_into()?);
            load_chain_via_builder(chain, inner)
        }
        RawTree::During(condition, inner) => {
            let during = ctx.during(condition.try_into()?);
            load_during_via_builder(during, inner)
        }
        // ... etc
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
| `Stamp` required at definition root | Bare `When(...)` in `effects: []` | `Err(MissingStamp)` (RON loader) |
| `During` directly wrapping `Fire(X)` — X must be `Reversible` | `Stamp(Bolt, During(NodeActive, Fire(Explode)))` | Compile error (builder) / `Err(NonReversibleInDuring)` (RON loader) |
| `On` only accepts participant targets | `Stamp(Bolt, When(..., On(ActiveBolts, Fire(...))))` | Compile error (builder: `ParticipantTarget` type) / `Err(InvalidOnTarget)` |
| `Spawned` has implicit target — no `On()` | `Spawned(Bolt, On(..., Fire(...)))` | Compile error (no `.on()` on SpawnedContext) / `Err(SpawnedCannotHaveExplicitTarget)` |
| `Spawned(Bolt)` + `Fire(SpawnBolts)` = direct loop | | `Err(SpawnLoop)` (RON loader) / runtime recursion depth limit |
| Indirect spawn loops | A spawns B spawns A | Runtime recursion depth limit (safety net) |
| Named participant not available on trigger | `When(NodeStartOccurred, On(Bolt, ...))` — NodeStart has no participants | Compile error (associated type) / `Err(InvalidParticipant)` |
| `Stamp(This, ...)` not valid | `This` is not a `Target` variant | Compile error (builder) / `Err(InvalidTarget)` (RON loader) |
