# Walls & Background

## Wall Rendering

Walls are gameplay entities that receive `AttachVisuals` at spawn — same SDF-on-quad approach as all other entities. `Shape::Rectangle` with configurable glow params.

**Wall-specific visuals** driven via the modifier system from `run/` domain:
- Temperature tint: `SetModifier(ColorShift(lerped_color))` on wall entities at each node transition
- Border glow intensity: `SetModifier(GlowIntensity(...))` based on run state

### Bolt-Wall Impact Flash

When a bolt hits a wall, the collision system (or the wall's hit recipe) triggers a small localized glow at the impact point. Two approaches depending on implementation:

1. **Recipe approach:** `ExecuteRecipe` with a small `SparkBurst` + `ExpandingRing` at the impact position. Simple, consistent with other impact VFX.
2. **Modifier approach:** Brief `SetModifier(CoreBrightness(2.0))` on the wall entity with a timed reset. Simpler but the glow covers the whole wall, not just the impact point.

Recommendation: recipe approach (localized VFX at impact position). The wall entity itself stays at baseline glow.

## Background Grid

**Single quad** covering the playfield area, rendered behind all entities. Uses `grid.wgsl` shader.

```
Entity:
  - Mesh2d (rectangle covering playfield bounds)
  - MeshMaterial2d<GridMaterial>
  - Transform at z = -10.0 (behind everything)
```

**GridMaterial** is a custom `Material2d` in `rantzsoft_vfx` (game-agnostic grid primitive).

**Shader uniforms:**
- `line_spacing: f32` — distance between grid lines (configurable via VfxConfig)
- `line_thickness: f32` — grid line width
- `color: vec4<f32>` — grid line color (from RunTemperature palette)
- `glow_intensity: f32` — overall grid brightness
- `playfield_bounds: vec4<f32>` — min_x, min_y, max_x, max_y

**Fragment shader:** Compute grid lines from world-space position. Thin bright lines with subtle exponential falloff perpendicular to the line. Grid is subtle — it provides spatial reference without competing with gameplay entities.

**Distortion interaction:** RadialDistortion post-processing warps screen UVs, which naturally warps the grid along with everything else. The grid doesn't need special distortion handling.

**Spawned by:** `screen/playing/` on `OnEnter(PlayingState::Active)`. Despawned on exit. The grid entity reads `RunTemperature` each frame to update its color uniform.

## Shield Barrier

The shield chip effect spawns a **barrier entity** — a semi-transparent energy field below the breaker. Uses a custom `shield.wgsl` Material2d shader with animated hexagonal pattern (pulsing white, per DR-3).

### Lifecycle

1. **Spawn:** When `ShieldActive` is added to the breaker entity, the effect system spawns a barrier entity:
   - Mesh2d (rectangle spanning breaker width × barrier height)
   - MeshMaterial2d<ShieldMaterial>
   - Positioned below breaker, z-layer behind breaker but in front of grid

2. **Idle:** Animated hexagonal shimmer. Pulsing white. Fully intact.

3. **Charge consumed:** Each charge loss:
   - Recipe fires on the barrier entity (sparks + brief intensity spike)
   - A `crack_seed` position is added (uniform array, up to 5). The shader procedurally darkens hex cells near each crack seed via noise sampling.
   - `shimmer_speed` increases, `intensity` decreases — looks increasingly unstable

4. **Final charge:** Dark regions cover 80%+ of the field. Game fires `TriggerFracture` on the barrier entity. Barrier despawns.

### Shield Shader Detail

See [shaders.md](shaders.md) — `shield.wgsl` section for the full shader algorithm.

## What Lives Where

| Concern | Owner |
|---------|-------|
| Wall `AttachVisuals` at spawn | `run/node/` (where walls are spawned) |
| Wall temperature tint modifiers | `run/temperature.rs` |
| Wall impact VFX | `bolt/` or `wall/` domain (sends ExecuteRecipe at collision) |
| Grid entity spawn/update | `screen/playing/` |
| `grid.wgsl`, `shield.wgsl` shaders | `rantzsoft_vfx` |
| Shield barrier spawn/lifecycle | `effect/effects/shield/` |
