# 5m: Combat Effect VFX

**Goal**: Author RON visual recipes for all triggered combat effects. Currently effects have gameplay logic but no visuals.

Architecture: `docs/architecture/rendering/recipes.md`, `docs/architecture/rendering/composition.md`

## What to Build

Each combat effect gets a RON recipe in `assets/recipes/` using the crate's primitive steps. Some effects with runtime-variable params use direct primitive messages (code-composed).

### Recipe-Composed Effects

| Effect | Primitives in Recipe | Notes |
|--------|---------------------|-------|
| **Shockwave** | ExpandingRing + RadialDistortion + ScreenShake + SparkBurst | Higher stacks = larger radius, brighter bloom, stronger distortion |
| **Explode** | ExpandingRing + SparkBurst + ScreenShake + ScreenFlash | Central HDR flash, fastest effect (~0.15s) |
| **Pulse** | ExpandingDisc + RadialDistortion + ScreenShake | Filled circle, faster than shockwave (~0.2s) |
| **Piercing Beam** | Beam (Forward direction) | Appears on pierce, lingers ~0.1s as fading afterimage |
| **Random (Flux)** | SparkBurst (multi-colored) | Brief prismatic flash, then selected effect's recipe fires |
| **Quick Stop** | SquashStretch modifier + SparkBurst | Brief compression + sparks from breaker leading edge |
| **Bump Force** | ExpandingRing (compact) + ScreenFlash | Concentrated impact, not expanding like Shockwave |
| **Time Penalty** | Beam (to timer position) + VignettePulse | Red-orange energy line + timer glitch |

### Code-Composed Effects (runtime params)

| Effect | Why Code-Composed | What Game Sends |
|--------|-------------------|-----------------|
| **Chain Lightning** | Variable target chain, resolved at runtime | Multiple `SpawnElectricArc { start, end }` per chain link |
| **Gravity Well** | Persistent, tracks entity position | `ExecuteRecipe` with anchored steps: `AnchoredDistortion(entity: Source)` + `AnchoredGlowMotes(entity: Source, inward: true)` |
| **Tether Beam** | Variable bolt pair endpoints | `ExecuteRecipe` with `AnchoredBeam(entity_a: Source, entity_b: Target)` per bolt pair |
| **Attraction** | Variable target entity | `ExecuteRecipe` with `AnchoredArc(entity_a: Source, entity_b: Target)` |
| **Ramping Damage** | Counter-driven intensity | `ExecuteRecipe` with `AnchoredRing(entity: Source)` + `SetModifier(RotationSpeed(...))` scaling with hit count |

### Effect VFX Dispatch

Each chip effect's `fire()` sends the appropriate VFX:
- Effects with standalone VFX: `ExecuteRecipe { recipe: "shockwave_default", position, camera }`
- Effects with entity modifiers: `AddModifier` on target entity
- Effects with continuous VFX: `ExecuteRecipe` with anchored steps + source/target entities
- Each effect's `reverse()` sends `RemoveModifier` for any modifiers it added

No `VfxKind` enum. No per-effect rendering modules. No translation layer. Each effect domain function sends the appropriate messages directly.

## Dependencies

- **Requires**: 5c (crate), 5d (post-processing: distortion), 5e (particles), 5k (screen effects)
- **Enhanced by**: 5g (bolt visuals for halo tinting), 5h (breaker visuals for quick stop)

## Verification

- Each combat effect has visible VFX
- Shockwave distorts the screen
- Gravity well warps the scene persistently
- Chain lightning arcs between correct targets
- Tether beam tracks bolt pair positions
- All effects use correct primitive types
- All existing tests pass
