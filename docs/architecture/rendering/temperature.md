# Temperature Palette — Hybrid Model

Runtime shift for global/ambient elements. Authored in RON for entities.

## RunTemperature Resource

```rust
/// Run progression temperature. 0.0 = cool (early nodes), 1.0 = hot (late nodes).
/// Drives ambient visual tone: grid tint, bloom color, wall border tint.
#[derive(Resource, Clone, Debug)]
pub struct RunTemperature(pub f32);
```

**Owned by `run/` domain.** Updated on node transitions. Read by `rantzsoft_vfx` systems for grid/bloom and by game systems for wall modifiers.

**Formula:** `temperature = (node_index as f32 / expected_nodes_per_act as f32).clamp(0.0, 1.0)`. The `expected_nodes_per_act` value comes from run configuration. For infinite runs, temperature cycles or caps — details TBD in run design.

**Palette endpoints** defined in `GraphicsDefaults` RON (in `shared/`):

```rust
pub struct TemperaturePalette {
    pub cool_grid: Hue,      // grid tint at temperature 0.0
    pub hot_grid: Hue,       // grid tint at temperature 1.0
    pub cool_bloom: Hue,     // ambient bloom at 0.0
    pub hot_bloom: Hue,      // ambient bloom at 1.0
    pub cool_wall: Hue,      // wall border tint at 0.0
    pub hot_wall: Hue,       // wall border tint at 1.0
}
```

Systems lerp between cool and hot endpoints using the temperature value.

## Transition Behavior

**Instant snap.** RunTemperature updates immediately on node transition. The transition animation (Flash/Sweep/Glitch/Collapse) masks the color change. By the time the new node is visible, the new temperature is already active. No interpolation system needed.

## Runtime Shifts (driven by RunTemperature)

| Element | How it reads temperature |
|---------|------------------------|
| Background grid | `grid.wgsl` shader reads temperature uniform → lerps grid line color between cool and hot |
| Ambient bloom | Bloom post-process tints toward warm at high temperature (uniform on camera) |
| Wall border tint | `run/` domain sends `SetModifier(ColorShift(...))` on wall entities each node transition |

## Authored in RON (no runtime shift)

- Cell colors (per cell definition RON)
- Bolt colors (per bolt definition RON)
- Breaker colors (per archetype RON)
- Particle base colors (per recipe RON)

The "temperature" feel comes from node layouts using appropriately-colored cell definitions for early vs late nodes. Early nodes use cool-tinted cell types; late nodes use warm-tinted cell types. The temperature resource shifts the ambient environment to match.
