# 5e: Breaker Visuals

## Summary

Replace the breaker's flat `Mesh2d`/`MeshMaterial2d` rectangle with the full visual identity system from `visuals/`. The breaker builder reads a `rendering` block from `BreakerDefinition` and directly attaches `Shape`, `Hue`, `GlowParams`, `Aura`, `Trail`, `EntityGlowMaterial`, and `ModifierStack` components at spawn time. Each archetype (Aegis, Chrono, Prism) gets a distinct visual identity: unique SDF shape, signature color, ambient aura, and movement trail. A new `sync_breaker_visual_modifiers` system drives runtime modifier updates based on breaker state (movement speed, dashing, settling). Chip effects apply visual modifiers via `AddModifier`/`RemoveModifier` messages.

No `AttachVisuals` god-message. No recipes. Builders attach visual components directly. The bump pop animation is migrated from Y-offset to a shader-driven `SquashStretch` modifier.

## Context

The current breaker renders as a flat `Rectangle::new(1.0, 1.0)` quad with a `ColorMaterial` in a single HDR color (see `breaker/builder/core/transitions.rs` lines 152-174). There is no per-archetype visual distinction — Aegis, Chrono, and Prism all look identical. The breaker builder's `Rendered` typestate currently holds `Handle<Mesh>` and `Handle<ColorMaterial>`, created from `color_rgb` in `BreakerDefinition`.

The revised architecture (see `docs/todos/detail/phase-5-rethink/architecture.md`) eliminates the `AttachVisuals` message pattern. Instead, each domain's builder directly attaches visual components from `visuals/` at spawn time. The `visuals/` domain (5e) provides the type vocabulary: `Shape`, `Hue`, `GlowParams`, `Aura`, `Trail`, `EntityGlowMaterial`, `ModifierStack`, `VisualModifier`. This phase wires those types into the breaker builder and adds breaker-specific rendering systems.

Design reference: `docs/design/graphics/gameplay-elements.md` — Breaker section.
Data-driven composition: `docs/design/graphics/data-driven-graphics.md` — Breaker Visual Composition.

## What to Build

### 1. BreakerDefinition Rendering Block

Add an `EntityVisualConfig` field to `BreakerDefinition`:

```
// In BreakerDefinition (breaker/definition.rs)
#[serde(default)]
pub rendering: EntityVisualConfig,
```

`EntityVisualConfig` (from `visuals/types/`) contains optional `shape`, `color`, `glow`, `aura`, `trail` fields. Each archetype's RON file defines its full visual identity:

| Archetype | Shape | Color | Glow | Aura | Trail |
|-----------|-------|-------|------|------|-------|
| Aegis | `Shield` | `CadetBlue` | core_brightness: 2.0, halo_radius: 0.4, halo_falloff: 3.0, bloom: 0.6 | `ShieldShimmer { pulse_speed: 1.5, intensity: 0.6, color: CadetBlue, radius: 1.4 }` | `ShieldEnergy { width: 4.0, fade_length: 80.0, color: CadetBlue, intensity: 1.2 }` |
| Chrono | `Angular` | `Gold` | core_brightness: 2.0, halo_radius: 0.35, halo_falloff: 3.5, bloom: 0.5 | `TimeDistortion { ripple_frequency: 2.0, echo_count: 3, intensity: 0.5, color: Gold }` | `Afterimage { copy_count: 4, fade_rate: 0.3, color: Gold, spacing: 12.0 }` |
| Prism | `Crystalline` | `MediumOrchid` | core_brightness: 2.5, halo_radius: 0.45, halo_falloff: 2.5, bloom: 0.7 | `PrismaticSplit { refraction_intensity: 0.6, spectral_spread: 0.4, color: MediumOrchid }` | `PrismaticSplit { spectral_spread: 0.5, fade_length: 60.0 }` |

The `rendering` field is optional with `serde(default)`. Breakers without a rendering block use `EntityVisualConfig::default()` (all `None` fields), which results in no visual components being attached (headless-safe).

### 2. Update RON Files

Add `rendering` blocks to each archetype's RON file:

- `assets/breakers/aegis.breaker.ron` — Shield shape, CadetBlue color, ShieldShimmer aura, ShieldEnergy trail
- `assets/breakers/chrono.breaker.ron` — Angular shape, Gold color, TimeDistortion aura, Afterimage trail
- `assets/breakers/prism.breaker.ron` — Crystalline shape, MediumOrchid color, PrismaticSplit aura, PrismaticSplit trail

### 3. Builder Visual Transition: Rendered Typestate

Replace the current `Rendered` typestate internals. Instead of holding `Handle<Mesh>` and `Handle<ColorMaterial>`, the `Rendered` typestate holds visual components derived from the `EntityVisualConfig`:

