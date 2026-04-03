# Cell Builder — Typestate Dimensions

## Dimensions

| Dimension | Uninit | Terminal states | Data carried |
|-----------|--------|-----------------|--------------|
| **Position** | `NoPosition` | `HasPosition` | `pos: Vec2` |
| **Dimensions** | `NoDimensions` | `HasDimensions` | `width: f32, height: f32` |
| **Health** | `NoHealth` | `HasHealth` | `hp: f32` |
| **Visual** | `Unvisual` | `Rendered`, `Headless` | `Rendered { mesh: Handle<Mesh>, material: Handle<ColorMaterial> }`, `Headless` (unit) |

## Builder struct

```rust
pub(crate) struct CellBuilder<P, D, H, V> {
    position: P,
    dimensions: D,
    health: H,
    visual: V,
    optional: OptionalCellData,
}
```

## Transition methods

Each dimension has exactly one transition method (or two for Visual):

| Method | From | To | Notes |
|--------|------|----|-------|
| `.position(Vec2)` | `NoPosition` | `HasPosition` | |
| `.dimensions(width, height)` | `NoDimensions` | `HasDimensions` | |
| `.hp(f32)` | `NoHealth` OR `HasHealth` | `HasHealth` | Transitions if `NoHealth`, overrides if `HasHealth`. Writes to override layer — wins over `definition()` regardless of call order |
| `.rendered(mesh, material)` | `Unvisual` | `Rendered` | Production path |
| `.headless()` | `Unvisual` | `Headless` | Test path |

## Optional methods (available in ANY state)

These don't change typestate — they mutate `OptionalCellData`:

| Method | What it sets | Notes |
|--------|-------------|-------|
| `.definition(&CellTypeDefinition)` | hp, damage visuals, behaviors, effects, alias, required_to_clear | Sets `NoHealth` → `HasHealth` as side effect |
| `.with_behavior(CellBehavior)` | Adds one behavior | Appends to behaviors vec |
| `.with_behaviors(Vec<CellBehavior>)` | Adds multiple behaviors | Appends all to behaviors vec |
| `.alias(char)` | `CellTypeAlias` | For hot-reload tracking |
| `.required_to_clear(bool)` | `RequiredToClear` marker | Defaults to false |
| `.damage_visuals(CellDamageVisuals)` | Damage color feedback params | |
| `.effects(Vec<RootEffect>)` | Effect chains | Override > definition > none |
| `.color_rgb([f32; 3])` | Cell color | Override > definition > default |

## Terminal methods

`build()` and `spawn()` are ONLY available when ALL four dimensions are in terminal states:

```
impl CellBuilder<HasPosition, HasDimensions, HasHealth, Rendered> { build(), spawn() }
impl CellBuilder<HasPosition, HasDimensions, HasHealth, Headless> { build(), spawn() }
```

No other combinations expose `build()` or `spawn()`.
