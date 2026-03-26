---
name: rantzsoft-crate-exports
description: Full public API breakdown for rantzsoft_spatial2d, rantzsoft_physics2d, rantzsoft_defaults, and rantzsoft_defaults_derive — types, traits, plugins, systems, SystemSets
type: reference
---

# rantzsoft_* Crate Public API

Analyzed against Bevy 0.18. Branch: `feature/spatial-physics-extraction`.

---

## rantzsoft_defaults

### lib.rs exports
```rust
pub use rantzsoft_defaults_derive::GameConfig;
```
This crate is a pure re-export shim. The only public item is the `GameConfig` derive macro.

---

## rantzsoft_defaults_derive

### Proc-macro crate — exports
```rust
#[proc_macro_derive(GameConfig, attributes(game_config))]
pub fn derive_game_config(input: TokenStream) -> TokenStream
```

**What `GameConfig` generates from a `*Defaults` struct:**
1. A `*Config` struct with identical fields deriving `Resource, Debug, Clone`
2. `impl From<*Defaults> for *Config` (field-by-field copy)
3. `impl Default for *Config` delegating to `*Defaults::default().into()`

**Usage annotation required:** `#[game_config(name = "BreakerConfig")]` on the `*Defaults` struct.

**No Bevy systems, plugins, types, or traits.** Pure proc-macro.

---

## rantzsoft_spatial2d

### lib.rs — public modules
```rust
pub mod components;
pub mod draw_layer;
pub mod plugin;
pub mod propagation;
pub mod systems;
```
No top-level `pub use` re-exports. Consumers must use full paths like
`rantzsoft_spatial2d::components::Position2D`.

### Plugin

| Name | Type | Generic | Schedule registrations |
|------|------|---------|------------------------|
| `RantzSpatial2dPlugin<D: DrawLayer>` | `struct + Plugin` | Yes — game-provided `DrawLayer` enum | `FixedFirst`: `save_previous`; `FixedUpdate`: `apply_velocity`; `RunFixedMainLoop` (AfterFixedMainLoop): `compute_globals → derive_transform::<D>` (chained) |

`impl Default for RantzSpatial2dPlugin<D>` is provided.

### Traits

| Trait | Module | Bounds | Method |
|-------|--------|--------|--------|
| `DrawLayer` | `draw_layer` | `Component + Copy + Send + Sync + 'static` | `fn z(&self) -> f32` |

### Components (all in `components` module)

| Type | Key derives | Notes |
|------|-------------|-------|
| `Position2D(pub Vec2)` | Component, Reflect, Deref, DerefMut | Ops: Add/Sub Vec2, Mul/Div f32; methods: `distance`, `distance_squared` |
| `Rotation2D(pub Rot2)` | Component, Reflect | Methods: `from_degrees`, `from_radians`, `as_radians`, `as_degrees`, `to_quat` |
| `Scale2D { x: f32, y: f32 }` | Component, Reflect | Methods: `new` (panics on zero), `uniform`, `to_vec3`; default = (1,1) |
| `PreviousPosition(pub Vec2)` | Component, Reflect, Deref, DerefMut | Snapshot for interpolation |
| `PreviousRotation(pub Rot2)` | Component, Reflect | Snapshot for interpolation |
| `PreviousScale { x, y }` | Component, Reflect | Snapshot for interpolation; default = (1,1) |
| `GlobalPosition2D(pub Vec2)` | Component, Reflect, Deref, DerefMut | Computed from hierarchy |
| `GlobalRotation2D(pub Rot2)` | Component, Reflect | Computed from hierarchy |
| `GlobalScale2D { x, y }` | Component, Reflect | Computed from hierarchy; default = (1,1) |
| `Velocity2D(pub Vec2)` | Component, Reflect, Deref, DerefMut | Methods: `speed`, `clamped`; Ops: Add/Sub Vec2, Mul/Div f32 |
| `PreviousVelocity(pub Vec2)` | Component, Reflect, Deref, DerefMut | Snapshot for interpolation |
| `ApplyVelocity` | Component, Reflect | Marker — opts entity into `apply_velocity` system |
| `InterpolateTransform2D` | Component, Reflect | Marker — opts entity into `save_previous` and interpolated path in `derive_transform` |
| `VisualOffset(pub Vec3)` | Component, Reflect, Deref, DerefMut | Applied after propagation (screen-shake etc.) |
| `Spatial2D` | Component | Required-components aggregate marker — automatically adds all of the above spatial components plus Transform. Default propagation = Relative for all axes. |

