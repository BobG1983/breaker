# Error Handling & Edge Cases

Default posture: **warn and skip**. The VFX crate never panics on invalid input from the game. Missing visuals are always better than a crash. Log `warn!()` for unexpected states so they surface during development.

## Edge Case Catalog

### Recipe Errors

| Situation | Behavior |
|-----------|----------|
| `ExecuteRecipe` references a recipe name not in `RecipeStore` | `warn!("Recipe '{}' not found, skipping", name)`. No VFX spawned. No lifecycle messages emitted. |
| Recipe has `AfterPhase(5)` but only 3 phases exist | `warn!("Phase trigger AfterPhase({}) references nonexistent phase in recipe '{}'", idx, name)`. Phase is never triggered. Other phases execute normally. |
| Recipe has `EntityRef::Source` but `ExecuteRecipe.source` is `None` | `warn!("Recipe '{}' step requires Source entity but none provided", name)`. Skip the anchored step. Non-anchored steps in the same phase still execute. |
| Recipe has `EntityRef::Target` but `ExecuteRecipe.target` is `None` | Same as above — skip the step, warn. |
| Recipe has screen effect steps but `ExecuteRecipe.camera` is `None` | `warn!("Recipe '{}' has screen effects but no camera provided, skipping screen steps", name)`. Spatial primitives still fire. |
| `CancelRecipe` targets a non-existent `RecipeExecution` entity | Silent no-op. The recipe may have already completed. |

### Entity Visual Errors

| Situation | Behavior |
|-----------|----------|
| `AttachVisuals` sent for an entity that already has visuals | `warn!("Entity {:?} already has visuals attached, skipping", entity)`. First attachment wins. To change visuals, despawn and re-spawn. |
| `AttachVisuals` sent for an entity that no longer exists | Silent no-op. The entity may have despawned between message send and processing. |
| `Shape::Custom` with 0 vertices | `warn!("Custom shape with 0 vertices")`. Fall back to `Shape::Circle`. |
| `Shape::Custom` with > 16 vertices | `warn!("Custom shape has {} vertices, max is 16, truncating", n)`. Use first 16 vertices. |

### Modifier Errors

| Situation | Behavior |
|-----------|----------|
| `SetModifier` / `AddModifier` targets entity without `ModifierStack` | Silent skip. Entity may not have received `AttachVisuals` yet, or was never registered for modifier tracking. Not worth logging — game-side per-frame sends would flood logs. |
| `RemoveModifier` with source key that doesn't exist | Silent no-op. Effect may have already been reversed. |
| `SetModifier` with `duration: Some(0.0)` or negative | Treat as immediate removal on next cleanup tick. No warn — this is a valid "remove immediately" pattern. |

### Primitive Errors

| Situation | Behavior |
|-----------|----------|
| Particle soft cap (8192) reached | New emitters skip spawning with `debug!("Particle cap reached, skipping emitter")`. Existing particles continue normally. Resume spawning when count drops below cap. |
| `SpawnExpandingRing` with `max_radius <= 0.0` | `warn!("ExpandingRing with non-positive radius")`. Skip spawn. |
| `SpawnBeam` with zero-length direction | `warn!("Beam with zero direction")`. Skip spawn. |
| `SpawnElectricArc` with `start == end` | `warn!("ElectricArc with zero length")`. Skip spawn. |

### Anchored Primitive Edge Cases

| Situation | Behavior |
|-----------|----------|
| Single-entity anchor (AnchoredRing, AnchoredDistortion, AnchoredGlowMotes): tracked entity despawns | Despawn the anchored primitive immediately. One-frame artifact acceptable. |
| Two-entity anchor (AnchoredBeam, AnchoredArc): one entity despawns, other still alive | **Retract** endpoint to surviving entity over ~0.1s (lerp to source position), then despawn. See [rantzsoft_vfx.md](rantzsoft_vfx.md) — Anchored Primitive Retraction. |
| Two-entity anchor: both entities despawn same frame | Despawn immediately. No retraction. |
| Anchored primitive source entity exists but is at `Vec2::ZERO` | Render normally. Zero is a valid position. |

### Screen Effect Edge Cases

| Situation | Behavior |
|-----------|----------|
| Multiple `TriggerScreenFlash` in same frame | Latest wins (overrides). Only one flash active. |
| `TriggerScreenShake` with shake_multiplier = 0.0 in VfxConfig | Silent no-op (shake disabled by user). |
| Distortion buffer full (16 sources) | Replace oldest source. `debug!("Distortion buffer full, replacing oldest source")`. |

### Trail Edge Cases

| Situation | Behavior |
|-----------|----------|
| Trail entity's source entity despawns | Trail despawns on next cleanup tick (TrailSource polling). |
| Trail entity has 0 samples in ring buffer | Don't render (no mesh update). Will populate next frame. |
| TrailLength modifier < 0.0 | Clamp to 0.0 (invisible trail, buffer still sampling). |

## Debug vs Release

All `warn!()` calls log in both debug and release builds. `debug!()` calls are compile-time stripped in release. No `debug_assert!()` is used — the warn-and-skip policy applies uniformly.