```
pub struct Rendered {
    pub(crate) entity_glow_material: Handle<EntityGlowMaterial>,
    pub(crate) mesh: Handle<Mesh>,
    pub(crate) shape: Option<Shape>,
    pub(crate) glow: Option<GlowParams>,
    pub(crate) aura: Option<Aura>,
    pub(crate) trail: Option<Trail>,
    pub(crate) color: Option<Hue>,
}
```

The `rendered()` method changes signature:

```
pub fn rendered(
    self,
    meshes: &mut Assets<Mesh>,
    entity_glow_materials: &mut Assets<EntityGlowMaterial>,
    rendering: &EntityVisualConfig,
) -> BreakerBuilder<D, Mv, Da, Sp, Bm, Rendered, R>
```

This method:
- Creates a unit quad `Mesh`
- Creates an `EntityGlowMaterial` with uniforms populated from `rendering.glow` (or sensible defaults)
- Sets `shape_type` uniform from `rendering.shape` via `shape_type_index()`
- Sets `color` uniform from `rendering.color` via `From<Hue> for LinearRgba`
- Stores the optional `aura` and `trail` for the `build()` terminal to attach

The `definition()` method stores the `EntityVisualConfig` in `OptionalBreakerData` so `rendered()` can read it. Alternatively, the caller passes the rendering config to `rendered()` explicitly.

### 4. Builder Terminal: Attach Visual Components

The `build()` methods for `Rendered` primary and extra breakers change to include visual components:

- `Mesh2d(mesh)` + `MeshMaterial2d(entity_glow_material)` — replaces old `ColorMaterial`
- `ModifierStack::default()` — enables runtime visual modification
- `Shape` component (if `rendering.shape` is `Some`)
- `GlowParams` component (if `rendering.glow` is `Some`)
- `Hue` component (if `rendering.color` is `Some`)

Aura and trail are NOT direct components on the breaker entity. They spawn as separate entities:
- **Aura**: child entity of the breaker, spawned via `commands` in `spawn()`. Has `AuraMaterial`, `Mesh2d` (slightly larger quad, ~1.4x radius), placed at z-offset -0.5 (behind parent). The `Aura` enum variant determines the `AuraUniforms::variant` uniform.
- **Trail**: top-level entity (NOT a child), stores `TrailSource(Entity)` pointing to the breaker entity. Self-despawns when the source entity despawns. Trail rendering technique depends on variant — see section 6.

### 5. Aura Rendering

Aura is a child mesh entity attached to the breaker. Single `AuraMaterial` with `variant` uniform selecting the algorithm in WGSL.

| Aura Variant | variant uniform | Visual |
|--------------|----------------|--------|
| `ShieldShimmer` | 0 | Pulsing energy field, shield-themed oscillation |
| `TimeDistortion` | 1 | Rippling time-echo rings, afterimage shimmer at rest |
| `PrismaticSplit` | 2 | Rainbow edge refractions, spectral scatter |

Aura spawn logic (in breaker `spawn()` or a dedicated `spawn_breaker_aura` system):
- Create child entity with `AuraMaterial`, `Mesh2d` (unit quad scaled ~1.4x breaker dimensions)
- Set `Transform` with z = -0.5 (renders behind breaker)
- Populate `AuraUniforms` from the `Aura` enum params (pulse_speed, intensity, color, radius, ripple_frequency, etc.)
- `aura.wgsl` shader animates the effect based on `variant` + time

The aura entity is a child of the breaker, so it moves, scales, and despawns with the breaker automatically.

### 6. Trail Rendering

Trail entities are top-level (NOT children of the breaker). Each trail type uses a different rendering technique:

| Trail Variant | Technique | Details |
|---------------|-----------|---------|
| `ShieldEnergy` | Mesh ribbon (`TriangleStrip`) | Ring buffer of positions sampled each frame. `TrailRibbonMaterial` with color + HDR intensity + alpha. Ribbon mesh rebuilt each frame from position history. |
| `Afterimage` | Pre-spawned entity pool | N ghost entities (e.g., 4 for Chrono), each with `EntityGlowMaterial` at reduced alpha. Repositioned each frame to recent positions from a position history ring buffer. Oldest ghosts have lowest alpha. |
| `PrismaticSplit` | 3 overlapping `ShieldEnergy` ribbons | Three ribbon entities with R, G, B tint offsets. Slight lateral offset for spectral split appearance. |

Trail systems (in `breaker/systems/` or `visuals/systems/`):
- `sample_breaker_trail_positions` — each frame, push breaker `Position2D` into the trail entity's ring buffer
- `update_trail_ribbon_mesh` — rebuild `ShieldEnergy` and `PrismaticSplit` ribbon meshes from ring buffer
- `update_trail_afterimage_pool` — reposition `Afterimage` ghost entities from ring buffer

