# Bevy 0.18.1 Mesh Generation Research

Verified against: docs.rs/bevy/0.18.1, docs.rs/bevy_mesh/0.18.1, GitHub bevyengine/bevy v0.18.0 source,
and existing project code in `breaker-game/src/`.

---

## 1. Built-in 2D Primitive Mesh Types

All of the following are in `bevy::math::primitives` (re-exported via `bevy::prelude`).
Every one implements `Meshable`, meaning `meshes.add(PrimitiveType::new(...))` compiles directly.

| Primitive          | Constructor                                     | Notes |
|--------------------|-------------------------------------------------|-------|
| `Circle`           | `Circle::new(radius: f32)`                      | 1 field: `radius: f32` |
| `Ellipse`          | `Ellipse::new(half_width, half_height)`         | |
| `Rectangle`        | `Rectangle::new(width, height)`                 | field: `half_size: Vec2` |
| `RegularPolygon`   | `RegularPolygon::new(circumradius, sides: u32)` | All vertices on a circle |
| `Rhombus`          | `Rhombus::new(h_diagonal, v_diagonal)`          | field: `half_diagonals: Vec2`; diamond orientation |
| `Triangle2d`       | `Triangle2d::new(a, b, c: Vec2)`                | |
| `Capsule2d`        | `Capsule2d::new(radius, length)`                | Pill/stadium; fields: `radius`, `half_length` |
| `Annulus`          | `Annulus::new(inner_r, outer_r)`                | Ring |
| `Arc2d`            | —                                               | Arc between two circle points |
| `CircularSector`   | —                                               | Pie slice |
| `CircularSegment`  | —                                               | Area between arc and chord |
| `ConvexPolygon`    | `ConvexPolygon::new(verts: impl IntoIterator<Item=Vec2>) -> Result<_, _>` | Custom convex shape; also `new_unchecked` |

The `Meshable` trait is:
```rust
pub trait Meshable {
    type Output: MeshBuilder;
    fn mesh(&self) -> Self::Output;
}
```
Calling `meshes.add(prim)` works via `From<Primitive> for Mesh` — the primitive is auto-converted.
You can also call `prim.mesh().build()` explicitly to get a `Mesh` value.

---

## 2. Hexagon, Octagon, Diamond

### Hexagon
```rust
meshes.add(RegularPolygon::new(circumradius, 6))
```
Default orientation: one vertex points straight up. For a flat-top hexagon, rotate the entity 30°.

### Octagon
```rust
meshes.add(RegularPolygon::new(circumradius, 8))
```

### Diamond
`Rhombus` is the correct primitive — it is a diamond/rhombus with independently settable
horizontal and vertical diagonals:
```rust
meshes.add(Rhombus::new(horizontal_diagonal, vertical_diagonal))
// e.g. wider-than-tall: Rhombus::new(2.0, 1.0)
```
This is NOT a rotated square — it's a true rhombus with `half_diagonals: Vec2` as its only field.
A rotated square would be `RegularPolygon::new(r, 4)` but rotated 45°; `Rhombus` is cleaner.

---

## 3. RoundedRectangle

**There is no built-in `RoundedRectangle` primitive or mesh in Bevy 0.18.1.**

The options are:
- **`Capsule2d`**: gives a pill shape (rectangle with semicircular end-caps). Not a fully rounded
  rectangle — only the top and bottom are rounded.
- **Manual mesh**: Generate a rounded rectangle by tesselating 4 quarter-circle arcs at the corners
  joined by 4 straight edges. See "Manual Mesh Creation" below.
- **Approximation**: For a slightly soft shape, use `Capsule2d` rotated 90° with a short half-length
  and large radius-to-length ratio.

Manual rounded rectangle approach (for reference, not project code):
- 8 vertices per arc segment × 4 corners = 32 corner vertices + 4 midpoints = ~36 vertices
- Fan-triangulate from center: `(N-1)` triangles for a convex shape with N boundary vertices

---

## 4. Manual Mesh Creation API

### `Mesh::new()` — Bevy 0.18 signature
```rust
pub fn new(primitive_topology: PrimitiveTopology, asset_usage: RenderAssetUsages) -> Mesh
```

**Important**: Bevy 0.18 added `RenderAssetUsages` as a REQUIRED second parameter.
Use `RenderAssetUsages::RENDER_WORLD` for static game meshes (not accessed from CPU after upload).

### `PrimitiveTopology` variants
```
PointList, LineList, LineStrip, TriangleList, TriangleStrip
```
Use `PrimitiveTopology::TriangleList` for solid filled 2D shapes.

