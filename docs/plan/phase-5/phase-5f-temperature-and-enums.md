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

### 6. Visual Identity Components — **DECISION REQUIRED**

Define the spawn-time visual identity components (part of the rendering communication pattern). Open question: bundled struct per entity type vs. separate components per visual property.

**Option A — Bundled struct:**
- `VisualIdentity { shape, color, aura_type, trail_type, damage_display, death_effect }` — one component, attached by owning domain at spawn
- Not all fields apply to all entities (breaker doesn't have damage_display; bolt doesn't have shape)

**Option B — Separate components:**
- `Shape(Shape)`, `Color(Color)`, `AuraType(AuraType)`, `DamageDisplay(DamageDisplay)`, etc. — each attached individually
- More query-friendly, allows partial updates, entities only get the components that apply to them

Both options: owning domain attaches at spawn, rendering/ reads via queries. Decision to be resolved in 5a (architecture doc) or 5b.

## What NOT to Do

- Do NOT implement the rendering of these enums (that's 5g-5j)
- Do NOT implement visual modifier stacking logic (that's 5n)
- Just define the types, integrate with RON, and attach identity components at spawn

## Dependencies

- **Requires**: 5c (rendering/ domain exists, visual identity component pattern established)
- **Enhanced by**: 5b (design decisions may refine some enum variants)

## Catalog Elements Addressed

From `catalog/systems.md` (Data-Driven Composition Enums):
- Shape enum: NONE → implemented (replaces CellShape + BreakerShape)
- Color enum: NONE → implemented (replaces CellColor + ColorAccent)
- DamageDisplay enum: NONE → implemented
- DeathEffect enum: NONE → implemented
- BreakerShape enum: NONE → merged into Shape
- ColorAccent enum: NONE → merged into Color
- AuraType enum: NONE → implemented
- TrailType enum: NONE → implemented
- VisualModifier system: NONE → types defined (logic in 5n)

From `catalog/systems.md` (Temperature):
- Run temperature resource: NONE → implemented
- Temperature application: NONE → resource built (per-element application in 5g-5j)

## Verification

- All enums deserialize from RON correctly
- Existing RON files load with new fields (defaults for backwards compat)
- Visual identity components are attached at entity spawn
- Temperature resource updates on node transitions
- All existing tests pass, game plays normally
