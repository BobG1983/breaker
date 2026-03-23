---
name: Bevy 0.18.1 spatial2d built-in audit
description: What Bevy 0.18.1 provides natively vs what must be custom-built for a spatial2d crate
type: reference
---

## Verified against Bevy 0.18.1 registry source (index.crates.io-1949cf8c6b5b557f)

---

## 1. 2D position/rotation/scale components — NOT provided

Bevy 0.18.1 has **no** `Position2D`, `Rotation2D`, `Scale2D`, `Transform2D`, or any 2D-specific spatial component.

What exists:
- `bevy::transform::components::Transform` — 3D struct: `{ translation: Vec3, rotation: Quat, scale: Vec3 }`
- `bevy::transform::components::GlobalTransform` — read-only computed world-space transform
- `bevy::math::Isometry2d` — in **bevy_math only**, NOT a Component. Fields: `{ rotation: Rot2, translation: Vec2 }`. Pure math type, not ECS.

In 2D, the z component of `Transform.translation` is **explicitly documented** as the z-order value:
> "In 2d, the last value of the Vec3 is used for z-ordering."

All 2D rendering goes through the full 3D `Transform`. There is no lighter 2D component.

**Verdict**: A custom `Position2D`, `Rotation2D`, `Scale2D`, `Angle`, or similar 2D components would be entirely new. Bevy provides none.

---

## 2. Fixed-timestep interpolation — NOT provided as built-in

Bevy 0.18.1 has **no** `TransformInterpolationPlugin`, `PhysicsInterpolationPlugin`, or any built-in automatic interpolation between FixedUpdate positions and render frames.

What Bevy provides (the building blocks):
- `Time<Fixed>::overstep_fraction() -> f32` — the lerp alpha (0.0–1.0 fraction of timestep already "used")
- `RunFixedMainLoopSystems::AfterFixedMainLoop` — correct schedule slot for interpolation

The interpolation logic (storing previous/current physics state, lerping into visual Transform) must be written manually. See `transform_interpolation.md` for the full verified pattern and the `bevy_transform_interpolation` third-party crate option.

**Verdict**: No built-in interpolation. Building it into a custom crate (or adopting `bevy_transform_interpolation`) is the right approach.

---

## 3. Draw ordering / Z-layer system — PARTIAL: z via Transform.z, RenderLayers for camera routing

Bevy 0.18.1 provides two mechanisms relevant to 2D draw ordering:

### A. `Transform.translation.z` — the primary 2D draw order mechanism

The official approach for 2D draw ordering is to set `Transform.translation.z`. Higher z renders in front. This is the only mechanism for ordering sprites/meshes within the same camera.

No named component like `ZLayer`, `DrawOrder`, or `SortKey` exists for non-UI entities.

### B. `RenderLayers` (bevy_camera::visibility::render_layers) — camera-visibility routing only

```rust
// bevy::prelude::RenderLayers
pub struct RenderLayers(SmallVec<[u64; 1]>);
```

- Path: `bevy::camera::visibility::RenderLayers` (also in prelude)
- Component; entities without it default to layer 0
- Cameras also get `RenderLayers`; a camera only renders entities whose layers intersect its own
- Used to make certain entities visible to only certain cameras (e.g., UI camera vs world camera)
- This is **not** a draw-order mechanism — it does not sort sprites; it controls camera visibility

Constructors: `RenderLayers::layer(n)`, `RenderLayers::none()`, `.with(n)`, `.without(n)`

### C. `ZIndex` / `GlobalZIndex` — UI ONLY (bevy_ui)

```rust
// bevy_ui ONLY
pub struct ZIndex(pub i32);
pub struct GlobalZIndex(pub i32);
```

These are `bevy::ui`-only components for ordering UI `Node` entities. They have NO effect on sprites or mesh2d entities.

**Verdict**: For 2D game sprites/meshes, `Transform.translation.z` is the only built-in ordering tool. No dedicated named z-layer component exists for non-UI 2D entities. A custom `ZLayer` component that maps to a specific z value would be entirely novel.

---

## 4. Parent/child absolute positioning — NOT provided

Bevy 0.18.1 has **no** built-in concept of "absolute" children that ignore parent transforms.

The transform propagation system (`TransformSystems::Propagate` in PostUpdate) unconditionally composes all parent transforms. The following do NOT exist:
- `PositionPropagation::Absolute`
- `InheritTransform` / `NoInheritTransform`
- `InheritScale` / `NoInheritScale`
- Any transform inheritance override mechanism

**Workaround pattern used in practice**: Maintain the child's absolute position manually by computing `parent_global_transform.inverse() * desired_absolute_transform` each frame and writing that to the child's local `Transform`. This is manual and not zero-cost.

**Verdict**: No built-in support. An `AbsolutePosition` component or similar in a custom crate would need to manually compensate for the parent transform every frame.

---

## 5. Transform2D or similar — NOT provided

No `Transform2D` type, trait, or plugin exists anywhere in Bevy 0.18.1. There is no Bevy RFC or built-in system that replaces `Transform` for 2D use cases. The Bevy project officially uses 3D `Transform` for all 2D work, relying on z=0.0 for "flat" scenes.

**Verdict**: A `Transform2D` component in a custom crate would be entirely novel.

---

## Summary table

| Feature | Bevy 0.18.1 native support |
|---|---|
| Position2D / Rotation2D / Scale2D components | None — use Transform (3D) or build custom |
| Fixed-timestep interpolation plugin | None — only `overstep_fraction()` raw value |
| 2D z-order / draw layer system | Only `Transform.translation.z` (no named layer component) |
| Absolute child positioning | None — must compensate manually |
| Transform2D / spatial2d types | None (Isometry2d exists in bevy_math as pure math only) |
| RenderLayers | Yes — but for camera visibility routing, not draw order |
| ZIndex / GlobalZIndex | Yes — UI only (Node entities) |