### `Indices` enum
```rust
pub enum Indices {
    U16(Vec<u16>),
    U32(Vec<u32>),
}
```

### Inserting attributes
```rust
mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, v_pos); // Vec<[f32; 3]>
mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL,   v_nor); // Vec<[f32; 3]>
mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0,     v_uv);  // Vec<[f32; 2]>
mesh.insert_indices(Indices::U32(indices));              // Vec<u32>
```

Constants:
- `Mesh::ATTRIBUTE_POSITION` — `Float32x3`
- `Mesh::ATTRIBUTE_NORMAL`   — `Float32x3`
- `Mesh::ATTRIBUTE_UV_0`     — `Float32x2`

For 2D shapes:
- All Z values in POSITION must be `0.0`
- All NORMAL values must be `[0.0, 0.0, 1.0]` (pointing out of screen)
- UV is optional for `ColorMaterial` (solid color); include if needed for textures

### Full manual mesh example (from mesh2d_manual.rs example)
```rust
let mut mesh = Mesh::new(
    PrimitiveTopology::TriangleList,
    RenderAssetUsages::RENDER_WORLD,
);

// Positions: Vec<[f32; 3]>, Z=0 for 2D
let positions: Vec<[f32; 3]> = vec![
    [0.0, 0.0, 0.0],   // center
    [1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0],
    // ...
];
mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);

// Normals: all pointing out of screen for 2D
let normals: Vec<[f32; 3]> = positions.iter().map(|_| [0.0, 0.0, 1.0]).collect();
mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

// Indices: counter-clockwise winding
mesh.insert_indices(Indices::U32(vec![0, 2, 1, ...]));

let handle = meshes.add(mesh);
commands.spawn((Mesh2d(handle), MeshMaterial2d(material_handle)));
```

### Import paths
```rust
use bevy::prelude::*;
// Mesh, Mesh2d, MeshMaterial2d, ColorMaterial, Circle, Rectangle,
// RegularPolygon, Rhombus, ConvexPolygon, Capsule2d are all in prelude

use bevy::render::mesh::Indices;
use bevy::render::render_asset::RenderAssetUsages;
use bevy::render::render_resource::PrimitiveTopology;
```

---

## 5. Mesh2d + Material2d Entity Setup

From existing project code (`spawn_breaker.rs`, `spawn_bolt.rs`):
```rust
commands.spawn((
    Mesh2d(meshes.add(Rectangle::new(1.0, 1.0))),
    MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::WHITE))),
));
```
For custom mesh:
```rust
commands.spawn((
    Mesh2d(meshes.add(my_mesh)),           // my_mesh: Mesh
    MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
));
```
Scale is applied via `Scale2D` component (project uses `rantzsoft_spatial2d`), not baked into mesh.
The project spawns unit-size meshes (1×1) and scales them via `Scale2D`.

---

## 6. Custom Shape Type Design for `CustomShape`

**Recommendation: `Vec<Vec2>` outline processed via `ConvexPolygon` or manual fan-triangulation.**

Options:
- `ConvexPolygon::new_unchecked(vertices)` — if the shape is convex (all game breaker shapes are).
  Implements `Meshable`, so `meshes.add(ConvexPolygon::new_unchecked(verts))` works directly.
- For concave shapes: store `Vec<Vec2>` outline + manually triangulate into a `Mesh`.
- Avoid storing a full `Mesh` handle in a component — instead store the minimal data needed to
  regenerate the mesh, and create the `Mesh` once at spawn.

Minimal `CustomShape` type (for project use):
```rust
pub struct CustomShape {
    pub vertices: Vec<Vec2>,  // counter-clockwise, centered at origin
}
```

To create a mesh:
```rust
let polygon = ConvexPolygon::new_unchecked(shape.vertices.iter().copied());
let handle = meshes.add(polygon);
```

---

## 7. Custom Shape Geometry Designs

All shapes centered at origin, bounding box approximately 2.0 wide × 1.0 tall
(matching a breaker paddle aspect ratio). Coordinates are in local space — actual
size is applied via `Scale2D`.

### Shape A: Shield (Aegis Breaker)
Wide convex shape with a curved convex top edge and a flat bottom edge.
Looks like a kite-shield or buckler seen from above.

```
ASCII (2.0 × 1.0 bounding box):
      ___________
   ,-'           '-,
  /                 \
 /                   \
|                     |
 \                   /
  '---___________---'
  (flat bottom edge)
```

