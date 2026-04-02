# 5i: Cell Visuals

**Goal**: Give each cell type a distinct SDF shape and color, implement damage state recipes, and add destruction effects — all via `AttachVisuals`, modifiers, and recipes.

Architecture: `docs/architecture/rendering/entity_visuals.md`, `docs/architecture/rendering/shaders.md`

## What to Build

### 1. Per-Type Cell AttachVisuals

Replace flat rectangles with `AttachVisuals` from cell RON rendering block:

| Cell Type | Shape | Color | Notes |
|-----------|-------|-------|-------|
| Standard | Rectangle | MediumSlateBlue | Clean, simple baseline |
| Tough | Hexagon | MediumSeaGreen | Brighter/denser glow |
| Lock | Octagon | Gold | Amber/gold tint |
| Regen | Circle | LimeGreen | Pulsing animation via modifier |

### 2. Damage State via Modifiers + Recipes

Cell damage is communicated through recipes defined in cell RON (`damage_recipe`, `hit_recipe`):
- Cell hit: `ExecuteRecipe` with `hit_recipe` at impact point, direction from bolt velocity
- Damage progression: modifiers driven by health fraction (`SetModifier(CoreBrightness(health_fraction))`)

### 3. Context-Adaptive Destruction

Cell RON has three death recipe fields (serde defaults for fallback):
- `death_recipe` — single kill
- `death_recipe_combo` — 2-4 rapid kills (falls back to death_recipe if None)
- `death_recipe_chain` — 5+ kills (falls back to death_recipe if None)

cells/ domain tracks recent kill rate and selects the appropriate recipe. See `docs/architecture/rendering/communication.md` — Cell Destruction VFX.

### 4. Destruction Shaders

Recipes reference destruction primitives in `rantzsoft_vfx`:
- `Disintegrate` — dissolve.wgsl (noise threshold ramp, burning edge glow)
- `Split` — shader clip-plane (two halves drift apart)
- `Fracture` — shader Voronoi (fragments scatter outward)

### 5. Special Cell VFX

- **Lock cell unlock**: Recipe with golden ShardBurst + ExpandingRing + ScreenFlash. Color transitions amber → true color via modifier.
- **Regen cell pulse**: Recipe with GlowMotes drifting upward on heal tick. Continuous GlowIntensity oscillation via modifier.
- **Shield cell orbit**: Smaller, brighter AttachVisuals. Visible orbit ring via AnchoredRing.
- **Powder Keg**: `AddModifier(AlphaOscillation { ... })` on the target cell for flickering/sparking.

## Dependencies

- **Requires**: 5c (crate), 5d (bloom), 5e (particles), 5f (types)
- **Independent of**: 5g, 5h, 5j

## Verification

- Each cell type has a distinct SDF shape
- Damage progression is visually clear (dimming, recipes)
- Destruction effects differ by context (single, combo, chain)
- Cell hit produces sparks in correct direction
- Lock unlock, regen pulse, Powder Keg VFX fire correctly
- All existing tests pass