**`Spatial2D` required components chain (auto-inserted on spawn):**
Position2D, Rotation2D, Scale2D, PreviousPosition, PreviousRotation, PreviousScale,
GlobalPosition2D, GlobalRotation2D, GlobalScale2D, PositionPropagation, RotationPropagation, ScalePropagation, Transform

**NOTE:** Velocity2D and PreviousVelocity are NOT required by Spatial2D — must be added explicitly.

### Propagation enums (in `propagation` module)

| Type | Variants | Default |
|------|----------|---------|
| `PositionPropagation` | `Relative`, `Absolute` | `Relative` |
| `RotationPropagation` | `Relative`, `Absolute` | `Relative` |
| `ScalePropagation` | `Relative`, `Absolute` | `Relative` |

All are `Component + Reflect`.

### Systems (in `systems` submodules)

These are pub-but-not-re-exported — accessed via `crate::systems::*::fn_name` or used internally by the plugin.

| Function | Module | Schedule (plugin) | Reads | Writes |
|----------|--------|--------------------|-------|--------|
| `save_previous` | `systems::save_previous` | `FixedFirst` | `Position2D`, `GlobalPosition2D?`, `Rotation2D`, `GlobalRotation2D?`, `Scale2D`, `GlobalScale2D?`, `Velocity2D` (all filtered `With<InterpolateTransform2D>`) | `PreviousPosition`, `PreviousRotation`, `PreviousScale`, `PreviousVelocity` |
| `apply_velocity` | `systems::apply_velocity` | `FixedUpdate` | `Velocity2D`, `Time<Fixed>` (filtered `With<ApplyVelocity>`) | `Position2D` |
| `compute_globals` | `systems::compute_globals` | `RunFixedMainLoop / AfterFixedMainLoop` (before derive_transform) | `Position2D`, `Rotation2D`, `Scale2D`, `ChildOf?`, `PositionPropagation?`, `RotationPropagation?`, `ScalePropagation?` | `GlobalPosition2D`, `GlobalRotation2D`, `GlobalScale2D` |
| `derive_transform::<D>` | `systems::derive_transform` | `RunFixedMainLoop / AfterFixedMainLoop` (after compute_globals) | `GlobalPosition2D`, `GlobalRotation2D`, `GlobalScale2D`, `D: DrawLayer`, `InterpolateTransform2D?`, `PreviousPosition?`, `PreviousRotation?`, `PreviousScale?`, `VisualOffset?`, `ChildOf?`, `Time<Fixed>` | `Transform` |
| `propagate_position::<D>` | `systems::propagate_position` | NOT registered by plugin — available for manual use | `Position2D`, `D: DrawLayer`, `InterpolateTransform2D?`, `PreviousPosition?`, `VisualOffset?`, `PositionPropagation?`, `ChildOf?`, `Time<Fixed>` | `Transform.translation` |
| `propagate_rotation` | `systems::propagate_rotation` | NOT registered by plugin — available for manual use | `Rotation2D`, `InterpolateTransform2D?`, `PreviousRotation?`, `RotationPropagation?`, `ChildOf?`, `Time<Fixed>` | `Transform.rotation` |
| `propagate_scale` | `systems::propagate_scale` | NOT registered by plugin — available for manual use | `Scale2D`, `PreviousScale`, `InterpolateTransform2D?`, `ScalePropagation?`, `ChildOf?`, `Time<Fixed>` | `Transform.scale` |

**Important:** `propagate_position`, `propagate_rotation`, and `propagate_scale` are older single-axis systems. They are public but NOT registered by `RantzSpatial2dPlugin`. The plugin uses `compute_globals` + `derive_transform` which handles all three axes in two passes and correctly handles the full hierarchy.

### SystemSets
None defined in this crate. System ordering is achieved via `.chain()` in `RunFixedMainLoop`.

---

## rantzsoft_physics2d

### lib.rs — public modules
```rust
pub mod aabb;
pub mod ccd;
pub mod collision_layers;
pub mod constraint;
pub mod plugin;
pub mod quadtree;
pub mod resources;
pub mod systems;
```

### Plugin

| Name | Type | Generic | Schedule registrations |
|------|------|---------|------------------------|
| `RantzPhysics2dPlugin` | `struct + Plugin` | No | `FixedUpdate`: `maintain_quadtree` (in `PhysicsSystems::MaintainQuadtree`), `enforce_distance_constraints` (in `PhysicsSystems::EnforceDistanceConstraints`) |