Actually a better silhouette — wide at middle, tapered to rounded sides, with
a slight inward curve at the bottom center (like a classic heater shield):

```
        ___________
      ,'           ',
     /               \
    |       * *       |
    |      *   *      |   (* = center)
     \    *     *    /
      '--*---------*--'
          flat bottom
```

**Vertex layout (8 vertices, unit bounding box — scale to desired size):**

```
Vertices (x, y), centered at origin, width=2.0 total, height=1.0 total:
  P0 = (-1.0,  0.0)   left edge mid
  P1 = (-0.8,  0.4)   upper-left
  P2 = (-0.3,  0.5)   upper mid-left
  P3 = ( 0.0,  0.5)   top center
  P4 = ( 0.3,  0.5)   upper mid-right
  P5 = ( 0.8,  0.4)   upper-right
  P6 = ( 1.0,  0.0)   right edge mid
  P7 = ( 0.3, -0.5)   lower-right (tapered bottom)
  P8 = ( 0.0, -0.45)  bottom center (slight indent)
  P9 = (-0.3, -0.5)   lower-left (tapered bottom)
```

ASCII art (approx):
```
         P3
        /    \
      P2      P4
     /            \
   P1              P5
   |                |
   P0              P6
    \              /
    P9----P8----P7
```

Triangulation (fan from center vertex added at origin):
```
Center = (0.0, 0.0)
Triangles: (C, P0, P1), (C, P1, P2), (C, P2, P3), (C, P3, P4),
           (C, P4, P5), (C, P5, P6), (C, P6, P7), (C, P7, P8),
           (C, P8, P9), (C, P9, P0)
```
This is a convex-ish shape — use `ConvexPolygon::new_unchecked([P0..P9])` (CCW order).

---

### Shape B: Angular (Chrono Breaker)
Sleek, sharp-angled chevron. Like a stealth fighter or arrowhead seen from above.
Forward-pointing (right side is the "front"), flat trailing edge.

```
ASCII (2.0 wide × 1.0 tall):

    P2____P3
   /        \___P4
  P1              \
  |               P5--P6
  P0          ____/
   \      ___/
    P7---P8
```

Better visualization — a flat-ended chevron:
```
                  P3
                 /
        P2------/
       /          \
P1---P0             P4
       \          /
        P6------/
                 \
                  P5
```

**Vertex layout (asymmetric left-right; right side is pointy "front"):**

```
P0 = (-1.0,  0.15)   top-left inner
P1 = (-1.0,  0.5)    top-left outer
P2 = ( 0.2,  0.5)    top mid-right
P3 = ( 1.0,  0.0)    right point (tip)
P4 = ( 0.2, -0.5)    bottom mid-right
P5 = (-1.0, -0.5)    bottom-left outer
P6 = (-1.0, -0.15)   bottom-left inner
P7 = ( 0.4,  0.0)    inner waist notch (center of chevron cutout) -- optional
```

Simpler 6-vertex chevron (convex, no notch):
```
P0 = (-1.0,  0.5)    top-left
P1 = ( 0.5,  0.5)    top-right shoulder
P2 = ( 1.0,  0.0)    right tip
P3 = ( 0.5, -0.5)    bottom-right shoulder
P4 = (-1.0, -0.5)    bottom-left
P5 = (-0.5,  0.0)    left indent (makes it a chevron)
```

ASCII art:
```
P0-----------P1
|             \
P5             P2 (tip)
|             /
P4-----------P3
```

Triangulation via `ConvexPolygon::new_unchecked([P0,P1,P2,P3,P4,P5])` (CCW order).

---

### Shape C: Crystalline (Prism Breaker)
Irregular multi-faceted polygon suggesting a cut gemstone viewed from above.
Asymmetric facets at different angles create a refractive crystal cross-section look.

```
ASCII (2.0 wide × 1.0 tall):

         P2--P3
        /      \
      P1        P4
     /            \
P0--/              \--P5
     \            /
      P7        P6
        \      /
         P8--P9 (could add more facets)
```

This is symmetric top/bottom but asymmetric facet widths for crystal feel.

**Vertex layout (8 vertices, irregular facet widths):**

```
P0 = (-1.0,  0.0)    left tip (widest point)
P1 = (-0.5,  0.4)    upper-left facet
P2 = (-0.1,  0.5)    upper narrow facet
P3 = ( 0.3,  0.5)    upper-right facet
P4 = ( 0.8,  0.25)   right-upper facet
P5 = ( 1.0,  0.0)    right tip
P6 = ( 0.8, -0.25)   right-lower facet
P7 = ( 0.3, -0.5)    lower-right facet
P8 = (-0.1, -0.5)    lower narrow facet
P9 = (-0.5, -0.4)    lower-left facet
```

