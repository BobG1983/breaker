# 5f: Temperature Palette & Data-Driven Enums

**Goal**: Build the temperature palette system and implement all visual composition types in `rantzsoft_vfx` that entity visual steps (5g-5j) will consume.

Architecture: `docs/architecture/rendering/temperature.md`, `docs/architecture/rendering/types.md`

## What to Build

### 1. Temperature Palette System

- `RunTemperature(f32)` resource in `run/` domain: 0.0 (cool) to 1.0 (hot), derived from node progression
- `TemperaturePalette` in `GraphicsDefaults` RON (shared/): cool/hot endpoint colors for grid, bloom, walls
- System in `run/` that reads `RunState.node_index` and updates `RunTemperature` on node transition
- Instant snap on transition (no interpolation — transition animation masks the change)
- Grid, bloom, and wall systems read RunTemperature to interpolate between palette endpoints

### 2. Hue Enum

~148 CSS named colors + `Custom(f32, f32, f32, f32)` in `rantzsoft_vfx`. RON files use CSS color names directly (e.g., `color: CadetBlue`, `color: Gold`). Implements `From<Hue> for Color`, `From<Color> for Hue`, `From<Hue> for LinearRgba`.

### 3. Shape Enum

Rectangle, RoundedRectangle, Hexagon, Octagon, Circle, Diamond, Shield, Angular, Crystalline, Custom(CustomShape). Selects which SDF function the entity_glow shader evaluates via an integer `shape_type` uniform.

### 4. Aura Enum

ShieldShimmer, TimeDistortion, PrismaticSplit — each with params (pulse_speed, intensity, color, etc.). Rendered via single `AuraMaterial` with variant uniform (not separate Material2d types).

### 5. Trail Enum

ShieldEnergy, Afterimage, PrismaticSplit — each with params. Trails are top-level entities (NOT children), track source entity via `TrailSource` component.

### 6. Visual Parameters

- `GlowParams` (core_brightness, halo_radius, halo_falloff, bloom)
- `HdrBrightness(f32)`, `BloomIntensity(f32)`, `EmissiveStrength(f32)` newtypes
- `EntityVisualConfig` struct (shape, color, glow, aura, trail)

### 7. VisualModifier Enum

12 variants: TrailLength, GlowIntensity, CoreBrightness, HaloRadius, ShapeScale, SpikeCount, ColorShift, ColorCycle, AlphaOscillation, SquashStretch, AfterimageTrail, RotationSpeed. SquashStretch is shader-uniform-based (doesn't affect collision AABB).

### 8. RON Integration

Update entity RON files with `rendering` blocks:
- Cell definition RON: shape, color, glow, damage_recipe, death_recipe, hit_recipe
- Breaker archetype RON: shape, color, glow, aura, trail, bump recipes
- Bolt definition RON: shape, color, glow, trail, spawn/death/expiry recipes
- Use `#[serde(default)]` for incremental migration

## What NOT to Do

- Do NOT implement the rendering of these types (that's 5g-5j)
- Do NOT implement modifier stacking logic (that's 5n)
- Define the types, integrate with RON, wire `AttachVisuals` message

## Dependencies

- **Requires**: 5c (rantzsoft_vfx crate exists)

## Verification

- All enums deserialize from RON correctly
- Existing RON files load with new rendering blocks (serde defaults for backwards compat)
- `AttachVisuals` message can be sent with `EntityVisualConfig`
- Temperature resource updates on node transitions
- All existing tests pass
