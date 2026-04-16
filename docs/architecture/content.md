# Content Identity — Enum Behaviors + RON Instances

**Behaviors** are Rust enums. **Content instances** are RON files that compose and tune those behaviors.

## Chip Content System

All chip content lives in the `chips/` domain. A single `ChipDefinition` type covers all chips. Every chip effect — whether passive (applied on selection) or triggered (fired on game events) — is expressed as a `RootNode` (`Stamp(StampTarget, Tree)` or `Spawn(EntityKind, Tree)`) carrying a `Tree`. There is no separate `ChipEffect`, `AmpEffect`, or `AugmentEffect` enum.

### Template-Based Authoring

Chips are authored as **templates** — one RON file per chip concept with per-rarity slots. The loader expands templates into individual `ChipDefinition`s at load time.

```ron
// assets/chips/standard/piercing.chip.ron
(
    name: "Piercing Shot",
    max_taken: 3,
    common:   (prefix: "Basic",   effects: [Stamp(Bolt, Fire(Piercing(charges: 1)))]),
    uncommon: (prefix: "Keen",    effects: [Stamp(Bolt, Fire(Piercing(charges: 2)))]),
    rare:     (prefix: "Brutal",  effects: [Stamp(Bolt, Sequence([
        Fire(Piercing(charges: 3)),
        Fire(DamageBoost(multiplier: 1.1)),
    ]))]),
)
```

Each slot is `Option<RaritySlot>` where `RaritySlot` has `prefix` (adjective prepended to the name) and `effects` (full effect list — no inheritance from lower rarities). `max_taken` is shared across all rarities structurally.

See `docs/design/decisions/chip-template-system.md` for the full design decision.

### Unified Effect Model

All chip and breaker effects are `RootNode` lists. The inner `Tree` references `EffectType` variants — each variant wraps a per-effect config struct that implements `Fireable`. The canonical location is `effect_v3/types/`.

```rust
// effect_v3/types/effect_type.rs
pub enum EffectType {
    SpeedBoost(SpeedBoostConfig),       // multiplier
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

```rust
// effect_v3/types/tree.rs
pub enum Tree {
    Fire(EffectType),
    When(Trigger, Box<Self>),
    Once(Trigger, Box<Self>),
    During(Condition, Box<ScopedTree>),
    Until(Trigger, Box<ScopedTree>),
    Sequence(Vec<Terminal>),
    On(ParticipantTarget, Terminal),
}

// effect_v3/types/root_node.rs
pub enum RootNode {
    Stamp(StampTarget, Tree),
    Spawn(EntityKind, Tree),
}
```

See `docs/architecture/effects/core_types.md` for the full type system reference (including `ScopedTree`, `Terminal`, `StampTarget`, `Trigger`, `Condition`, `EntityKind`, `ParticipantTarget`, `TriggerContext`).

### Effect Application

When a player selects a chip, `dispatch_chip_effects` walks the chip's `effects: Vec<RootNode>`. For each `RootNode::Stamp(target, tree)`, it resolves the target via `DispatchTargets` and calls `commands.stamp_effect(entity, chip_name, tree)` on each resolved entity (or fires the effect immediately if `tree` is `Tree::Fire(_)`). Trigger bridge systems in `effect_v3/triggers/<category>/` later walk the entries on each matching trigger and queue `commands.fire_effect`.

- **Stat effects** (`SpeedBoost`, `DamageBoost`, `Piercing`, `SizeBoost`, `BumpForce`, `QuickStop`, `Vulnerable`, `RampingDamage`, `Anchor`, `FlashStep`, `Attraction`): `Fireable::fire` pushes onto an `EffectStack<Config>` component on the entity. A per-effect recalculation system in `EffectV3Systems::Tick` reads the stack and applies the effective value.
- **AoE/spawn effects** (`Shockwave`, `ChainLightning`, `Explode`, `Pulse`, `GravityWell`, `TetherBeam`, etc.): `Fireable::fire` spawns a child entity (or sends a `DamageDealt<Cell>` message). These carry chip attribution via the `EffectSourceChip` component for damage tracking.
- **Shield**: `Fireable::fire` spawns a `ShieldWall` entity (a timed visible floor wall) with a `ShieldWallTimer`. If a wall already exists, the timer is reset in-place. `tick_shield_wall_timer` despawns the wall when the timer expires.

**Adding new content:** new RON template file, no recompile. **Adding new behavior types:** new `EffectType` variant + new module in `effect_v3/effects/` + `Fireable` impl + dispatch arm + (optional) `ReversibleEffectType` variant + `Reversible` impl + `MyConfig::register(app)` call. See `docs/architecture/effects/adding_effects.md`.

### Registries

- **`ChipTemplateRegistry`** (`SeedableRegistry` `Resource`) loads all `.chip.ron` template files from `assets/chips/standard/`. Templates are expanded into `ChipDefinition`s at catalog-build time via `populate_catalog`.
- **`ChipCatalog`** (`Resource`) holds all expanded `ChipDefinition`s plus `Recipe`s. Built at load time from `ChipTemplateRegistry` + `EvolutionTemplateRegistry`. Paired `Vec<String>` preserves insertion order for deterministic chip offers.
- **`EvolutionTemplateRegistry`** (`SeedableRegistry` `Resource`) loads `.evolution.ron` files from `assets/chips/evolutions/`. Evolution templates are expanded into `ChipDefinition`s with `rarity: Evolution` and `Recipe` entries at catalog-build time.

## Cell Type Content System (Implemented — Phase 2)

Cell type content lives in `cells/definition.rs` as `CellTypeDefinition`. Each cell type is a RON file; the `CellTypeRegistry` maps string aliases to definitions for use in node layout grids.

```rust
// cells/definition.rs
#[derive(Asset, TypePath, Deserialize)]
pub struct CellTypeDefinition {
    pub id: String,
    pub alias: String,                  // string key used in layout grids (e.g., "S", "Gu")
    #[serde(default)]
    pub toughness: Toughness,           // Weak | Standard | Tough — determines base HP
    pub color_rgb: [f32; 3],
    pub required_to_clear: bool,
    pub damage_hdr_base: f32,
    pub damage_green_min: f32,
    pub damage_blue_range: f32,
    pub damage_blue_base: f32,
    pub behaviors: Option<Vec<CellBehavior>>, // optional per-cell behaviors
}

// CellBehavior controls special cell mechanics:
pub enum CellBehavior {
    Regen { rate: f32 },            // HP/sec regeneration rate
    Guarded(GuardedBehavior),       // guardian shield with hp_fraction, color, slide_speed
}
```

## RON Validation — ron-lsp

Every RON file MUST include a type annotation comment on the first line linking it to the Rust type it deserializes into:

```ron
// assets/chips/piercing.chip.ron
/* @[brickbreaker::chips::ChipTemplate] */
(
    name: "Piercing",
    max_taken: 3,
    ...
)
```

[`ron-lsp`](https://github.com/jasonjmcghee/ron-lsp) uses these annotations to validate RON files against actual Rust struct/enum definitions — catching type mismatches, missing fields, and invalid enum variants without running the game. Run `ron-lsp check .` to validate all annotated RON files in bulk.
