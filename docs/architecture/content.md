# Content Identity — Enum Behaviors + RON Instances

**Behaviors** are Rust enums. **Content instances** are RON files that compose and tune those behaviors.

## Chip Content System

All chip content lives in the `chips/` domain. A single `ChipDefinition` type covers all chips. Every chip effect — whether passive (applied on selection) or triggered (fired on game events) — is expressed as an `EffectNode` tree. There is no separate `ChipEffect`, `AmpEffect`, or `AugmentEffect` enum.

### Template-Based Authoring

Chips are authored as **templates** — one RON file per chip concept with per-rarity slots. The loader expands templates into individual `ChipDefinition`s at load time.

```ron
// assets/chips/standard/piercing.chip.ron
(
    name: "Piercing Shot",
    max_taken: 3,
    common: (prefix: "Basic", effects: [On(target: Bolt, then: [Do(Piercing(1))])]),
    uncommon: (prefix: "Keen", effects: [On(target: Bolt, then: [Do(Piercing(2))])]),
    rare: (prefix: "Brutal", effects: [On(target: Bolt, then: [Do(Piercing(3)), Do(DamageBoost(1.1))])]),
)
```

Each slot is `Option<RaritySlot>` where `RaritySlot` has `prefix` (adjective prepended to the name) and `effects` (full effect list — no inheritance from lower rarities). `max_taken` is shared across all rarities structurally.

See `docs/design/decisions/chip-template-system.md` for the full design decision.

### Unified Effect Model

All chip and breaker effects are `EffectNode` trees referencing `EffectKind` — the actual action enum. There is no separate `Effect` enum. The canonical location is `effect/core/types/definitions/enums.rs`.

```rust
// effect/core/types/definitions/enums.rs

pub enum EffectNode {
    When { trigger: Trigger, then: Vec<EffectNode> },
    Do(EffectKind),
    Once(Vec<EffectNode>),
    On { target: Target, #[serde(default)] permanent: bool, then: Vec<EffectNode> },
    Until { trigger: Trigger, then: Vec<EffectNode> },
    Reverse { effects: Vec<EffectKind>, chains: Vec<EffectNode> },  // internal only
}

pub enum EffectKind {
    // Stat effects — applied via fire(), reversed via reverse()
    Piercing(u32),
    DamageBoost(f32),
    SpeedBoost { multiplier: f32 },
    SizeBoost(f32),
    BumpForce(f32),
    Attraction { attraction_type: AttractionType, force: f32, #[serde(default)] max_force: Option<f32> },
    RampingDamage { damage_per_trigger: f32 },
    QuickStop { multiplier: f32 },

    // AoE / spawn effects
    Shockwave { base_range: f32, range_per_level: f32, stacks: u32, speed: f32 },
    Pulse { base_range: f32, range_per_level: f32, stacks: u32, speed: f32, #[serde(default = "default_pulse_interval")] interval: f32 },
    Explode { range: f32, damage_mult: f32 },
    ChainLightning { arcs: u32, range: f32, damage_mult: f32, #[serde(default = "default_chain_lightning_arc_speed")] arc_speed: f32 },
    PiercingBeam { damage_mult: f32, width: f32 },
    TetherBeam { damage_mult: f32, #[serde(default)] chain: bool },

    // Spawn effects
    SpawnBolts { #[serde(default = "one")] count: u32, #[serde(default)] lifespan: Option<f32>, #[serde(default)] inherit: bool },
    ChainBolt { tether_distance: f32 },
    SpawnPhantom { duration: f32, max_active: u32 },
    GravityWell { strength: f32, duration: f32, radius: f32, max: u32 },

    // Protection / special
    Shield { duration: f32 },          // spawns a timed floor wall (ShieldWall + ShieldWallTimer)
    SecondWind,                        // unit variant — no fields
    LoseLife,
    TimePenalty { seconds: f32 },

    // Breaker utility effects
    FlashStep,                             // unit variant — teleport on reversal-during-settling
    MirrorProtocol { #[serde(default)] inherit: bool },
    Anchor { bump_force_multiplier: f32, perfect_window_multiplier: f32, plant_delay: f32 },
    CircuitBreaker { bumps_required: u32, #[serde(default = "one")] spawn_count: u32, #[serde(default)] inherit: bool, shockwave_range: f32, shockwave_speed: f32 },

    // Meta effects
    RandomEffect(Vec<(f32, EffectNode)>),
    EntropyEngine { max_effects: u32, pool: Vec<(f32, EffectNode)> },
}
```

See `docs/architecture/effects/core_types.md` for full field-level documentation of each variant.

### Effect Application

When a player selects a chip, the chip dispatch system pushes the chip's `EffectNode` trees onto the breaker/bolt entity's `BoundEffects`. Bridge systems in `effect/triggers/` evaluate `BoundEffects` and `StagedEffects` on every matching trigger, calling `commands.fire_effect(entity, effect_kind, chip_name)` for each `Do` leaf they encounter.

- **Stat effects** (`SpeedBoost`, `DamageBoost`, `Piercing`, `SizeBoost`, `BumpForce`, `QuickStop`): `fire()` pushes a value onto an `Active*` stack component (e.g., `ActiveSpeedBoosts(Vec<f32>)`). Consumers read these stacks directly via `.multiplier()` / `.total()` — there is no separate `Recalculate` step.
- **AoE/spawn effects** (`Shockwave`, `ChainLightning`, `Explode`, etc.): `fire()` spawns a request entity or directly damages nearby cells via `DamageDealt<Cell>` message. These carry chip attribution via `EffectSourceChip` component (and `DamageDealt.source_chip` field) for damage tracking.
- **Shield**: `fire()` spawns a `ShieldWall` entity (a timed visible floor wall) with a `ShieldWallTimer`. If a wall already exists, the timer is reset in-place. `tick_shield_wall_timer` despawns the wall when the timer expires. No component is inserted on the target entity.

**Adding new content:** new RON template file, no recompile. **Adding new behavior types:** new `EffectKind` variant in `effect/core/types/definitions/enums.rs` + new module in `effect/effects/` + fire/reverse arms in `definitions/fire.rs` and `definitions/reverse.rs` + `register()` call, requires recompile.

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