Trail entities store `TrailSource(Entity)` and self-despawn when the source entity despawns (checked each frame).

### 7. Breaker Dynamic State via Modifiers

New `sync_breaker_visual_modifiers` system in `breaker/systems/`. Runs in `FixedUpdate` after movement systems. Reads breaker movement state and sends `SetModifier` messages to drive visual response:

| Breaker State | Modifier | Source Key | Value |
|---------------|----------|------------|-------|
| Moving (any nonzero velocity) | `TrailLength` | `"breaker_speed"` | `speed / max_speed` (0.0 to 1.0 fraction) |
| Dashing (`DashState::Active`) | `GlowIntensity` | `"breaker_dash"` | `1.5` |
| Dashing | `TrailLength` | `"breaker_dash_trail"` | `2.0` (long trail during dash) |
| Settling (`DashState::Settling`) | `GlowIntensity` | `"breaker_settle"` | `1.2` (fading glow) |
| Idle (zero velocity, no dash) | (removes above modifiers) | — | — |

The system sends `SetModifier` each tick for active states and `RemoveModifier` when states end. `SetModifier` is absolute (replaces any existing modifier with the same source key). These are visuals-domain messages consumed by the modifier computation system (5j).

### 8. Chip Effect Visual Modifiers on Breaker

Chip effects that target the breaker apply visual modifiers via `AddModifier`/`RemoveModifier` messages in their `fire()`/`reverse()` functions:

| Chip Effect | Modifier(s) | Source Key |
|-------------|-------------|------------|
| Speed boost | `TrailLength(1.3)` + `GlowIntensity(1.2)` | `"speed_boost"` |
| Width boost | `ShapeScale(width_multiplier)` | `"width_boost"` |
| Bump force boost | `CoreBrightness(1.5)` | `"bump_force"` |

These modifiers stack with diminishing returns via the `ModifierStack` (computation in 5j). The visual change is additive on top of the base state modifiers from section 7.

### 9. Bump Pop Migration: Y-Offset to SquashStretch

Replace the current `BumpFeedbackState` Y-offset animation (`bump_visual/system.rs`) with a shader-driven approach:

**Current**: `animate_bump_visual` modifies `Position2D.y` with an eased pop curve.
**New**: `trigger_bump_visual` sends `SetModifier(SquashStretch { x_scale: 1.2, y_scale: 0.8 })` with source `"bump_pop"`. A timer-based system (reusing `BumpFeedbackState` or a new component) removes the modifier after 2-3 frames (~0.05s), letting the entity_glow shader handle the visual deformation without touching `Position2D`.

This eliminates the Position2D manipulation that can interact poorly with collision systems. The squash/stretch is purely visual (shader uniform), not spatial.

The `animate_bump_visual` system is retained but simplified: instead of modifying `Position2D`, it sets and clears the `SquashStretch` modifier based on the `BumpFeedbackState` timer. The easing curve for the deformation can follow the existing `BumpFeedback` params (rise_ease, fall_ease, peak_fraction).

### 10. Remove Old Visual Code

- Remove `Handle<Mesh>` and `Handle<ColorMaterial>` from the `Rendered` typestate
- Remove `ResMut<Assets<Mesh>>` and `ResMut<Assets<ColorMaterial>>` from `rendered()` parameters
- Remove `color_rgb` field from `BreakerDefinition` (replaced by `rendering.color`)
- Remove `color_rgb` from `OptionalBreakerData` and `.with_color()` method
- Remove `DEFAULT_COLOR_RGB` constant
- Remove `color_from_rgb` usage in breaker builder (if exclusive to breaker)
- Update `spawn_or_reuse_breaker` system to pass `EntityGlowMaterial` assets instead of `ColorMaterial`
- Update all test helpers that build breakers with `rendered()` to use the new signature

## Archetype Visual Identity Summary

| Archetype | SDF Shape | Signature Color | Aura Effect | Trail Effect | Character |
|-----------|-----------|-----------------|-------------|--------------|-----------|
| Aegis | Shield — wide, convex front face, protective silhouette | CadetBlue (blue/cyan) | ShieldShimmer — pulsing defensive energy field | ShieldEnergy — solid protective wake ribbon | Defensive, steady, resilient |
| Chrono | Angular — sleek, sharp geometric edges | Gold (amber/warm) | TimeDistortion — rippling time-echo rings, afterimage shimmer | Afterimage — fading ghost copies at recent positions | Fast, precise, time-bending |
| Prism | Crystalline — faceted, multi-angled, refractive | MediumOrchid (magenta/violet) | PrismaticSplit — rainbow edge refractions, spectral scatter | PrismaticSplit — trail separates into spectral RGB ribbons | Chaotic, refractive, multiplying |

## Breaker States (Visual Only)

