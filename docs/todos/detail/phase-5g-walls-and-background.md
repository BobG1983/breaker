# 5g: Wall Visuals & Playfield Background

## Summary

Make playfield boundaries visible and the background alive. Wall entities (currently invisible collision bodies) get `EntityGlowMaterial` attachment via the wall builder. The shield barrier floor wall gets a dedicated energy field shader. A background grid shader provides a static spatial reference surface with temperature-tinted color. Wall impact flashes fire at bolt collision points. Void `ClearColor` updates to deep blue-black. No background energy sprites in this phase (deferred to a later polish pass -- they depend on `rantzsoft_particles2d` integration which is out of scope here).

This phase delivers rendering attachment and shaders only. Temperature palette application (the system that reads `RunTemperature` and updates wall/grid colors at runtime) is built in 5j (dynamic visuals).

## Context

The revised architecture (see `docs/todos/detail/phase-5-rethink/architecture.md`) eliminates the old `AttachVisuals` god-message. Builders attach visual components directly at spawn time. For walls, this means the wall builder's `visible()` transition gains an `EntityGlowMaterial`-based path (replacing the current `ColorMaterial` placeholder), and `spawn_walls` calls `.visible()` instead of `.invisible()`.

The shield barrier is a special case: it is a floor wall entity spawned by the shield chip effect (`breaker-game/src/effect/effects/shield/system.rs`), not by the regular `spawn_walls` system. It gets a dedicated `ShieldBarrierMaterial` with an animated hexagonal energy field pattern, procedural crack damage, and a shatter/fracture death effect.

## Dependencies

- **Requires**: 5e (visuals domain -- `EntityGlowMaterial`, `Hue`, `GlowParams`, `TemperaturePalette`, `RunTemperature` types, additive blend helper, SDF utilities)
- **Independent of**: 5f (bolt visuals), 5g (breaker visuals), 5h (cell visuals) -- can run in parallel with all three
- **Required by**: 5j (dynamic visuals -- temperature palette application system reads wall/grid materials)

## What to Build

### 1. Wall Builder EntityGlowMaterial Integration

**Current state**: `WallBuilder<S, Visible>` stores `Handle<Mesh>` + `Handle<ColorMaterial>`. The `visible()` transition on `WallBuilder<S, Invisible>` creates a `Rectangle` mesh and a `ColorMaterial`.

**Target**: Replace `ColorMaterial` with `EntityGlowMaterial` for wall rendering. Walls are SDF rectangles with subtle glow.

Changes to `breaker-game/src/walls/builder/core/types.rs`:
- `Visible` struct stores `Handle<Mesh>` + `Handle<EntityGlowMaterial>` (instead of `Handle<ColorMaterial>`)
- The mesh remains a unit quad (`Rectangle::new(1.0, 1.0)`) -- the SDF shader handles shape rendering via `half_extents` uniform

Changes to `breaker-game/src/walls/builder/core/transitions.rs`:
- `visible()` method takes `&mut Assets<Mesh>` + `&mut Assets<EntityGlowMaterial>` (instead of `&mut Assets<ColorMaterial>`)
- Constructs `EntityGlowUniforms` with:
  - `shape_type: 0` (Rectangle)
  - `core_brightness: 0.15` (very subtle -- walls must not compete with gameplay)
  - `halo_radius: 0.3`
  - `halo_falloff: 4.0`
  - `bloom_intensity: 0.1`
  - Color from resolved `color_rgb` or default white, converted to `Vec4` via `Hue`
  - `dissolve_threshold: 0.0` (no dissolve)
  - `alpha_override: 1.0`
  - All modifier fields zeroed (no spike, squash, rotation)
- Returns `EntityGlowMaterial` handle

Changes to `breaker-game/src/walls/builder/core/terminal.rs`:
- `WallBuilder<S, Visible>::build()` inserts `Mesh2d` + `MeshMaterial2d<EntityGlowMaterial>` (instead of `MeshMaterial2d<ColorMaterial>`)

