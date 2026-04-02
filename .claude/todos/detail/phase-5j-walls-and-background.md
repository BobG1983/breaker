# 5j: Walls & Background

**Goal**: Make playfield boundaries visible and the background alive, establishing the spatial canvas for all foreground elements.

Architecture: `docs/architecture/rendering/walls_and_background.md`, `docs/architecture/rendering/shaders.md`

## What to Build

### 1. Wall Rendering via AttachVisuals

Current: Invisible collision entities.

Target: Walls get `AttachVisuals` at spawn with `Shape::Rectangle`, subtle glow. Temperature tint driven by `run/` domain via `SetModifier(ColorShift(...))` on node transitions.

### 2. Wall Bolt-Impact Flash

When bolt hits a wall, fire a small recipe at the impact position:
- `SparkBurst` + `ExpandingRing` at the collision point
- Brief, localized — the wall entity itself stays at baseline glow

### 3. Shield Barrier Energy Field

Custom `shield.wgsl` Material2d shader in `rantzsoft_vfx`:
- Semi-transparent energy field below breaker, spanning breaker width
- Animated hexagonal/honeycomb pattern (pulsing white, per DR-3)
- Procedural crack damage via noise-seeded dark regions (up to 5 crack seeds)
- On each charge loss: recipe fires on barrier (sparks + intensity spike), crack seed added
- On last charge: `TriggerFracture` on barrier entity + despawn
- See `docs/architecture/rendering/walls_and_background.md` — Shield Barrier section

### 4. Background Grid

Single quad with `grid.wgsl` shader covering playfield area:
- Grid lines computed from UV coordinates
- Configurable density via `VfxConfig`
- Color driven by `RunTemperature` (lerp between cool/hot palette endpoints)
- Very dim — spatial reference, not competition with gameplay
- Spawned by `screen/playing/` on `OnEnter(PlayingState::Active)`

### 5. Background Energy Sprites

Occasional GlowMote particles traveling along grid lines:
- Very subtle — 1-2 visible at any time
- Sense that the grid is alive without being distracting

### 6. Void Background Color

Update `ClearColor` to near-black (#050510 deep blue-black). Allows gravity well voids to register as even darker.

## Dependencies

- **Requires**: 5c (crate), 5d (bloom), 5e (particles for sparks/motes), 5f (temperature)
- DR-3 resolved: patterned white shield. DR-6 resolved: configurable grid density.
- **Independent of**: 5g, 5h, 5i

## Verification

- Walls visible as subtle glowing SDF borders
- Wall impact flash fires at bolt collision point
- Shield barrier visible with hex pattern, cracks on charge loss, shatters on last
- Background grid renders with temperature-driven color
- Energy sprites drift along grid lines
- Void color is correct deep blue-black
- All existing tests pass
