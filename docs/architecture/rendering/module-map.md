# rantzsoft_vfx Module Map

Concrete module tree for the `rantzsoft_vfx` crate. Synthesized from the rendering architecture docs — this is the authoritative source for where code lives within the crate.

## Crate Root

```
rantzsoft_vfx/
  Cargo.toml
  src/
    lib.rs                          // RantzVfxPlugin, headless(), prelude re-exports
    config.rs                       // VfxConfig resource
    layer.rs                        // VfxLayer trait

    types/
      mod.rs                        // pub use re-exports
      hue.rs                        // Hue enum (~148 CSS colors + Custom), From<Color>/From<Hue>
      shape.rs                      // Shape enum (Rectangle, Circle, Hexagon, etc.)
      glow.rs                       // GlowParams, HdrBrightness, BloomIntensity, EmissiveStrength
      aura.rs                       // Aura enum (ShieldShimmer, TimeDistortion, PrismaticSplit)
      trail.rs                      // Trail enum (ShieldEnergy, Afterimage, PrismaticSplit)
      direction.rs                  // Direction enum (N, S, E, W, Forward, Backward)
      entity_ref.rs                 // EntityRef enum (Source, Target)
      shake_tier.rs                 // ShakeTier enum (Micro, Small, Medium, Heavy)

    entity_visuals/
      mod.rs                        // pub use re-exports
      messages.rs                   // AttachVisuals message
      config.rs                     // EntityVisualConfig struct
      handler.rs                    // AttachVisuals handler system — mesh/material/aura/trail

    modifiers/
      mod.rs                        // pub use re-exports
      types.rs                      // VisualModifier enum, ModifierKind enum
      messages.rs                   // SetModifier, AddModifier, RemoveModifier messages
      stack.rs                      // ModifierStack component, ModifierConfig resource
      computation.rs                // per-frame modifier computation system
      cleanup.rs                    // timed modifier auto-removal

    recipes/
      mod.rs                        // pub use re-exports
      types.rs                      // Recipe, Phase, PhaseTrigger, RepeatConfig, PrimitiveStep
      store.rs                      // RecipeStore resource, loading pipeline
      messages.rs                   // ExecuteRecipe, CancelRecipe, RecipeStarted/Complete/Phase*
      dispatch.rs                   // recipe dispatch system (phase state machine, entity resolution)
      active_recipe.rs              // ActiveRecipe component, lifecycle management

    primitives/
      mod.rs                        // pub use re-exports
      messages.rs                   // SpawnExpandingRing, SpawnBeam, SpawnSparkBurst, etc.
      geometric/
        mod.rs
        expanding_ring.rs           // ExpandingRing + ExpandingDisc handler
        beam.rs                     // Beam handler (one-shot directional)
        energy_ring.rs              // EnergyRing handler (persistent orbital)
        glow_line.rs                // GlowLine handler (wall borders, barrier base)
      particle/
        mod.rs
        emitter.rs                  // ParticleEmitter component, EmissionMode, SpawnParams
        particle.rs                 // Particle component, update system
        material.rs                 // ParticleMaterial (additive blend via specialize)
        spark_burst.rs              // SparkBurst handler
        shard_burst.rs              // ShardBurst handler
        glow_motes.rs               // GlowMotes handler
        trail_burst.rs              // TrailBurst handler
      electric_arc.rs               // ElectricArc handler (segmented line mesh, NOT particles)
      anchored/
        mod.rs
        beam.rs                     // AnchoredBeam handler (entity-tracking, retraction)
        distortion.rs               // AnchoredDistortion handler
        ring.rs                     // AnchoredRing handler
        arc.rs                      // AnchoredArc handler (entity-tracking, retraction)
        glow_motes.rs               // AnchoredGlowMotes handler
      destruction/
        mod.rs
        disintegrate.rs             // Disintegrate handler (dissolve_threshold animation)
        split.rs                    // Split handler (clip-plane mesh halves)
        fracture.rs                 // Fracture handler (shader Voronoi)
      text/
        mod.rs
        glitch_text.rs              // GlitchText handler + GlitchMaterial

    trails/
      mod.rs                        // pub use re-exports
      ribbon.rs                     // ShieldEnergy trail: ring buffer, mesh ribbon, TrailRibbonMaterial
      afterimage.rs                 // Afterimage trail: pre-spawned sprite pool
      prismatic.rs                  // PrismaticSplit trail: 3 overlapping ribbons
      source.rs                     // TrailSource component, cleanup polling system

    auras/
      mod.rs                        // pub use re-exports
      material.rs                   // AuraMaterial (single type, variant-switched)
      handler.rs                    // aura child entity spawn system

    screen_effects/
      mod.rs                        // pub use re-exports
      messages.rs                   // TriggerScreenShake, TriggerScreenFlash, etc.
      shake.rs                      // ScreenShake system (camera Transform offset)
      flash.rs                      // ScreenFlash FullscreenMaterial
      distortion.rs                 // RadialDistortion FullscreenMaterial + distortion buffer
      chromatic.rs                  // ChromaticAberration FullscreenMaterial
      desaturation.rs               // Desaturation FullscreenMaterial
      vignette.rs                   // Vignette FullscreenMaterial
      crt.rs                        // CRT FullscreenMaterial
      collapse_rebuild.rs           // Collapse/Rebuild transition FullscreenMaterial
      slow_motion.rs                // DilationRamp resource, ramp system (Time<Real>)

    materials/
      mod.rs                        // pub use re-exports
      entity_glow.rs                // EntityGlowMaterial (Material2d, SDF-on-quad)
      grid.rs                       // GridMaterial (background grid)
      shield.rs                     // ShieldMaterial (hexagonal energy field)
      holographic.rs                // HolographicMaterial (chip card foil shimmer)
      trail_ribbon.rs               // TrailRibbonMaterial (alpha-weighted additive)

    sets.rs                         // VfxSet enum (system sets for ordering)

  assets/
    shaders/
      entity_glow.wgsl              // SDF entity shader (core + halo + dissolve + spikes)
      aura.wgsl                     // variant-switched aura shader
      trail_ribbon.wgsl             // mesh ribbon trail shader
      beam.wgsl                     // beam SDF-on-quad shader
      ring.wgsl                     // expanding ring / disc SDF shader
      glow_line.wgsl                // glowing line segment shader
      timer_wall.wgsl               // gauge glow bar shader
      grid.wgsl                     // background playfield grid shader
      shield.wgsl                   // shield barrier hexagonal energy field
      particle.wgsl                 // particle quad shader (additive)
      electric_arc.wgsl             // arc ribbon shader (additive)
      glitch_text.wgsl              // glitch text overlay shader
      holographic.wgsl              // holographic foil shimmer shader
      flash.wgsl                    // screen flash (FullscreenMaterial, shader-side additive)
      distortion.wgsl               // radial distortion (FullscreenMaterial)
      chromatic_aberration.wgsl     // chromatic aberration (FullscreenMaterial)
      desaturation.wgsl             // desaturation (FullscreenMaterial)
      vignette.wgsl                 // vignette (FullscreenMaterial)
      crt.wgsl                      // CRT overlay (FullscreenMaterial)
      collapse_rebuild.wgsl         // tile-based transition (FullscreenMaterial)
      noise.wgsl                    // simplex noise 2D/3D (utility)
      sdf.wgsl                      // 2D SDF primitives (utility)
      voronoi.wgsl                  // 2D Voronoi cell ID (utility)
```

