# Gameplay Elements

How each core game object should look. All elements follow the "light is the material" principle — nothing is opaque or textured, everything emits or glows.

## Bolt

The bolt is the **most important visual element**. It must be trackable at all times, through any amount of particle chaos, screen effects, or visual density. Pillar 4: "the bolt and breaker occupy a visual layer that cuts through effects."

### Visual Design
- **Form**: Energy orb — a bright central sphere surrounded by a softer glow halo
- **Wake**: Trailing energy wake behind the bolt showing direction and recent path. Wake length scales with speed.
- **Core**: Bright white/warm center that is always the brightest non-flash element on screen. HDR >1.0 so it blooms.
- **Halo**: Softer glow around the core. Color can be modified by active effects (chip visual modifiers can tint or add secondary effects to the halo).

### State Communication (Visual Only)
The bolt communicates its state through its appearance, not through floating text or numbers:
- **Speed**: Wake length and brightness increase with speed. A fast bolt has a long, vivid trail. A slow bolt has a short, dim trail.
- **Piercing active**: Bolt gains a sharper, more angular glow or visible energy spikes
- **Damage boosted**: Core brightness increases, halo color shifts toward hot (amber/white)
- **Shield on bolt**: Additional visible aura ring around the bolt (distinct from the halo)

### Chip Effect Visual Modifiers
Certain chip effects modify the bolt's appearance additively — they layer on top of the base bolt identity:
- Longer trails
- Additional particle emitters
- Color tint shifts on the halo
- Spikier/more angular glow shapes
- Dripping/streaming energy effects

These modifiers are defined per-effect and stack. A bolt with 3 stacked speed boosts should look visibly different from a bolt with 1. The visual complexity of the bolt IS the build's expression.

## Breaker

The breaker is the player's avatar — the thing they identify with and control. Each of the three archetypes has a **fully distinct visual identity**: different shape, different color, different aura, different trail effects.

### Archetype Visual Identities

| Archetype | Shape | Color Accent | Aura | Dash Trail |
|-----------|-------|-------------|------|------------|
| Aegis | Shield-shaped — wider, protective, convex front face | Blue/cyan | Shield energy shimmer — a defensive field visible around the breaker | Shield energy trail — solid, protective-feeling wake |
| Chrono | Angular/sharp — sleek, fast-looking, geometric edges | Amber/gold | Time distortion ripples — subtle visual echo/afterimage at rest | Time-echo afterimages — multiple fading copies showing recent positions |
| Prism | Crystalline — faceted, refractive, multi-angled | Magenta/violet | Prismatic light splitting — rainbow edge refractions, light scatters from surfaces | Prismatic light split trail — the trail separates into spectral colors |

### Breaker States
- **Idle**: Base appearance with ambient aura
- **Moving**: Aura intensifies slightly in the direction of movement
- **Dashing**: Full trail effect active, glow intensifies, archetype-specific dash visuals at maximum
- **Settling** (post-dash): Trail fading, aura returning to idle intensity

### Data-Driven Composition
Breaker visuals are composed from RON-defined enum values — shape, color accent, aura type, trail type. New archetypes can be created by combining existing visual building blocks without new code. See `data-driven-graphics.md`.

### Chip Effect Visual Modifiers
Like the bolt, the breaker's appearance can be modified by active chip effects:
- Width boost: Breaker visually stretches (already implemented via EntityScale)
- Speed boost: Aura trails become more intense/longer
- Bump force boost: Impact point glow intensifies

## Cells

Cells (bricks) are the targets. Each cell type has a **distinct shape AND color AND damage state**, making them immediately identifiable at a glance.

### Cell Type Visual Identities

| Cell Type | Shape | Color | Distinguishing Feature |
|-----------|-------|-------|----------------------|
| Standard | Rectangle/rounded rectangle | Temperature-following (shifts with run progression) | Clean, simple — the baseline |
| Tough | Hexagonal or reinforced rectangle with visible internal structure | Brighter/denser glow than standard, shifted toward white | Looks heavier, more substantial |
| Lock | Octagonal or rectangle with visible keyhole/lock glyph | Amber/gold with locked-state indicator | Glyph unlocks/dissolves when conditions met |
| Regen | Circular or organic shape | Green-tinted glow | Pulsing animation — the cell visibly breathes/regenerates |

### Cell Damage States
All cells communicate their health visually — no HP bars, no numbers:
- **Full health**: Clean, bright, fully formed shape
- **Damaged**: Cracks/fractures appear in the glow. Brightness dims. Shape destabilizes slightly.
- **Near death**: Heavy fracturing, flickering glow, shape barely holding together
- **Destroyed**: Adaptive death effect (see `effects-particles.md` — scales with context)

### Orbiting Shields (on Regen cells)
Shield cells that orbit a parent cell should be visually distinct from the parent:
- Smaller, brighter, clearly separate entities
- Visible orbit path (faint ring or trail)
- Shield color distinct from the parent cell

### Chip Effect Visual Modifiers on Cells
Certain chip effects change how cells look:
- **Powder Keg**: Cell gains a volatile/unstable visual — flickering, sparking, looks like it's about to explode
- **Other effects**: Each effect that modifies cell behavior should add a visible indicator layered on the cell's base identity

These modifiers layer additively — a Tough cell with Powder Keg looks like a reinforced cell that's also sparking and volatile.

### Data-Driven Composition
Cell visuals are composed from RON-defined enum values — shape, color, state handling, modifiers. New cell types can be created by combining visual building blocks. See `data-driven-graphics.md`.

## Walls (Playfield Boundaries)

Walls are always present but should never compete with gameplay elements for attention.

### Visual Design
- **Form**: Thin glowing border line along each playfield edge
- **Base state**: Very subtle glow (background grid brightness range, <0.3 HDR)
- **On bolt impact**: Brief pulse/flash at the impact point that travels a short distance along the wall, then fades. The wall "registers" the hit.
- **Color**: Follows the temperature palette — cool-tinted early, warm-tinted late
- **Bottom edge**: When shield is active, the bottom wall displays a visible energy barrier (see `feedback-juice.md`)

## Playfield Background

The background is the largest visual surface and must support (not compete with) all foreground effects.

### Visual Design
- **Form**: Flat 2D grid — straight horizontal and vertical lines forming a regular grid
- **Color**: Very dim — barely visible against the void. Enough to give spatial context, not enough to draw the eye.
- **Animation**: Occasional energy "sprites" (small bright points) travel along grid lines. Very subtle — one or two visible at any time, moving at a slow constant speed. They provide a sense that the grid is alive/active without being distracting.
- **Temperature**: Grid tint follows the run's temperature palette (cool early, warm late)

### Interaction with Effects
The grid is a **passive reference surface**. It does not warp, bend, or react to game events directly. Instead, full-screen shader effects (gravity wells, shockwaves, distortion) warp the rendered screen, which incidentally warps the grid as viewed. This means:
- Gravity well shader warps the screen in its radius — the grid appears bent through the distortion
- Shockwave shader creates a radial distortion — the grid ripples as the wave passes
- The grid itself is always rendered flat and straight; distortion is a rendering effect, not a grid modification