| State | Visual Response |
|-------|----------------|
| Idle | Base appearance, ambient aura at rest intensity |
| Moving | Trail length scales with speed fraction, aura subtly intensifies in movement direction |
| Dashing | Full trail at maximum length, glow intensifies to 1.5x, archetype-specific dash visuals at peak |
| Settling (post-dash) | Trail fading, glow easing from 1.5x back toward 1.0x |
| Bump press | SquashStretch deformation (shader-only, 2-3 frames) |

## What NOT to Do

- Do NOT implement an `AttachVisuals` message — builders attach visual components directly at spawn time
- Do NOT implement a recipe system — there are no recipes in the revised architecture
- Do NOT implement `ExecuteRecipe` for bump grade VFX — that is 5k (bump VFX) which uses direct particle/flash functions
- Do NOT implement the modifier computation system that reads `ModifierStack` and updates material uniforms — that is 5j (dynamic visuals)
- Do NOT implement the trail update systems from scratch if they belong in `visuals/systems/` — coordinate with 5e's module structure
- Do NOT implement temperature palette application to the breaker — that is 5j
- Do NOT modify bolt, cell, or wall visual code — those are 5f, 5h, 5i respectively
- Do NOT add particle effects (spark bursts, shockwaves on bump) — those are 5k
- Do NOT implement screen flash or screen shake on bump — those are 5k (bump VFX)
- Do NOT put game vocabulary in `visuals/` domain code — `visuals/` does not know about breakers

## Module Structure Changes

```
breaker-game/src/breaker/
    definition.rs                    // + rendering: EntityVisualConfig field
    builder/
        core/
            types.rs                 // Rendered typestate: EntityGlowMaterial replaces ColorMaterial
            transitions.rs           // rendered() new signature, definition() stores rendering config
            terminal.rs              // build() attaches visual components, spawn() creates aura + trail
    systems/
        mod.rs                       // + sync_breaker_visual_modifiers, spawn_breaker_aura, trail systems
        sync_breaker_visual_modifiers/
            mod.rs
            system.rs                // reads breaker state, sends SetModifier/RemoveModifier
            tests/
                mod.rs
                movement_tests.rs    // speed fraction -> TrailLength modifier
                dash_tests.rs        // dash state -> GlowIntensity modifier
                idle_tests.rs        // idle -> modifiers removed
        bump_visual/
            system.rs                // migrated: SquashStretch modifier instead of Position2D offset
            tests.rs                 // updated: verify SquashStretch modifier, not Y-offset

breaker-game/src/breaker/components/
    trail_source.rs                  // TrailSource(Entity) component (or in visuals/components/)
```

Trail and aura systems may live in `visuals/systems/` rather than `breaker/systems/`, since they are generic (any entity with an Aura or Trail could use them). Decision: trail/aura update systems go in `visuals/systems/` because they operate on visual types only; breaker-specific spawning logic stays in `breaker/`.

## Dependencies

- **Requires**: 5e (visuals domain — Shape, Hue, GlowParams, Aura, Trail, EntityGlowMaterial, AuraMaterial, TrailRibbonMaterial, ModifierStack, EntityVisualConfig, VisualModifier types and materials)
- **Independent of**: 5f (bolt visuals), 5h (cell visuals), 5i (walls & background)
- **Feeds into**: 5j (dynamic visuals — modifier computation reads ModifierStack, applies to material uniforms), 5k (bump VFX — particle bursts and screen effects on bump grades)

## Verification

- All three archetypes render with distinct SDF shapes (Shield, Angular, Crystalline) via entity_glow shader
- Each archetype has its signature color rendered through EntityGlowMaterial
- Aura renders as a child entity behind the breaker with correct variant animation (ShieldShimmer / TimeDistortion / PrismaticSplit)
- Trail follows breaker movement and despawns when breaker entity despawns
- `sync_breaker_visual_modifiers` sends correct modifiers based on movement state (idle, moving, dashing, settling)
- Chip effect visual modifiers (speed boost, width boost, bump force) produce correct `AddModifier`/`RemoveModifier` messages
- Bump pop uses `SquashStretch` modifier instead of `Position2D` Y-offset
- `BreakerDefinition` RON files deserialize with `rendering` block correctly
- `BreakerDefinition` RON files without `rendering` block still parse (serde default)
- Builder `rendered()` creates `EntityGlowMaterial` instead of `ColorMaterial`
- Headless builders are unaffected (no visual components attached)
- Old `color_rgb` field and `DEFAULT_COLOR_RGB` are removed
- All existing breaker tests pass (updated for new builder signature)
- All existing workspace tests pass
- `cargo all-dclippy` clean
- `cargo all-dtest` clean

## Status
`[NEEDS DETAIL]` — blocked on 5e API design (how builders consume EntityVisualConfig, how aura/trail entities are spawned, how modifiers are sent)