## lib.rs Exports

```rust
pub mod types;
pub mod entity_visuals;
pub mod modifiers;
pub mod recipes;
pub mod primitives;
pub mod trails;
pub mod auras;
pub mod screen_effects;
pub mod materials;
pub mod sets;

mod config;
mod layer;

pub use config::VfxConfig;
pub use layer::VfxLayer;

pub struct RantzVfxPlugin { /* ... */ }
```

## Prelude

```rust
// rantzsoft_vfx/src/lib.rs or a prelude.rs
pub mod prelude {
    pub use crate::{
        RantzVfxPlugin, VfxConfig, VfxLayer,
        types::{Hue, Shape, Aura, Trail, GlowParams, HdrBrightness, BloomIntensity,
                Direction, EntityRef, ShakeTier},
        entity_visuals::{AttachVisuals, EntityVisualConfig},
        modifiers::{VisualModifier, ModifierKind, ModifierConfig,
                    SetModifier, AddModifier, RemoveModifier},
        recipes::{ExecuteRecipe, CancelRecipe, RecipeComplete, RecipeStarted,
                  PhaseStarted, PhaseComplete, Recipe, PrimitiveStep},
        primitives::messages::*,  // SpawnExpandingRing, TriggerScreenShake, etc.
        screen_effects::messages::*,
        sets::VfxSet,
    };
}
```

## Naming Conventions

- **Handler systems**: `handle_<message_name>` (e.g., `handle_attach_visuals`, `handle_execute_recipe`)
- **Update systems**: `update_<thing>` (e.g., `update_particles`, `update_anchored_beams`)
- **Cleanup systems**: `cleanup_<thing>` (e.g., `cleanup_expired_primitives`, `cleanup_trail_sources`)
- **Materials**: `<Name>Material` (e.g., `EntityGlowMaterial`, `AuraMaterial`, `TrailRibbonMaterial`)
- **Messages**: verb-noun (e.g., `AttachVisuals`, `ExecuteRecipe`, `TriggerScreenShake`, `SpawnExpandingRing`)