### SystemSets

```rust
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum PhysicsSystems {
    MaintainQuadtree,
    EnforceDistanceConstraints,
}
```

Located in `plugin` module. Game systems that need to run after quadtree maintenance use `.after(PhysicsSystems::MaintainQuadtree)`.

### Components

| Type | Module | Key derives | Notes |
|------|--------|-------------|-------|
| `Aabb2D { center: Vec2, half_extents: Vec2 }` | `aabb` | Component | `#[require(Spatial2D)]` — spawning Aabb2D automatically pulls in all Spatial2D components. Methods: `new`, `from_min_max`, `min`, `max`, `contains_point`, `overlaps`, `expand_by`, `contains_aabb`, `merge` |
| `CollisionLayers { membership: u32, mask: u32 }` | `collision_layers` | Component, Default | Methods: `new`, `interacts_with`. Filter rule: `self.mask & other.membership != 0`. Default = (0, 0) = invisible |
| `DistanceConstraint { entity_a: Entity, entity_b: Entity, max_distance: f32 }` | `constraint` | Component, Clone | Data-only constraint; solver system lives in this crate. |

### Resources

| Type | Module | Notes |
|------|--------|-------|
| `CollisionQuadtree { quadtree: Quadtree }` | `resources` | Initialized by plugin via `init_resource`. Default bounds: center=(0,0), half_extents=(600,400), max_items_per_leaf=8, max_depth=8. Methods: `new`, `Default`. |

### Types (non-ECS, non-Component)

| Type | Module | Notes |
|------|--------|-------|
| `Quadtree` | `quadtree` | Internal spatial index. Public methods: `new`, `insert(Entity, Aabb2D, CollisionLayers)`, `remove(Entity) -> bool`, `query_aabb(&Aabb2D) -> Vec<Entity>`, `query_aabb_filtered(&Aabb2D, CollisionLayers) -> Vec<Entity>`, `query_circle(Vec2, f32) -> Vec<Entity>`, `query_circle_filtered(Vec2, f32, CollisionLayers) -> Vec<Entity>`, `len() -> usize`, `is_empty() -> bool`, `clear()` |
| `RayHit { distance: f32, normal: Vec2 }` | `ccd` | Output of `ray_vs_aabb` |

### Free functions (non-system)

| Function | Module | Signature |
|----------|--------|-----------|
| `ray_vs_aabb` | `ccd` | `(origin: Vec2, direction: Vec2, max_dist: f32, aabb: &Aabb2D) -> Option<RayHit>` |

### Constants

| Name | Module | Value |
|------|--------|-------|
| `MAX_BOUNCES` | `ccd` | `4u32` |
| `CCD_EPSILON` | `ccd` | `0.01f32` |

### Systems (in `systems` submodules)

The `systems::mod` re-exports both:
```rust
pub use enforce_distance_constraints::enforce_distance_constraints;
pub use maintain_quadtree::maintain_quadtree;
```

| Function | Module | Schedule (plugin) | Reads | Writes |
|----------|--------|--------------------|-------|--------|
| `maintain_quadtree` | `systems::maintain_quadtree` | `FixedUpdate` (in `PhysicsSystems::MaintainQuadtree`) | `RemovedComponents<Aabb2D>`, `Aabb2D` (Added), `GlobalPosition2D` (Changed), `CollisionLayers` (Changed) | `CollisionQuadtree` (ResMut) — insert/remove/update |
| `enforce_distance_constraints` | `systems::enforce_distance_constraints` | `FixedUpdate` (in `PhysicsSystems::EnforceDistanceConstraints`) | `DistanceConstraint`, `Position2D`, `Velocity2D` | `Position2D`, `Velocity2D` |

**`maintain_quadtree` uses `GlobalPosition2D` (world-space), NOT `Position2D` (local-space).**
The world-space AABB inserted is `Aabb2D::new(global_pos.0, aabb.half_extents)`.

---

## Cross-crate dependency note

`rantzsoft_physics2d` depends on `rantzsoft_spatial2d` (uses `GlobalPosition2D`, `Position2D`, `Velocity2D`, `Spatial2D`).

`Aabb2D` has `#[require(Spatial2D)]`, so any entity spawned with `Aabb2D` automatically gets the full spatial component set from `rantzsoft_spatial2d`.

The `enforce_distance_constraints` system uses `Position2D` and `Velocity2D` from `rantzsoft_spatial2d`.