New `WallDefinition` fields in `breaker-game/src/walls/definition.rs`:
- `glow: Option<GlowParams>` with `#[serde(default)]` -- allows per-definition glow override
- Existing `color_rgb: Option<[f32; 3]>` remains, but the builder converts it to `Hue::Custom(r, g, b, 1.0)` internally

Update `breaker-game/assets/walls/wall.wall.ron`:
- No changes needed (defaults are fine for the standard wall definition)

### 2. spawn_walls Uses visible() Path

**Current state**: `spawn_walls` in `breaker-game/src/state/run/node/systems/spawn_walls/system.rs` calls `.spawn(&mut commands)` which uses the `Invisible` build path.

**Target**: Call `.visible()` to attach `EntityGlowMaterial`, making walls render as subtle glowing borders.

Changes to `spawn_walls`:
- System signature adds `ResMut<Assets<Mesh>>`, `ResMut<Assets<EntityGlowMaterial>>`
- Each wall builder chain calls `.visible(&mut meshes, &mut materials)` before `.spawn(&mut commands)`
- Left, right, ceiling walls all use the same visual path

Remove the `#[allow(dead_code)]` attributes on:
- `Visible` struct in `types.rs`
- `visible()` method in `transitions.rs`
- `WallBuilder<S, Visible>::build()` and `spawn()` in `terminal.rs`

### 3. Shield Barrier Energy Field Shader

**Current state**: Shield effect in `breaker-game/src/effect/effects/shield/system.rs` spawns a floor wall with `ColorMaterial` and hardcoded color `[0.3, 0.6, 2.0]`.

**Target**: Replace with a dedicated `ShieldBarrierMaterial` rendering an animated hexagonal energy field.

#### ShieldBarrierMaterial

New material in `breaker-game/src/visuals/materials/shield_barrier.rs`:

```rust
ShieldBarrierMaterial {
    uniforms: ShieldBarrierUniforms,
}

ShieldBarrierUniforms {
    color: Vec4,               // base energy field color (default: cyan-blue HDR)
    intensity: f32,            // overall brightness (HDR, default: 1.5)
    hex_scale: f32,            // hexagonal pattern density (default: 12.0)
    pulse_speed: f32,          // animation speed for hex pulse (default: 2.0)
    crack_count: u32,          // number of active cracks (0-5)
    crack_seeds: [Vec4; 2],    // 5 crack seed positions packed into 2 Vec4s (xy pairs + padding)
    crack_intensity: f32,      // darkness of crack regions (0.0 = no cracks, 1.0 = fully dark)
    alpha: f32,                // overall alpha (used for fade-in/out)
    _padding: Vec2,
}
```

Uses the additive blend `specialize()` pattern from visuals/ (`apply_additive_blend`).

#### shield_barrier.wgsl

New shader at `breaker-game/assets/shaders/shield_barrier.wgsl`:

Algorithm per fragment:
1. Compute hexagonal tiling UV from fragment position (pointy-top hexagons)
2. Compute distance to nearest hex edge -- this creates the honeycomb grid lines
3. Pulse brightness along hex edges using `sin(time * pulse_speed + hex_center_distance)`
4. For each active crack (up to `crack_count`): compute noise-seeded dark region around `crack_seeds[i]` position, darken the hex cells near the crack
5. Final color: `color.rgb * intensity * hex_edge_brightness * (1.0 - crack_darkness)` with alpha from hex edge proximity

Visual characteristics (per design doc DR-3):
- Semi-transparent energy field spanning breaker width
- Animated hexagonal/honeycomb pattern
- Pulsing white/cyan energy along hex edges
- Procedural crack damage via noise-seeded dark regions

#### Shield Effect fire() Update

Changes to `breaker-game/src/effect/effects/shield/system.rs`:
- `fire()` spawns with `ShieldBarrierMaterial` instead of `ColorMaterial`
- Creates `ShieldBarrierUniforms` with default values (0 cracks initially)
- Removes the manual `ColorMaterial` creation code

