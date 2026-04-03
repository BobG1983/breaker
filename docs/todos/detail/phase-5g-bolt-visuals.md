# 5g: Bolt Visuals

**Goal**: Transform the bolt from a flat colored circle into an SDF energy orb with glow, wake trail, and modifier-driven visual state.

**Prerequisite**: Bolt definitions spec (`.claude/specs/bolt-definitions-code.md`) must be implemented first. See `docs/architecture/rendering/bolt-graphics-migration.md` for the migration delta.

Architecture: `docs/architecture/rendering/entity_visuals.md`, `docs/architecture/rendering/bolt-graphics-migration.md`

## What to Build

### 1. Bolt Entity Rendering via AttachVisuals

Replace `Mesh2d`/`MeshMaterial2d` in `spawn_bolt` with `AttachVisuals` message:
- `EntityVisualConfig { shape: Circle, color: [from bolt RON], glow: [from bolt RON], trail: [from bolt RON] }`
- Crate's AttachVisuals handler creates SDF quad with `entity_glow.wgsl`, attaches trail entity
- Remove `ResMut<Assets<Mesh>>` and `ResMut<Assets<ColorMaterial>>` from spawn systems

### 2. Bolt Dynamic State via Modifiers

New `sync_bolt_visual_modifiers` system in bolt/ domain sends `SetModifier` each FixedUpdate:
- Speed → `TrailLength(speed_fraction * 2.0)`, source: `"bolt_speed"`
- Piercing → `SpikeCount(piercing_count)`, source: `"bolt_piercing"`
- Serving → `CoreBrightness(0.7)`, source: `"bolt_serving"` (dimmer while hovering)

Chip effects send `AddModifier`/`RemoveModifier` in their fire/reverse functions (speed boost, damage boost, size boost, etc.).

### 3. Bolt Event VFX via Recipes

Bolt RON has recipe names for event VFX:
- `spawn_recipe` — fired on bolt spawn (brief energy ring + flash)
- `death_recipe` — fired on bolt lost (exit streak)
- `expiry_recipe` — fired on lifespan expiry (inward implosion)

Game sends `ExecuteRecipe` at event time. Recipes authored in `assets/recipes/`.

### 4. ExtraBolt Distinction

Extra bolts (from multi-bolt effects) also get `AttachVisuals` from their `BoltDefinitionRef`. Visual distinction via modifiers:
- `AddModifier(GlowIntensity(0.7))` — slightly dimmer
- `AddModifier(TrailLength(0.6))` — shorter trail

### 5. PhantomBolt Visual

Phantom bolts get modifiers for spectral appearance:
- `AddModifier(AlphaOscillation { min: 0.3, max: 0.8, frequency: 3.0 })`
- `AddModifier(AfterimageTrail(true))`
- Different color in the bolt definition RON

### 6. Bolt Serving (Hover) State

When `BoltServing` component is present:
- `SetModifier(CoreBrightness(0.7))` — dimmer
- No trail (trail doesn't spawn until launch, or use `SetModifier(TrailLength(0.0))`)
- Pulsing glow via recipe or modifier oscillation

### 7. Bolt Lifespan Indicator

For bolts with `BoltLifespan`:
- Below 30%: `SetModifier(CoreBrightness(fraction * 2.0))` — dims
- Below 15%: `SetModifier(AlphaOscillation { ... })` — flicker
- At expiry: `ExecuteRecipe` with `expiry_recipe`

## Dependencies

- **Requires**: Bolt definitions spec (implemented), 5c (crate), 5d (bloom/additive), 5e (particles for trail/sparks), 5f (types + AttachVisuals)
- **Independent of**: 5h, 5i, 5j

## Verification

- Bolt renders as SDF energy orb with glow and bloom
- Trail visible and scales with speed via modifier
- Piercing bolts have visible spikes
- ExtraBolt, PhantomBolt look distinct
- Serving bolt is dimmer with no trail
- Lifespan indicator dims and flickers
- Event VFX (spawn, death, expiry) fire correctly
- All existing tests pass
