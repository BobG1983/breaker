# Phase 5 Revised Architecture

## Core Change

The monolithic `rantzsoft_vfx` crate is replaced by two focused crates plus game-side visual systems. No recipe system. No god-messages. Builders/spawn systems attach visuals directly.

## Crate Layout

### rantzsoft_particles2d

**Scope**: CPU particle engine. Game-agnostic. Zero game vocabulary.

| Provides | Details |
|----------|---------|
| `Particle` component | velocity, lifetime, rotation_speed, gravity, color, size |
| `ParticleEmitter` component | emission mode, spawn params |
| `EmissionMode` enum | `Continuous { rate }`, `Burst { count }`, `OneShot { count }` |
| `SpawnParams` struct | lifetime range, velocity shape (Radial/Cone/Directional), speed range, size range, color, HDR brightness, gravity, rotation speed range |
| `ParticleMaterial` | Custom Material2d with additive blend via specialize() |
| Update system | Apply gravity, advance position, rotate, fade alpha over lifetime |
| Cleanup system | Despawn on lifetime expiry |
| Soft cap | 8192 concurrent particles, emitters skip spawning if cap reached |
| Common presets | `RadialBurst { count, speed_range, lifetime_range, ... }`, `DirectionalBurst { direction, spread, ... }`, `ContinuousEmitter { rate, ... }` — generic, configurable, no game vocabulary |
| Plugin | `RantzParticles2dPlugin` — registers systems, nothing else |

