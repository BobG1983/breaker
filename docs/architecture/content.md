# Content Identity — Enum Behaviors + RON Instances

**Behaviors** are Rust enums. **Content instances** are RON files that compose and tune those behaviors.

## Chip Content System (Implemented — Phase 4b)

All chip content lives in the `chips/` domain. A single `ChipDefinition` type covers Amps, Augments, and Overclocks. Each chip has exactly one `ChipEffect`, which wraps either an `AmpEffect` (bolt) or `AugmentEffect` (breaker).

```rust
// chips/definition.rs

// Behavior enums — exhaustive, matchable, compiler-checked
pub(crate) enum AmpEffect {
    Piercing(u32),      // bolt passes through N cells before stopping
    DamageBoost(f32),   // fractional bonus damage per stack
    SpeedBoost(f32),    // flat bolt speed increase per stack
    ChainHit(u32),      // chains to N additional cells on hit
    SizeBoost(f32),     // increases bolt radius by a fraction per stack
}

pub(crate) enum AugmentEffect {
    WidthBoost(f32),    // flat breaker width increase per stack
    SpeedBoost(f32),    // flat breaker speed increase per stack
    BumpForce(f32),     // flat bump force increase per stack
    TiltControl(f32),   // flat tilt sensitivity increase per stack
}

pub(crate) enum ChipEffect {
    Amp(AmpEffect),
    Augment(AugmentEffect),
    Overclock,          // deferred to Phase 4d
}

// Content instance — data-driven, no recompile to add
#[derive(Asset, TypePath, Deserialize)]
pub(crate) struct ChipDefinition {
    pub name: String,
    pub kind: ChipKind,         // Amp | Augment | Overclock
    pub description: String,
    pub rarity: Rarity,         // Common | Uncommon | Rare | Legendary
    pub max_stacks: u32,
    pub effect: ChipEffect,
}
```

```ron
// assets/amps/piercing.amp.ron
(
    name: "Piercing Shot",
    kind: Amp,
    description: "Bolt passes through the first cell it hits",
    rarity: Common,
    max_stacks: 3,
    effect: Amp(Piercing(1)),
)
```

**Adding new content:** new RON file, no recompile. **Adding new behavior types:** new enum variant, requires recompile (appropriate — new behavior means new code).

The `ChipRegistry` (`Resource`) loads all chip RON definitions at boot. Game logic looks up definitions through the registry by name string. A paired `Vec<String>` preserves insertion order for deterministic chip offers.

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

## Effect Application — Flat Components on Entities

When a player selects a chip, `apply_chip_effect` fires a `ChipEffectApplied` observer event. Per-effect observer handlers in `chips/effects/` insert or update a flat component on the bolt or breaker entity:

```rust
// Amp effects land on the bolt entity
struct Piercing(pub u32);           // max pierces (reset on wall hit)
struct PiercingRemaining(pub u32);  // remaining pierces this wall-bounce cycle
struct DamageBoost(pub f32);        // accumulated fractional damage bonus
struct BoltSpeedBoost(pub f32);     // accumulated flat speed bonus
struct BoltSizeBoost(pub f32);      // accumulated fractional radius bonus
struct ChainHit(pub u32);           // chain hit count

// Augment effects land on the breaker entity
struct WidthBoost(pub f32);         // accumulated flat width bonus
struct BreakerSpeedBoost(pub f32);  // accumulated flat speed bonus
struct BumpForceBoost(pub f32);     // accumulated flat bump force bonus
struct TiltControlBoost(pub f32);   // accumulated flat tilt sensitivity bonus
```

Stacking increments the existing component's value. Production systems query for these components directly — if absent, the system uses the base value. No wrapper struct or registry lookup at gameplay time.

## RON Validation — ron-lsp

Every RON file MUST include a type annotation comment on the first line linking it to the Rust type it deserializes into:

```ron
// assets/amps/piercing.amp.ron
/* @[brickbreaker::chips::ChipDefinition] */
(
    name: "Piercing Shot",
    ...
)
```

[`ron-lsp`](https://github.com/jasonjmcghee/ron-lsp) uses these annotations to validate RON files against actual Rust struct/enum definitions — catching type mismatches, missing fields, and invalid enum variants without running the game. Run `ron-lsp check .` to validate all annotated RON files in bulk.