ASCII art (10-sided crystal):
```
            P2-P3
           /      \
         P1        P4
        /              \
P0-----/                \---P5
        \              /
         P9        P6
           \      /
            P8-P7
```

**Key**: the facet widths are deliberately irregular:
- Left side has a wide horizontal span (tip at P0)
- Upper facets: P1→P2 is wide, P2→P3 is narrow, P3→P4 is medium
- Lower facets mirror this asymmetry for crystal-like appearance

Triangulation: `ConvexPolygon::new_unchecked([P0..P9])` (CCW from P0 going up-left).

**For a more irregular/asymmetric crystal** (concave faces are NOT needed — just varied facet angles):
```
P0 = (-1.0,  0.0)
P1 = (-0.6,  0.35)
P2 = ( 0.0,  0.5)
P3 = ( 0.4,  0.45)   (asymmetric top — shifts facet angle)
P4 = ( 0.85, 0.2)
P5 = ( 1.0,  0.0)
P6 = ( 0.7, -0.3)    (asymmetric bottom)
P7 = ( 0.2, -0.5)
P8 = (-0.4, -0.4)
P9 = (-0.8, -0.15)
```

---

## 8. Summary: How to Create Each Shape

| Shape       | API                                                            | Notes |
|-------------|----------------------------------------------------------------|-------|
| Circle      | `meshes.add(Circle::new(1.0))`                                 | Confirmed in project code |
| Rectangle   | `meshes.add(Rectangle::new(1.0, 1.0))`                        | Confirmed in project code |
| Hexagon     | `meshes.add(RegularPolygon::new(r, 6))`                        | |
| Octagon     | `meshes.add(RegularPolygon::new(r, 8))`                        | |
| Diamond     | `meshes.add(Rhombus::new(w, h))`                               | `w` = horizontal diagonal total |
| Pill/Stadium| `meshes.add(Capsule2d::new(radius, length))`                   | Rounded ends only |
| Custom convex | `meshes.add(ConvexPolygon::new_unchecked(verts))`            | Verts must be CCW convex |
| Fully custom | `Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD)` + `insert_attribute` + `insert_indices` | |

---

## 9. Project-Specific Notes

- The project uses **unit-size meshes** (1.0 × 1.0) and applies scale via `Scale2D`. Custom shapes
  should follow this convention: design vertices in a normalized bounding box [-1..1, -0.5..0.5]
  and let `Scale2D` handle the actual size.
- `ColorMaterial` is the material type used for solid-color 2D meshes (no textures needed for
  paddle shapes). `MeshMaterial2d<ColorMaterial>` is the component.
- NORMAL and UV attributes are technically optional for `ColorMaterial` rendering (no textures,
  no lighting in 2D), but Bevy may require them for some render pipelines. Include them to be safe.
- `RenderAssetUsages::RENDER_WORLD` is correct for static meshes — saves memory vs `MAIN_WORLD | RENDER_WORLD`.

---

## 10. Gotchas

1. **`Mesh::new()` requires `RenderAssetUsages` in 0.18** — the 2-arg constructor. Do NOT use
   `Mesh::new(PrimitiveTopology::TriangleList)` (old 1-arg form from pre-0.14).

2. **`ConvexPolygon::new()` returns `Result`** — use `new_unchecked()` when vertices are known
   to be convex, or handle the error. The `Concave` error is the only failure mode.

3. **Winding order**: counter-clockwise for front faces. Bevy's 2D renderer does not cull back
   faces by default, but use CCW to be safe.

4. **Rhombus is NOT a rotated square**: `Rhombus::new(w, h)` creates a true rhombus with
   independently settable diagonals. Use it for diamond shapes directly.

5. **No built-in RoundedRectangle**: Use `Capsule2d` for pill shapes, or build a manual mesh.

6. **Z=0 for all 2D meshes**: ATTRIBUTE_POSITION values must have `[x, y, 0.0]`.

7. **Import paths for manual mesh**: `Indices` and `PrimitiveTopology` are NOT in `bevy::prelude`.
   They need explicit imports:
   ```rust
   use bevy::render::mesh::Indices;
   use bevy::render::render_resource::PrimitiveTopology;
   use bevy::render::render_asset::RenderAssetUsages;
   ```