#### Shield Crack System

New system in `breaker-game/src/effect/effects/shield/`:
- `update_shield_cracks` -- when `ShieldWallTimer` remaining fraction decreases past thresholds, increment `crack_count` and add a new `crack_seed` position (randomized along the barrier width)
- Runs in `FixedUpdate` after `deduct_shield_on_reflection`
- Maps timer fraction to crack count: 80% = 1 crack, 60% = 2, 40% = 3, 20% = 4, 10% = 5

#### Shield Shatter on Last Charge

When the shield wall is about to despawn (timer finishes or reflection cost depletes it):
- Brief intensity spike on the `ShieldBarrierMaterial` (set `intensity` to 3.0 for 2-3 frames)
- Then despawn
- Screen flash at bottom edge (via `TriggerScreenFlash` from `rantzsoft_postprocess` -- if 5d is available; otherwise skip screen flash for now and add a TODO)
- Particle burst along the barrier (via `rantzsoft_particles2d` -- if 5c is available; otherwise skip particles for now and add a TODO)

### 4. Wall Impact Flash

**Current state**: `BoltImpactWall` message is sent when a bolt hits a wall, but there is no visual response.

**Target**: Brief glow pulse at the impact point that travels a short distance along the wall, then fades.

#### Approach

The impact flash is NOT a modification to the wall's own material (which would require per-pixel impact position in the wall shader). Instead, it is a **child entity** spawned at the collision point with its own `EntityGlowMaterial`.