**Does NOT own**: game-specific burst names (SparkBurst, ShardBurst — those are game presets using the crate's API), particle mesh shapes (game provides), game-specific colors.

### rantzsoft_postprocess

**Scope**: FullscreenMaterial post-processing infrastructure + common shader effects. Game-agnostic.

| Provides | Details |
|----------|---------|
| FullscreenMaterial helpers | ViewTarget ping-pong, render graph node ordering, material registration |
| Screen flash shader | `flash.wgsl` — additive flash overlay, color + intensity + duration |
| Radial distortion shader | `distortion.wgsl` — fixed-array source distortion (shockwaves, gravity wells) |
| Chromatic aberration shader | `chromatic_aberration.wgsl` — RGB channel offset |
| Vignette shader | `vignette.wgsl` — edge darkening with radius/intensity |
| Desaturation shader | `desaturation.wgsl` — per-pixel saturation reduction |
| CRT overlay shader | `crt.wgsl` — scanline + curvature + noise |
| Trigger messages | `TriggerScreenFlash`, `TriggerRadialDistortion`, `TriggerChromaticAberration`, etc. |
| Config resource | `PostProcessConfig` — per-effect enable/disable, intensity defaults |
| Plugin | `RantzPostProcessPlugin` — registers shaders, materials, systems, messages |

**Does NOT own**: game-specific shader effects, bloom tuning (Bevy's built-in Bloom), temperature-aware anything, game visual types.

### breaker-game — `visuals/` domain (top-level, peer to bolt/, cells/, etc.)

**Scope**: Game-specific visual composition types, modifier system, temperature palette.

| Provides | Details |
|----------|---------|
| `Hue` enum | ~148 CSS colors + Custom(f32,f32,f32,f32). Used in RON for all color references. |
| `Shape` enum | Rectangle, RoundedRectangle, Hexagon, Octagon, Circle, Diamond, Shield, Angular, Crystalline, Custom. SDF selection. |
| `Aura` enum | ShieldShimmer, TimeDistortion, PrismaticSplit — with params. Single AuraMaterial. |
| `Trail` enum | ShieldEnergy, Afterimage, PrismaticSplit — with params. Trail entities track source. |
| `GlowParams` struct | core_brightness, halo_radius, halo_falloff, bloom |
| `EntityVisualConfig` struct | shape, color, glow, aura, trail — composed in RON |
| `VisualModifier` enum | 12 variants for runtime visual changes from chip effects |
| `ModifierStack` component | Stacked modifiers with diminishing returns (visual-only) |
| `RunTemperature` resource | 0.0–1.0 cool→hot, drives palette shifts |
| SDF entity shader | `entity_glow.wgsl` — the core entity rendering shader |
| Additive blend material | Base material with specialize() for additive blending |
| Glitch text shader | `glitch_text.wgsl` — highlight moment typography |
| Holographic shader | `holographic.wgsl` — evolution chip cards |

### breaker-game — per-domain visual systems

**Scope**: Each domain's builder/spawn system directly attaches visual components. No messages needed.

| Domain | Visual responsibility |
|--------|---------------------|
| `bolt/` | Builder attaches shape, glow, trail based on BoltDefinition's rendering block |
| `breaker/` | Builder attaches shape, glow, aura, trail based on BreakerDefinition's rendering block |
| `cells/` | Builder attaches shape, glow based on CellTypeDefinition's rendering block |
| `walls/` | Builder attaches glow, shield barrier shader (for shield walls) |
| `effect/` | Effect fire() functions trigger particles, screen effects, entity modifications directly |
| `state/run/node/` | AnimateIn plays cell slam-in animations, bolt birthing |
| `state/run/` | Temperature palette updates on node transition |

## What's Eliminated

| Old concept | Replacement |
|-------------|-------------|
| `rantzsoft_vfx` monolithic crate | Two focused crates + game-side code |
| Recipe system (RON-driven VFX sequences) | Direct Rust functions per VFX effect |
| `AttachVisuals` god-message | Builders/spawn systems attach directly |
| `ExecuteRecipe` message | Direct function calls or system triggers |
| `RecipeStore` resource | Not needed — no recipes |
| VFX-crate-owned game types (Shape, Aura, Trail in rantzsoft) | Game-owned types in breaker-game |

## Communication Pattern (Simplified)

| Direction | Mechanism |
|-----------|-----------|
| Gameplay → Post-processing | Trigger messages (`TriggerScreenFlash`, `TriggerRadialDistortion`, etc.) |
| Gameplay → Particles | Spawn `ParticleEmitter` entities directly |
| Gameplay → Entity visuals | Builders attach visual components at spawn time |
| Chip effects → Entity visuals | `SetModifier` / `AddModifier` / `RemoveModifier` — game-side messages in `visuals/` |
| Node progression → Palette | System reads `RunTemperature`, updates material uniforms |

## Dependency Graph

```
rantzsoft_particles2d (standalone)
rantzsoft_postprocess (standalone, may depend on bevy render internals)
rantzsoft_spatial2d (existing)
rantzsoft_physics2d (existing, depends on spatial2d)

breaker-game depends on:
  rantzsoft_particles2d
  rantzsoft_postprocess
  rantzsoft_spatial2d
  rantzsoft_physics2d
  rantzsoft_defaults
```

No dependency between particles2d and postprocess — they're independent.

## visuals/ Domain Rules

- `visuals/` provides the visual vocabulary: types (Shape, Hue, Aura, Trail, GlowParams), shaders (entity_glow, additive material, glitch_text, holographic), and general visual systems (modifier computation, temperature palette application)
- `visuals/` has its own plugin: `VisualsPlugin`
- Domains (bolt/, cells/, breaker/, walls/) set up their entities' visuals via their builders — they choose which Shape, Hue, Aura, Trail to use based on their definitions
- `visuals/` does NOT know about bolt, breaker, cell, or wall — it only knows about its own types
- Same relationship as `effect/`: effect domain provides infrastructure, other domains define their effects in RON

## Open Questions (Not Blocking Phase 5)

- PlayfieldConfig may belong under node/ or state/run/node/ rather than shared/ — revisit after state lifecycle refactor settles
- Whether node-related gameplay belongs under state/ (routing/ui/lifecycle) or stays as a separate domain — deferred
