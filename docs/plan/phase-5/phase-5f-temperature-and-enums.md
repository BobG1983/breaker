# 5f: Temperature Palette & Data-Driven Enums

**Goal**: Build the temperature palette system that shifts colors across a run, and implement all visual composition enums that entity visual steps (5g-5j) will consume.

## What to Build

### 1. Temperature Palette System

A rendering/ resource tracking the run's visual temperature:

- **RunTemperature** resource: `f32` from 0.0 (cool) to 1.0 (hot), derived from node progression
- **Temperature palette lookup**: Given a temperature value, returns the current color set:
  - Nodes 1-3 (0.0-0.3): Cool — deep blues, teals, cyans
  - Nodes 4-6 (0.3-0.6): Transitional — cyan→purple, violet, early magenta
  - Nodes 7-9 (0.6-0.9): Hot — magentas, ambers, warm whites
  - Final/boss (0.9-1.0): White-hot — whites and golds with magenta/amber accents
- **Temperature application targets**: Grid tint, default cell glow, particle base color, wall border tint, ambient bloom color
- **Temperature-exempt elements**: Bolt core, breaker archetype colors, rarity tier colors, UI elements

System that reads `RunState.node_index` and updates `RunTemperature` on node transitions.

### 2. Cell Visual Composition Enums

All visual composition enums are defined in rendering/ as generic rendering primitives — not prefixed with entity names. Any entity can use any variant. Owning domains (cells/, breaker/, etc.) reference these types in their RON data and attach them as components at spawn.

- **Shape**: Rectangle, RoundedRectangle, Hexagon, Octagon, Circle, Diamond, Shield, Angular, Crystalline
- **Color**: TemperatureDefault, CoolBlue, CoolCyan, CoolGreen, WarmAmber, WarmMagenta, WarmRed, Gold, Neutral, BlueCyan, Amber, Magenta
- **DamageDisplay**: Fracture, Fade, Flicker, Shrink, ColorShift
- **DeathEffect**: Dissolve, Shatter, EnergyRelease, Custom(String)
- **AuraType**: ShieldShimmer, TimeDistortion, PrismaticSplit
- **TrailType**: ShieldEnergy, Afterimage, PrismaticSplit

Each enum derives `Deserialize` for RON integration. Cell definition RON files and breaker archetype RON files gain these fields.

### 4. Visual Modifier Enums

Defined in rendering/ (cross-cutting concern):

- **ColorShift**: Warmer, Cooler, Custom(Color)
- **ShapeModifier**: Spikier, Smoother, Larger
- **VisualModifier** struct: trail_length_multiplier, glow_intensity_multiplier, color_shift, particle_emitter, shape_modifier

These are the building blocks for 5n (visual modifier system).

### 5. RON Integration

Update existing RON data files with the new enum fields:
- Cell definition RON files: add cell_shape, cell_color, damage_display, death_effect
- Breaker archetype RON files: add breaker_shape, color_accent, aura_type, trail_type
- Use sensible defaults matching current placeholder visuals

### 6. Visual Identity Components (Separate Components)

Each visual property is its own component. Entities only get the ones that apply:
- `Shape(Shape)`, `Color(Color)`, `AuraType(AuraType)`, `TrailType(TrailType)`, `DamageDisplay(DamageDisplay)`, `DeathEffect(DeathEffect)`
- Cell gets: Shape + Color + DamageDisplay + DeathEffect
- Breaker gets: Shape + Color + AuraType + TrailType
- Bolt gets: Color (mostly state-driven)

Owning domain attaches at spawn from RON data. rendering/ reads via queries.

## What NOT to Do

- Do NOT implement the rendering of these enums (that's 5g-5j)
- Do NOT implement visual modifier stacking logic (that's 5n)
- Just define the types, integrate with RON, and attach identity components at spawn

## Dependencies

- **Requires**: 5c (rendering/ domain exists, visual identity component pattern established)
- **Enhanced by**: 5b (design decisions may refine some enum variants)

## What This Step Builds

- RunTemperature resource + system that updates on node transitions
- Generic visual enums in rendering/: Shape, Color, DamageDisplay, DeathEffect, AuraType, TrailType
- VisualModifier types (trail_length_multiplier, glow_intensity_multiplier, color_shift, etc.)
- Separate visual identity components (Shape, Color, AuraType, etc.) attached at spawn
- RON integration for cell definitions and breaker archetypes

## Verification

- All enums deserialize from RON correctly
- Existing RON files load with new fields (defaults for backwards compat)
- Visual identity components are attached at entity spawn
- Temperature resource updates on node transitions
- All existing tests pass, game plays normally
