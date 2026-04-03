# Cell Builder — Struct Definitions

## Typestate markers

```rust
// ── Position ────────────────────────────────────────────
pub(crate) struct NoPosition;
pub(crate) struct HasPosition {
    pub(in crate::cells::builder) pos: Vec2,
}

// ── Dimensions ──────────────────────────────────────────
pub(crate) struct NoDimensions;
pub(crate) struct HasDimensions {
    pub(in crate::cells::builder) width: f32,
    pub(in crate::cells::builder) height: f32,
}

// ── Health ──────────────────────────────────────────────
pub(crate) struct NoHealth;
pub(crate) struct HasHealth {
    pub(in crate::cells::builder) hp: f32,
}

// ── Visual ──────────────────────────────────────────────
pub(crate) struct Unvisual;
pub(crate) struct Rendered {
    pub(crate) mesh: Handle<Mesh>,
    pub(crate) material: Handle<ColorMaterial>,
}
pub(crate) struct Headless;
```

## CellBehavior enum

```rust
#[derive(Deserialize, Clone, Debug)]
pub(crate) enum CellBehavior {
    Regen { rate: f32 },
    Shielded(ShieldBehavior),
}
```

`ShieldBehavior` stays as-is (spawn-time config for orbit children):
```rust
#[derive(Deserialize, Clone, Debug)]
pub(crate) struct ShieldBehavior {
    pub count: u32,
    pub radius: f32,
    pub speed: f32,
    pub hp: f32,
    pub color_rgb: [f32; 3],
}
```

## OptionalCellData

```rust
#[derive(Default)]
pub(in crate::cells::builder) struct OptionalCellData {
    // Definition layer (set by definition)
    pub definition_damage_visuals: Option<CellDamageVisuals>,
    pub definition_behaviors: Option<Vec<CellBehavior>>,
    pub definition_effects: Option<Vec<RootEffect>>,
    pub definition_color_rgb: Option<[f32; 3]>,
    pub definition_required_to_clear: Option<bool>,
    pub definition_alias: Option<char>,

    // Override layer (set by individual setters — wins over definition regardless of call order)
    pub override_hp: Option<f32>,
    pub override_damage_visuals: Option<CellDamageVisuals>,
    pub override_behaviors: Option<Vec<CellBehavior>>,
    pub override_effects: Option<Vec<RootEffect>>,
    pub override_color_rgb: Option<[f32; 3]>,
    pub override_required_to_clear: Option<bool>,
}
```

Resolution order: override > definition > default.

## CellBuilder

```rust
pub(crate) struct CellBuilder<P, D, H, V> {
    pub(in crate::cells::builder) position: P,
    pub(in crate::cells::builder) dimensions: D,
    pub(in crate::cells::builder) health: H,
    pub(in crate::cells::builder) visual: V,
    pub(in crate::cells::builder) optional: OptionalCellData,
}
```

Entry point:
```rust
impl Cell {
    pub(crate) fn builder() -> CellBuilder<NoPosition, NoDimensions, NoHealth, Unvisual> {
        CellBuilder {
            position: NoPosition,
            dimensions: NoDimensions,
            health: NoHealth,
            visual: Unvisual,
            optional: OptionalCellData::default(),
        }
    }
}
```
