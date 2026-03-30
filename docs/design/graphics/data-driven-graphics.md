# Data-Driven Graphics

How visual properties are defined in RON data files so that new entities (cells, breakers, effects) can be created through data composition rather than code changes.

## Design Principle

Every visual aspect of a game entity should be definable through **enum composition in RON files**. A new cell type, breaker archetype, or effect variant should be creatable by combining existing visual building blocks — shape, color, aura, trail, state handling — without writing new rendering code.

New rendering code is only needed when a genuinely new visual primitive is required (a new shader, a new particle behavior, a new shape type). But combinations of existing primitives are pure data.

## Cell Visual Composition

A cell's visual identity is composed from independent enums:

```ron
// Example: a Tough cell with green tint and hexagonal shape
(
    cell_shape: Hexagon,
    cell_color: CoolGreen,
    damage_display: Fracture,    // how damage shows (cracks, fade, flicker)
    death_effect: Shatter,       // how destruction looks (dissolve, shatter, energy_release)
    // ... gameplay fields (hp, cell_type, etc.)
)
```

### Cell Shape Enum
Defines the geometric silhouette:
- `Rectangle` — standard rectangular cell
- `RoundedRectangle` — softer corners, still rectangular
- `Hexagon` — six-sided, reads as "tough/reinforced"
- `Octagon` — eight-sided, reads as "special/locked"
- `Circle` — round, reads as "organic/alive" (regen)
- `Diamond` — rotated square, reads as "valuable/rare"

### Cell Color Enum
Defines the base glow color (modified by temperature shift):
- `TemperatureDefault` — follows the run's current temperature palette
- `CoolBlue`, `CoolCyan`, `CoolGreen` — fixed cool colors
- `WarmAmber`, `WarmMagenta`, `WarmRed` — fixed warm colors
- `Gold` — special/reward cells
- `Neutral` — white/silver, no hue

### Damage Display Enum
Defines how damage accumulation is shown:
- `Fracture` — cracks appear and grow with damage
- `Fade` — brightness/opacity decreases with damage
- `Flicker` — cell flickers more frequently as health drops
- `Shrink` — cell physically shrinks with damage
- `ColorShift` — color shifts toward red/dim as health drops

### Death Effect Enum
Defines the destruction visual (context-adaptive — see `effects-particles.md`):
- `Dissolve` — clean fade with spark burst (single kill)
- `Shatter` — fracture into shards (combo context)
- `EnergyRelease` — expanding light ring (chain reaction context)
- `Custom(String)` — named custom effect for special cell types

## Breaker Visual Composition

A breaker archetype's visual identity is composed from:

```ron
// Example: Chrono archetype
(
    breaker_shape: Angular,
    color_accent: Amber,
    aura_type: TimeDistortion,
    trail_type: Afterimage,
    // ... gameplay fields (speed, width, dash params, etc.)
)
```

### Breaker Shape Enum
- `Shield` — wide, convex front face (Aegis)
- `Angular` — sleek, sharp geometric edges (Chrono)
- `Crystalline` — faceted, multi-angled, refractive (Prism)

### Color Accent Enum
The archetype's signature color (used for aura, trail, bump flash):
- `BlueCyan` — Aegis
- `Amber` — Chrono
- `Magenta` — Prism

### Aura Type Enum
The ambient effect around the breaker at rest:
- `ShieldShimmer` — defensive energy field
- `TimeDistortion` — rippling time-echo effect
- `PrismaticSplit` — rainbow edge refractions

### Trail Type Enum
The visual left behind during dash:
- `ShieldEnergy` — solid, protective-feeling wake
- `Afterimage` — fading copies showing recent positions
- `PrismaticSplit` — trail separates into spectral colors

## Effect Visual Modifiers

Chip effects that modify entity appearance define their visual changes as data:

```ron
// Example: Speed Boost chip visual modifier
(
    target: Bolt,
    visual_modifier: (
        trail_length_multiplier: 1.5,
        glow_intensity_multiplier: 1.2,
        color_shift: Some(Warmer),
        particle_emitter: None,
        shape_modifier: None,
    ),
)
```

### Visual Modifier Fields
- `trail_length_multiplier: f32` — scales trail/wake length
- `glow_intensity_multiplier: f32` — scales glow brightness
- `color_shift: Option<ColorShift>` — shifts hue (Warmer, Cooler, Custom(Color))
- `particle_emitter: Option<ParticleEmitter>` — adds a persistent particle source
- `shape_modifier: Option<ShapeModifier>` — changes the shape (Spikier, Smoother, Larger)
- `additional_effect: Option<String>` — named additional effect (e.g., "dripping_energy", "electric_crackle")

Modifiers **stack with diminishing returns on visuals**. Each additional stack of the same modifier contributes less visual scaling than the previous:

- 1st stack: full multiplier (e.g., 1.5x trail length)
- 2nd stack: reduced multiplier (e.g., 1.35x)
- 3rd stack: further reduced (e.g., 1.2x)
- 4th+ stacks: minimal additional visual scaling (e.g., 1.1x)

The exact diminishing curve is tunable per modifier type. The goal is to prevent late-run bolts from becoming screen-filling blurs while still making high-stack builds look visually distinct.

**IMPORTANT**: Diminishing returns apply ONLY to visual modifiers, NOT to gameplay effects. A bolt with 5 speed boost stacks still gets the full gameplay speed multiplier — it just doesn't render with a 7.5x trail length. The visual representation is an approximation of power, not a 1:1 mapping.

Different modifier types stack independently — a bolt with speed boost + damage boost has both the longer trail AND the color shift toward hot. The visual is the combination of all active modifiers, each with its own diminishing curve applied.

## Evolution Visual Definitions

Evolutions have **bespoke VFX** that cannot be composed from the standard modifier system. Each evolution references a unique named effect:

```ron
// Example: Nova Lance evolution
(
    name: "Nova Lance",
    evolution_vfx: "nova_lance_beam",
    // ... gameplay fields
)
```

The `evolution_vfx` field references a hardcoded visual effect implementation. Evolutions are rare enough (8 total) that bespoke VFX is justified — they are the visual reward for reaching the power ceiling.

## Adding New Visual Primitives

When a new visual capability is needed:
1. Add a new variant to the relevant enum (e.g., `CellShape::Pentagon`)
2. Implement the rendering for that variant in the shader/mesh system
3. Use it in RON data files

The enum definitions live in the game crate's component types. The rendering implementations live in the rendering/VFX systems. RON files reference enum variants by name.
