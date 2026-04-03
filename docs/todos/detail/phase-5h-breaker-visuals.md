# 5h: Breaker Visuals

**Goal**: Give each breaker archetype a fully distinct visual identity — different SDF shape, color, aura, and trail — via `AttachVisuals` and modifier messages.

Architecture: `docs/architecture/rendering/entity_visuals.md`, `docs/architecture/rendering/types.md`

## What to Build

### 1. Per-Archetype AttachVisuals

Replace flat rectangle with `AttachVisuals` from breaker RON rendering block:

| Archetype | Shape | Color | Aura | Trail |
|-----------|-------|-------|------|-------|
| Aegis | Shield | CadetBlue | ShieldShimmer | ShieldEnergy |
| Chrono | Angular | Gold | TimeDistortion | Afterimage |
| Prism | Crystalline | MediumOrchid | PrismaticSplit | PrismaticSplit |

Each uses `entity_glow.wgsl` SDF shader with the appropriate shape_type uniform.

### 2. Aura Rendering

Single `AuraMaterial` with variant uniform (ShieldShimmer=0, TimeDistortion=1, PrismaticSplit=2). Aura is a child mesh entity of the breaker, slightly larger (1.4x radius), placed at z-0.5 (behind parent). See `docs/architecture/rendering/types.md` — Aura Rendering Technique.

### 3. Trail Rendering

Trail entities are top-level (NOT children). Each trail type:
- ShieldEnergy: mesh ribbon (TriangleStrip), ring buffer of positions
- Afterimage: pre-spawned entity pool, repositioned each frame
- PrismaticSplit: 3 overlapping ShieldEnergy ribbons with RGB tint

Trail entities store `TrailSource(Entity)` and self-despawn when source despawns.

### 4. Breaker Dynamic State via Modifiers

New system in breaker/ sends `SetModifier` each FixedUpdate:
- Movement → `TrailLength(speed_fraction)`, source: `"breaker_speed"`
- Dashing → `GlowIntensity(1.5)`, source: `"breaker_dash"`

Chip effects (speed boost, width boost, bump force) send `AddModifier`/`RemoveModifier`:
- Speed boost: `TrailLength(1.3)` + `GlowIntensity(1.2)`, source: `"speed_boost"`
- Width boost: `ShapeScale(width_multiplier)`, source: `"width_boost"`
- Bump force: `CoreBrightness(1.5)`, source: `"bump_force"`

### 5. Bump Grade Recipes

Breaker RON has per-grade recipes:
- `perfect_bump_recipe` — gold flash + spark burst + micro-shake
- `early_bump_recipe` — dim archetype flash + few sparks
- `late_bump_recipe` — minimal flash
- Whiff: no recipe fired (silence IS feedback)

Game sends `ExecuteRecipe` on `BumpPerformed { grade }`.

### 6. Bump Pop

Replace current Y-offset pop with:
- `SquashStretch { x_scale: 1.2, y_scale: 0.8 }` modifier (2-3 frames, shader-only)
- Brief archetype-color flash at contact point via recipe

## Dependencies

- **Requires**: 5c (crate), 5d (bloom/additive), 5e (particles), 5f (types + AttachVisuals)
- **Independent of**: 5g, 5i, 5j

## Verification

- All three archetypes have distinct shapes, colors, auras, and trails
- Aura renders behind breaker with correct animation per variant
- Trail follows breaker movement and despawns on breaker despawn
- Bump grade produces correct recipe VFX per grade
- Speed/width/force modifiers produce visible changes
- All existing tests pass