New system `spawn_wall_impact_flash`:
- Reads `BoltImpactWall` messages
- For each message, queries the bolt's `Position2D` and the wall's `Position2D` + `Scale2D`
- Computes the impact point (bolt position clamped to the wall's surface)
- Spawns a small flash entity at the impact point:
  - `EntityGlowMaterial` with `Shape::Circle`, high `core_brightness` (2.0), large `halo_radius` (1.5), fast `halo_falloff` (8.0)
  - Color matches the wall's current color (or temperature palette wall color)
  - `FadeOut` component (from visuals/) with ~0.15s duration -- flash fades quickly
  - `PunchScale` component -- brief scale-up then settle
  - `GameDrawLayer::Fx` (renders above the wall)
  - `CleanupOnNodeExit` for safety

Schedule: runs in `Update`, after `BoltSystems::WallCollision`

Location: `breaker-game/src/walls/systems/spawn_wall_impact_flash.rs` (new file, wall domain owns this system since it is wall-specific visual feedback)

Register in `WallPlugin`.

### 5. Background Grid Shader

**Current state**: No background rendering. `ClearColor` is the only visible background.

**Target**: Single full-playfield quad with a `BackgroundGridMaterial` shader rendering a subtle grid pattern.

#### BackgroundGridMaterial

New material in `breaker-game/src/visuals/materials/background_grid.rs`:

```rust
BackgroundGridMaterial {
    uniforms: BackgroundGridUniforms,
}

BackgroundGridUniforms {
    grid_color: Vec4,          // line color (default: dim blue, HDR ~0.08)
    cell_size: f32,            // grid cell size in UV space (default: 0.04 -- ~25 cells across)
    line_thickness: f32,       // grid line width in UV space (default: 0.002)
    playfield_size: Vec2,      // width, height in world units (for aspect correction)
}
```

Does NOT use additive blend -- the grid is opaque (rendered behind everything). Uses standard alpha blend so the grid lines are visible against the void.

#### background_grid.wgsl

New shader at `breaker-game/assets/shaders/background_grid.wgsl`:

Algorithm per fragment:
1. Scale UV by `playfield_size` for aspect-correct grid cells
2. Compute distance to nearest grid line: `min(fract(uv / cell_size), 1.0 - fract(uv / cell_size))`
3. Grid line alpha: `smoothstep(line_thickness, 0.0, distance)` -- anti-aliased edges
4. Final color: `grid_color * grid_line_alpha`

Visual characteristics (per design doc):
- Flat 2D grid -- straight horizontal and vertical lines
- Very dim -- barely visible against the void
- Static reference surface -- does NOT warp, bend, or react to game events
- Post-processing effects (gravity wells, shockwaves) warp the rendered screen which incidentally warps the grid as viewed

#### Background Grid Spawn

New system `spawn_background_grid`:
- Runs on `OnEnter(NodeState::Loading)` (alongside wall spawning)
- Spawns a single quad entity covering the full playfield area:
  - `Mesh2d` with a `Rectangle` matching playfield dimensions
  - `MeshMaterial2d<BackgroundGridMaterial>`
  - Position at playfield center, Z = -1.0 (behind all gameplay entities)
  - `CleanupOnNodeExit`

Location: this system lives in `breaker-game/src/state/run/node/systems/` as a new file (it is node lifecycle, not visuals domain). Alternatively it could live in a `background/` module under walls/ since the background is part of the playfield boundary visual identity. Decision: place in `breaker-game/src/walls/systems/spawn_background_grid.rs` -- the wall domain owns the playfield boundaries and background.

Register in `WallPlugin`, same schedule as `spawn_walls`.

#### GameDrawLayer Update

Add a new variant to `GameDrawLayer` in `breaker-game/src/shared/draw_layer.rs`:
- `Background` with Z = -1.0 (behind all other layers)

### 6. Void ClearColor Update

**Current state**: `ClearColor` uses `PlayfieldConfig::background_color()` which returns `[0.02, 0.01, 0.04]` (dark purple-black).

**Target**: Update to `[0.02, 0.02, 0.06]` (deep blue-black, #050510 equivalent in linear space). The design doc specifies deep blue-black so gravity well voids register as even darker.

Changes:
- Update `background_color_rgb` default in `PlayfieldConfig::default()` at `breaker-game/src/shared/playfield.rs`
- Update the matching value in test helpers at `breaker-game/src/state/run/node/systems/spawn_cells_from_layout/tests/helpers.rs`

### 7. VisualsPlugin Material Registration

Register the two new materials in `VisualsPlugin` (if 5e has already created it) or in `WallPlugin` (if 5i ships before 5e, which is unlikely given the dependency):
- `Material2dPlugin::<ShieldBarrierMaterial>`
- `Material2dPlugin::<BackgroundGridMaterial>`

These registrations belong in `VisualsPlugin` since it owns all material plugins. `WallPlugin` only registers wall-domain systems and messages.

## Type Definitions

### ShieldBarrierMaterial

```rust
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct ShieldBarrierMaterial {
    #[uniform(0)]
    pub uniforms: ShieldBarrierUniforms,
}

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct ShieldBarrierUniforms {
    pub color: Vec4,
    pub intensity: f32,
    pub hex_scale: f32,
    pub pulse_speed: f32,
    pub crack_count: u32,
    pub crack_seeds: [Vec4; 2],    // 5 positions packed: [0].xy, [0].zw, [1].xy, [1].zw, unused
    pub crack_intensity: f32,
    pub alpha: f32,
    pub _padding: Vec2,
}
```

### BackgroundGridMaterial

```rust
#[derive(Asset, TypePath, AsBindGroup, Clone, Debug)]
pub struct BackgroundGridMaterial {
    #[uniform(0)]
    pub uniforms: BackgroundGridUniforms,
}

#[derive(Clone, Copy, Debug, ShaderType)]
pub struct BackgroundGridUniforms {
    pub grid_color: Vec4,
    pub cell_size: f32,
    pub line_thickness: f32,
    pub playfield_size: Vec2,
}
```

### ShieldCrackState Component

```rust
#[derive(Component, Debug, Default)]
pub struct ShieldCrackState {
    pub crack_count: u32,
    pub crack_seeds: Vec<Vec2>,    // positions along barrier width
    pub last_fraction: f32,         // last timer fraction checked (for threshold detection)
}
```

## Module Structure

New/modified files:

```
breaker-game/src/visuals/materials/
    shield_barrier.rs               // ShieldBarrierMaterial, ShieldBarrierUniforms
    background_grid.rs              // BackgroundGridMaterial, BackgroundGridUniforms

breaker-game/src/walls/
    builder/core/types.rs           // Visible struct updated: EntityGlowMaterial
    builder/core/transitions.rs     // visible() updated: EntityGlowMaterial path
    builder/core/terminal.rs        // build() updated: MeshMaterial2d<EntityGlowMaterial>
    systems/
        mod.rs                      // add spawn_wall_impact_flash, spawn_background_grid
        spawn_wall_impact_flash.rs  // new: wall impact flash system
        spawn_background_grid.rs    // new: background grid spawn system
    plugin.rs                       // register new systems

breaker-game/src/effect/effects/shield/
    system.rs                       // fire() updated: ShieldBarrierMaterial
    mod.rs                          // re-export ShieldCrackState if needed

breaker-game/src/shared/
    draw_layer.rs                   // add Background variant (Z = -1.0)
    playfield.rs                    // update background_color_rgb default

breaker-game/assets/shaders/
    shield_barrier.wgsl             // new: hexagonal energy field shader
    background_grid.wgsl            // new: grid line shader
```

## What NOT to Do

- Do NOT implement the temperature palette application system that reads `RunTemperature` and updates wall/grid colors dynamically -- that is 5j (dynamic visuals). Walls and grid use static initial colors; 5j adds the runtime tint system.
- Do NOT implement background energy sprites (particles traveling along grid lines) -- those depend on `rantzsoft_particles2d` emitter integration and are polish, not core rendering. Add a TODO for a later phase.
- Do NOT modify the shield effect's gameplay mechanics (timer, reflection cost, charges) -- only change the visual rendering.
- Do NOT add aura or trail components to walls -- walls are simple SDF rectangles with glow. Per the architecture table in `docs/todos/detail/phase-5-rethink/architecture.md`, walls get "glow, shield barrier shader (for shield walls)" only.
- Do NOT put wall-specific systems in the visuals/ domain -- wall impact flash and background grid spawning belong in the walls/ domain. Only material/shader definitions go in visuals/.
- Do NOT modify the `BoltImpactWall` message format -- the impact flash system reads the existing message and queries positions from ECS.
- Do NOT add particle effects for shield shatter or wall impact if 5c (`rantzsoft_particles2d`) is not yet available. Add TODOs marking where particles will be integrated later.

## Verification

- Left, right, ceiling walls render as subtle glowing SDF rectangles (not invisible)
- Wall glow is dim enough to not compete with bolt/breaker/cell entities (core_brightness < 0.3 HDR)
- Shield barrier renders with animated hexagonal honeycomb pattern when shield chip activates
- Shield barrier shows procedural cracks as timer depletes (1 crack at 80%, up to 5 at 10%)
- Shield barrier intensity spikes briefly before despawn
- Wall impact flash appears at the correct bolt-wall collision point
- Wall impact flash fades out within ~0.15s
- Wall impact flash does not persist or accumulate (each flash is independent)
- Background grid renders behind all gameplay entities (Z = -1.0)
- Background grid lines are very dim (HDR ~0.08) and anti-aliased
- Background grid has correct aspect ratio (not stretched)
- ClearColor is deep blue-black
- `GameDrawLayer::Background` Z is -1.0
- All existing wall builder tests pass (Visible path tests updated for EntityGlowMaterial)
- All existing shield effect tests pass (gameplay mechanics unchanged)
- `cargo all-dclippy` clean
- `cargo all-dtest` clean
- All WGSL shader files parse without syntax errors

## Status
`[NEEDS DETAIL]` — blocked on 5e API design (how wall builder consumes EntityGlowMaterial, shield barrier material registration pattern)
