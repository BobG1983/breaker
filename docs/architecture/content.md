# Content Identity — Enum Behaviors + RON Instances

**Behaviors** are Rust enums. **Content instances** are RON files that compose and tune those behaviors.

## Chip Content System

All chip content lives in the `chips/` domain. A single `ChipDefinition` type covers all chips. Every chip effect — whether passive (applied on selection) or triggered (fired on game events) — is expressed as an `EffectNode` tree. There is no separate `ChipEffect`, `AmpEffect`, or `AugmentEffect` enum.

### Template-Based Authoring

Chips are authored as **templates** — one RON file per chip concept with per-rarity slots. The loader expands templates into individual `ChipDefinition`s at load time.

```ron
// assets/chips/piercing.chip.ron
(
    name: "Piercing",
    max_taken: 3,
    common: Some((prefix: "Basic", effects: [OnSelected([Piercing(1)])])),
    uncommon: Some((prefix: "Keen", effects: [OnSelected([Piercing(2)])])),
    rare: Some((prefix: "Brutal", effects: [OnSelected([Piercing(3), DamageBoost(0.1)])])),
    legendary: None,
)
```

Each slot is `Option<RaritySlot>` where `RaritySlot` has `prefix` (adjective prepended to the name) and `effects` (full effect list — no inheritance from lower rarities). `max_taken` is shared across all rarities structurally.

See `docs/design/decisions/chip-template-system.md` for the full design decision.

### Unified Effect Model

```rust
// effect/definition.rs (canonical location for Effect types)

// ALL chip and breaker effects are EffectNode trees — no wrapper enums
pub enum EffectNode {
    When { trigger: Trigger, then: Vec<EffectNode> },  // trigger gate
    Do(Effect),                                          // leaf action
    Until { until: Trigger, then: Vec<EffectNode> },   // conditional removal
    Once(Vec<EffectNode>),                              // one-shot fire
}

pub enum Trigger {
    PerfectBump, Bump, EarlyBump, LateBump, BumpWhiff, NoBump,
    PerfectBumped, Bumped, EarlyBumped, LateBumped,
    Impact(ImpactTarget), CellDestroyed, BoltLost, Death,
    Selected, TimeExpires(f32), NodeTimerThreshold(f32),
    // serde renames keep RON files backward-compatible (e.g., OnPerfectBump → PerfectBump)
}

pub enum Effect {
    // Passive effects (via OnSelected / When(Selected, ...))
    Piercing(u32),
    DamageBoost(f32),
    SpeedBoost { target: Target, multiplier: f32 },
    ChainHit(u32),
    SizeBoost(Target, f32),            // Bolt = radius, Breaker = width
    Attraction(AttractionType, f32),
    BumpForce(f32),
    TiltControl(f32),
    RampingDamage { bonus_per_hit: f32 },  // no max_bonus — uncapped accumulation

    // Triggered effects (via bridge systems)
    Shockwave { base_range: f32, range_per_level: f32, stacks: u32, speed: f32 },
    ChainBolt { tether_distance: f32 },
    LoseLife,
    TimePenalty { seconds: f32 },
    SpawnBolts { count: u32, lifespan: Option<f32>, inherit: bool },
    SpeedBoost { target: Target, multiplier: f32 },
    RandomEffect(Vec<(f32, EffectNode)>),
    EntropyEngine { threshold: u32, pool: Vec<(f32, EffectNode)> },
    // ... (see effect/definition.rs for full list)
}

pub enum Target { Bolt, Breaker, AllBolts }
pub enum ImpactTarget { Cell, Breaker, Wall }
pub enum AttractionType { Cell, Wall, Breaker }

// chips/definition.rs — chip content uses EffectNode directly
#[derive(Asset, TypePath, Deserialize)]
pub struct ChipDefinition {
    pub name: String,
    pub description: String,
    pub rarity: Rarity,
    pub max_stacks: u32,              // renamed from max_taken
    pub effects: Vec<EffectNode>,     // EffectNode trees (not TriggerChain)
    pub ingredients: Option<Vec<EvolutionIngredient>>,
    pub template_name: Option<String>,
}
```

### Effect Application

When a player selects a chip, `dispatch_chip_effects` dispatches based on the effect type:

- **`When(trigger: OnSelected, ...)` nodes**: Evaluated immediately. Each `Do(effect)` leaf fires a typed passive event (e.g., `PiercingApplied`, `DamageBoostApplied`) via `fire_passive_event`. Per-effect observer handlers in `effect/effects/` insert or update flat components on bolt or breaker entities.
- **Other `EffectNode` trees** (When(OnPerfectBump, ...), Until(...), etc.): Pushed to `ActiveEffects` resource for evaluation by bridge systems on matching game events.

```rust
// Passive effects land as flat components on entities
struct Piercing(pub u32);           // bolt: max pierces
struct DamageBoost(pub f32);        // bolt: accumulated damage bonus
struct BoltSpeedBoost(pub f32);     // bolt: accumulated flat speed bonus
struct BoltSizeBoost(pub f32);      // bolt: accumulated fractional radius bonus
struct ChainHit(pub u32);           // bolt: chain hit count
struct BreakerSpeedBoost(pub f32);  // breaker: accumulated flat speed bonus
struct BumpForceBoost(pub f32);     // breaker: accumulated flat bump force bonus
struct TiltControlBoost(pub f32);   // breaker: accumulated flat tilt sensitivity bonus
```

Stacking increments the existing component's value. Production systems query for these components directly — if absent, the system uses the base value.

**Adding new content:** new RON template file, no recompile. **Adding new behavior types:** new `Effect` variant in `effect/definition.rs` + new file in `effect/effects/` + `register()` call in `EffectPlugin`, requires recompile (appropriate — new behavior means new code).

### Registries

- **`ChipRegistry`** (`Resource`) loads all chip template RON files and expands them into `ChipDefinition`s at boot. Game logic looks up definitions through the registry.
- **`EvolutionRegistry`** (`Resource`) loads evolution recipe RON files separately. Evolution recipes combine chip ingredients into evolved chips at boss nodes.

## Cell Type Content System (Implemented — Phase 2)

Cell type content lives in `cells/definition.rs` as `CellTypeDefinition`. Each cell type is a RON file; the `CellTypeRegistry` maps single-character aliases to definitions for use in node layout grids.

```rust
// cells/definition.rs
#[derive(Asset, TypePath, Deserialize)]
pub struct CellTypeDefinition {
    pub id: String,
    pub alias: char,            // single-char key used in layout grids
    pub hp: f32,                // hit points (f32 for damage calculations)
    pub color_rgb: [f32; 3],
    pub required_to_clear: bool,
    pub damage_hdr_base: f32,
    pub damage_green_min: f32,
    pub damage_blue_range: f32,
    pub damage_blue_base: f32,
    pub behavior: CellBehavior, // optional: locked, regen_rate (serde default = no behavior)
}

// CellBehavior controls special cell mechanics:
pub struct CellBehavior {
    pub locked: bool,           // immune to damage until all adjacent cells are cleared
    pub regen_rate: Option<f32>, // HP/sec regeneration rate (None = no regen)
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
