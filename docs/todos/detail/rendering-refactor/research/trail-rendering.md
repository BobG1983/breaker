# Bevy 0.18.1 — 2D Motion Trail Rendering Research

> **Raw research.** Architecture decisions that differ from these findings are in `docs/architecture/rendering/`.
> Specifically: this doc recommends `ColorMaterial` with `AlphaMode2d::Blend` for ribbon trails.
> **Architecture decision**: custom `TrailRibbonMaterial` with alpha-weighted additive blending via `specialize()` — `src_factor: SrcAlpha, dst_factor: One`. Same blend mode as ParticleMaterial. Trails add light but fade to invisible (not black). See `rantzsoft_vfx.md` — Trail Rendering section.
> The API patterns in this research (TriangleStrip topology, ring buffer, `RenderAssetUsages::default()`, PostUpdate sampling) are still valid.

Verified against docs.rs/bevy/0.18.1, official Bevy examples, and migration guides.
Session date: 2026-03-30.

---

## Summary of Findings

Three trail types are feasible in Bevy 0.18.1 using only the built-in APIs (no extra crates
required). A fourth option — `bevy_hanabi` — exists for GPU particle-based trails but has
significant constraints. Findings for each type and all supporting API details follow.

---

## 1. Mesh Ribbon Trail (solid energy wake)

### Concept
Store a ring buffer of N world-space positions. Each frame, generate a quad-strip mesh:
two vertices per position sample (one on each side of the velocity tangent, offset by half
the trail width). The vertices near the head are full alpha; near the tail they are zero.

### Key Bevy 0.18.1 APIs

#### Spawning a dynamic 2D mesh
```rust
use bevy::{
    prelude::*,
    render::{
        mesh::PrimitiveTopology,
        render_asset::RenderAssetUsages,
    },
    sprite_render::{AlphaMode2d, MeshMaterial2d},
};

// Setup: create the mesh asset once and store the handle
#[derive(Resource)]
struct TrailMeshHandle(Handle<Mesh>);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // CRITICAL: must include MAIN_WORLD for per-frame mutation from main world
    let mesh = Mesh::new(
        PrimitiveTopology::TriangleStrip,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    let handle = meshes.add(mesh);
    commands.insert_resource(TrailMeshHandle(handle.clone()));

    commands.spawn((
        Mesh2d(handle),
        MeshMaterial2d(materials.add(ColorMaterial {
            color: Color::WHITE,
            alpha_mode: AlphaMode2d::Blend,
            ..default()
        })),
    ));
}
```

#### Per-frame mesh update
```rust
fn update_trail_mesh(
    trail_handle: Res<TrailMeshHandle>,
    mut meshes: ResMut<Assets<Mesh>>,
    trail: Res<TrailPositions>,  // ring buffer of Vec2 + age
) {
    let Some(mesh) = meshes.get_mut(&trail_handle.0) else { return; };

    let (positions, colors): (Vec<[f32; 3]>, Vec<[f32; 4]>) = trail
        .samples
        .iter()
        .enumerate()
        .flat_map(|(i, sample)| {
            let t = i as f32 / trail.samples.len() as f32;
            let alpha = 1.0 - t;
            let perp = sample.perp * TRAIL_HALF_WIDTH * (1.0 - t * 0.5);
            [
                ([sample.pos.x + perp.x, sample.pos.y + perp.y, 0.0],
                 [1.0, 1.0, 1.0, alpha]),
                ([sample.pos.x - perp.x, sample.pos.y - perp.y, 0.0],
                 [1.0, 1.0, 1.0, alpha]),
            ]
        })
        .unzip();

    // In Bevy 0.18, use try_insert_attribute to avoid panic on extracted meshes
    // (insert_attribute panics if mesh was extracted to render world with RENDER_WORLD only)
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    // No indices needed for TriangleStrip — sequential vertices form triangles
}
```

### PrimitiveTopology choice
- **`TriangleStrip`** is ideal for ribbon trails. Vertices `[L0, R0, L1, R1, L2, R2]`
  automatically form quad pairs: `L0 R0 L1`, `L1 R0 R1`, `L1 R1 L2`, etc.
- **`TriangleList`** works too but requires explicit index data (2 triangles per quad segment =
  6 indices per pair of samples). TriangleStrip is more memory-efficient.

### CRITICAL: RenderAssetUsages in Bevy 0.18
**This is a Bevy 0.18-specific gotcha** (introduced by retained render world in 0.18):

- `RenderAssetUsages::RENDER_WORLD` only: mesh data extracted and inaccessible from main world.
  Calling `meshes.get_mut()` or `mesh.insert_attribute()` panics with
  `"Mesh has been extracted to RenderWorld. To access vertex attributes, the mesh asset_usage
  must include MAIN_WORLD"`.
- `RenderAssetUsages::default()` = `MAIN_WORLD | RENDER_WORLD`: safe, allows per-frame updates.
- For per-frame dynamic trails, always use `RenderAssetUsages::default()` or explicitly
  `RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD`.
- Alternatively, use `try_insert_attribute()` which returns `Result<_, MeshAccessError>` instead
  of panicking — graceful degradation if somehow the mesh gets extracted-only.
- In Bevy 0.18, the `Aabb` is automatically updated after mesh mutation — no need to manually
  remove and re-insert the Aabb component (this WAS required in 0.17).

### `Assets::get_mut` performance note
Calling `Assets::get_mut()` unconditionally sends `AssetEvent::Modified`. For a trail updated
every frame, this is expected behavior. There is a known issue (PR #22460) tracking tools to
avoid unnecessary Modified events — not yet merged as of Bevy 0.18.1.

### Transform/world-space consideration
**The trail mesh entity should NOT be a child of the moving entity.**

If the trail mesh is a child entity, its local-space Transform is relative to the parent. But
trail vertex positions are world-space historical positions — they cannot be expressed in the
parent's local space without inverse-transforming every vertex each frame.

Instead:
- Trail mesh entity is a **top-level entity** (no parent).
- Its `Transform` is `Transform::IDENTITY` (or fixed at origin).
- All vertex positions are written in world space.
- The system queries `GlobalTransform::translation()` on the moving entity to sample positions.

To read world-space position of the moving entity:
```rust
fn sample_bolt_position(
    bolt_query: Query<&GlobalTransform, With<Bolt>>,
    mut trail: ResMut<TrailPositions>,
) {
    for gt in &bolt_query {
        let world_pos: Vec3 = gt.translation();
        trail.push(world_pos.truncate()); // Vec2
    }
}
```

### Vertex attributes needed
- `Mesh::ATTRIBUTE_POSITION` — `Vec<[f32; 3]>` (z = 0.0 for 2D, or use z for layering)
- `Mesh::ATTRIBUTE_COLOR` — `Vec<[f32; 4]>` (RGBA, fading alpha along trail)
- Optionally `Mesh::ATTRIBUTE_UV_0` for a texture-mapped trail

### ColorMaterial for ribbon trail
```rust
ColorMaterial {
    color: Color::WHITE,          // tint; WHITE = vertex colors show through
    alpha_mode: AlphaMode2d::Blend,
    texture: None,                // or Some(trail_texture_handle)
    ..default()
}
```
- `AlphaMode2d::Blend` is required for alpha transparency. `Opaque` ignores alpha.
- Vertex colors on `Mesh::ATTRIBUTE_COLOR` are multiplied by `ColorMaterial.color`.
- For per-vertex color fading: set `ColorMaterial.color = Color::WHITE`, put all
  alpha variation in the vertex color attribute.

---

## 2. Afterimage Trail (fading sprite copies)

### Concept
Maintain a fixed pool of N sprite entities at historical positions, each with progressively
lower alpha. No mesh generation needed — just repositioning existing entities and updating
their color alpha.

### Implementation pattern
Pre-spawn N "ghost" sprite entities in setup. Each frame, rotate which entity gets the
"freshest" position (ring buffer indexing). Compute alpha per ghost as `(N - age) / N`.

```rust
const AFTERIMAGE_COUNT: usize = 8;

#[derive(Component)]
struct Afterimage {
    index: usize,  // 0 = freshest, N-1 = oldest
}

#[derive(Resource)]
struct AfterimagePool {
    entities: [Entity; AFTERIMAGE_COUNT],
    write_head: usize,
}

fn setup_afterimages(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let mut entities = [Entity::PLACEHOLDER; AFTERIMAGE_COUNT];
    let image = asset_server.load("sprites/bolt.png");
    for i in 0..AFTERIMAGE_COUNT {
        entities[i] = commands.spawn((
            Sprite {
                image: image.clone(),
                color: Color::srgba(1.0, 1.0, 1.0, 0.0), // start invisible
                ..default()
            },
            Transform::default(),
            Afterimage { index: i },
        )).id();
    }
    commands.insert_resource(AfterimagePool {
        entities,
        write_head: 0,
    });
}

fn update_afterimages(
    bolt_query: Query<(&GlobalTransform, &Sprite), With<Bolt>>,
    mut pool: ResMut<AfterimagePool>,
    mut sprite_query: Query<(&mut Sprite, &mut Transform), With<Afterimage>>,
) {
    let Ok((bolt_gt, bolt_sprite)) = bolt_query.get_single() else { return; };
    let world_pos = bolt_gt.translation();

    // Write newest position to write_head slot
    let new_entity = pool.entities[pool.write_head];
    if let Ok((mut sprite, mut transform)) = sprite_query.get_mut(new_entity) {
        transform.translation = world_pos;
        sprite.color = Color::srgba(1.0, 1.0, 1.0, 1.0 / AFTERIMAGE_COUNT as f32);
    }

    // Fade all other slots based on age
    for slot in 0..AFTERIMAGE_COUNT {
        let age = (pool.write_head + AFTERIMAGE_COUNT - slot) % AFTERIMAGE_COUNT;
        let entity = pool.entities[slot];
        if let Ok((mut sprite, _)) = sprite_query.get_mut(entity) {
            let alpha = 1.0 - (age as f32 / AFTERIMAGE_COUNT as f32);
            sprite.color = Color::srgba(1.0, 1.0, 1.0, alpha * 0.8);
        }
    }

    pool.write_head = (pool.write_head + 1) % AFTERIMAGE_COUNT;
}
```

### Why entity pool (not spawn/despawn)?
- Spawning/despawning N entities every frame is expensive (archetype changes, allocations).
- Pre-spawning a fixed pool and updating component data is O(N) component writes — cheap.
- Bevy sprites with `Color::srgba(_, _, _, 0.0)` are transparent but still rendered (small GPU
  cost). Alternative: use `Visibility::Hidden` for truly invisible slots.

### Sprite vs Mesh2d for afterimage
- **Sprite** is simpler: just set `sprite.color` alpha. No mesh, no material handle.
- **Mesh2d + ColorMaterial**: requires `materials.get_mut(handle)` per ghost per frame — more
  overhead. Use Sprite unless you need custom geometry.

### Sprite struct (Bevy 0.18.1 confirmed)
```rust
pub struct Sprite {
    pub image: Handle<Image>,
    pub texture_atlas: Option<TextureAtlas>,
    pub color: Color,
    pub flip_x: bool,
    pub flip_y: bool,
    pub custom_size: Option<Vec2>,
    pub rect: Option<Rect>,
    pub image_mode: SpriteImageMode,
}
```
Required components auto-inserted: `Transform`, `Visibility`, `VisibilityClass`, `Anchor`.
Constructors: `Sprite::from_image(handle)`, `Sprite::from_color(color, size)`,
`Sprite::sized(size)`, `Sprite::from_atlas_image(handle, atlas)`.

---

## 3. Prismatic / Spectral Trail

### Concept
A trail that splits into spectral colors (R, G, B channels offset, like a prism). Two
implementation approaches:

### Approach A: Multiple overlapping ribbon trails (no custom shader)
Spawn 3 ribbon trail entities (or more for smoother spectrum), each with a different
`ColorMaterial` color and a slight position offset:
- Red trail: offset slightly in one direction, `ColorMaterial.color = RED.with_alpha(0.7)`
- Green trail: no offset, `ColorMaterial.color = GREEN.with_alpha(0.7)`
- Blue trail: offset in opposite direction, `ColorMaterial.color = BLUE.with_alpha(0.7)`
- All use `AlphaMode2d::Blend`
- Layer them with different Z values to control blend order

This is the simplest approach — pure ECS, no custom shaders.

### Approach B: Custom Material2d with WGSL shader (single mesh)
Define a custom material that takes a `t` uniform (0→1 head→tail) and maps it to a
spectral gradient in WGSL.

```rust
#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
struct PrismaticTrailMaterial {
    #[uniform(0)]
    head_color: LinearRgba,
    #[uniform(1)]
    tail_color: LinearRgba,
}

impl Material2d for PrismaticTrailMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/prismatic_trail.wgsl".into()
    }
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}
```

Register with:
```rust
app.add_plugins(Material2dPlugin::<PrismaticTrailMaterial>::default());
```

Spawn with:
```rust
commands.spawn((
    Mesh2d(trail_mesh_handle),
    MeshMaterial2d(prismatic_materials.add(PrismaticTrailMaterial { ... })),
));
```

WGSL fragment shader concept (prismatic_trail.wgsl):
```wgsl
@fragment
fn fragment(in: VertexOutput) -> @location(0) vec4<f32> {
    // in.uv.x = position along trail (0=head, 1=tail)
    // in.uv.y = position across trail width (0=left, 1=right)
    let t = in.uv.x;
    let alpha = 1.0 - t;
    // Spectral split: R channel leads, B channel lags
    let r = mix(1.0, 0.0, clamp(t - 0.0, 0.0, 1.0));
    let g = mix(1.0, 0.0, clamp(t - 0.1, 0.0, 1.0));
    let b = mix(1.0, 0.0, clamp(t - 0.2, 0.0, 1.0));
    return vec4<f32>(r, g, b, alpha * in.color.a);
}
```

For this to work, `Mesh::ATTRIBUTE_UV_0` must be set on the mesh (u = trail progress
0→1, v = cross-width 0→1).

### Approach comparison
| | Approach A (multi-ribbon) | Approach B (custom shader) |
|---|---|---|
| Complexity | Low — pure ECS | High — WGSL required |
| GPU cost | 3x draw calls | 1 draw call |
| Visual quality | Good for simple spectral | Precise, continuous gradient |
| Hot-reload friendly | Yes | Yes (asset server shader) |
| Recommended for | Prototype / fast implementation | Final visual polish |

---

## 4. bevy_hanabi (GPU particle system — NOT recommended for this use case)

`bevy_hanabi` 0.18 is compatible with Bevy 0.18. It supports 2D rendering via Camera2d and
includes ribbon/trail effects via `EffectAsset::with_ribbons()`.

**However**: bevy_hanabi is GPU-driven. The particle positions are computed on GPU and cannot
be read back to CPU. This means:
- Cannot position trail particles at exact Bolt world positions
- Trail is driven by emission position + velocity, not by exact historical ECS positions
- Not suitable for pixel-perfect trails that must exactly trace Bolt movement
- Suitable only for loose particle smoke/sparks, not geometric ribbon trails

**Recommendation**: Do not use bevy_hanabi for the three trail types described. Use the
pure-Bevy mesh/sprite approaches above.

---

## 5. `GlobalTransform` for sampling world positions

```rust
// Reading world-space position (confirmed Bevy 0.18.1):
fn sample_positions(query: Query<&GlobalTransform, With<Bolt>>) {
    for gt in &query {
        let world_pos: Vec3 = gt.translation();  // -> Vec3
        let pos_2d: Vec2 = world_pos.truncate(); // -> Vec2
    }
}
```

Other methods:
- `gt.translation()` → `Vec3`
- `gt.translation_vec3a()` → `Vec3A`
- `gt.to_isometry()` → isometric part (rotation + translation, no scale)
- `gt.compute_transform()` → `Transform` (decomposed scale/rot/trans)
- `gt.affine()` → `Affine3A`

**1-frame lag caveat**: `GlobalTransform` is updated during `PostUpdate` (TransformSystems::Propagate).
If you sample it in `Update` immediately after setting `Transform`, you get the previous frame's
world position. For trail sampling, this is fine — trail history is intentionally one step behind.

---

## 6. Transparency Quick Reference (Bevy 0.18.1)

### For Sprite-based afterimage trails:
```rust
// Set alpha directly on the Sprite color field:
sprite.color = Color::srgba(1.0, 1.0, 1.0, 0.5); // 50% alpha
```
Sprites are transparent by default when alpha < 1.0, no AlphaMode needed.

### For Mesh2d-based ribbon trails:
```rust
// ColorMaterial must use AlphaMode2d::Blend for alpha to work:
ColorMaterial {
    color: Color::srgba(1.0, 1.0, 1.0, 0.5), // or set via vertex colors
    alpha_mode: AlphaMode2d::Blend,
    ..default()
}
```
Available `AlphaMode2d` variants:
- `AlphaMode2d::Opaque` — ignores alpha, depth buffered
- `AlphaMode2d::Mask(threshold: f32)` — binary cut-off
- `AlphaMode2d::Blend` — standard alpha blending (required for trails)

`AlphaMode2d` is in `bevy::sprite_render::AlphaMode2d`.

---

## 7. Module Paths (Bevy 0.18.1 confirmed)

| Type | Module path |
|------|-------------|
| `Mesh` | `bevy::prelude::Mesh` |
| `Mesh2d` | `bevy::prelude::Mesh2d` (also `bevy::sprite_render::Mesh2d`) |
| `MeshMaterial2d<M>` | `bevy::prelude::MeshMaterial2d` (also `bevy::sprite_render::MeshMaterial2d`) |
| `ColorMaterial` | `bevy::prelude::ColorMaterial` (also `bevy::sprite_render::ColorMaterial`) |
| `AlphaMode2d` | `bevy::sprite_render::AlphaMode2d` |
| `Material2d` | `bevy::sprite_render::Material2d` |
| `Material2dPlugin<M>` | `bevy::sprite_render::Material2dPlugin` |
| `PrimitiveTopology` | `bevy::render::render_resource::PrimitiveTopology` |
| `RenderAssetUsages` | `bevy::render::render_asset::RenderAssetUsages` |
| `GlobalTransform` | `bevy::prelude::GlobalTransform` |
| `Sprite` | `bevy::prelude::Sprite` |

---

## 8. Recommended Implementation Strategy (for this project)

Given that:
- The project uses Bevy 0.18.1 with 2D features only
- The project uses game vocabulary: Bolt (ball), Breaker (paddle)
- The project has strict lint rules (no `#[allow(...)]`)

### Solid energy wake (ribbon mesh)
- Top-level entity (no parent), `Transform::IDENTITY`
- Ring buffer resource holding `Vec<TrailSample>` (world Vec2 + perp Vec2)
- Mesh with `RenderAssetUsages::default()`, `PrimitiveTopology::TriangleStrip`
- ColorMaterial with `AlphaMode2d::Blend`
- Update system in `PostUpdate` (after TransformSystems::Propagate so GlobalTransform is fresh)
- Use `mesh.insert_attribute()` (not `try_insert_attribute`) since MAIN_WORLD is guaranteed

### Fading afterimage copies
- Pre-spawned entity pool of N Sprite entities
- Ring buffer indexing, update alpha via `sprite.color`
- Sprites are top-level (no parent)
- Sample Bolt `GlobalTransform::translation()` in `PostUpdate`
- N = 6–10 is visually sufficient; beyond 16 gives diminishing returns

### Prismatic color split
- Start with Approach A (3 overlapping ribbon meshes, different ColorMaterial colors)
- Use Z-offset to layer: blue at z=0.0, green at z=0.1, red at z=0.2
- Slight positional offset per color channel (velocity-perpendicular direction)
- Upgrade to Approach B (custom Material2d shader) if visual quality demands it

---

## Sources consulted
- docs.rs/bevy/0.18.1 (Mesh, Sprite, ColorMaterial, GlobalTransform, RenderAssetUsages,
  Material2d, MeshMaterial2d, Mesh2d, PrimitiveTopology, AlphaMode2d)
- bevy.org/examples: mesh2d, mesh2d_alpha_mode, mesh2d_vertex_color_texture, transparency_2d,
  shader_material_2d
- github.com/bevyengine/bevy/blob/v0.18.1/examples/2d/mesh2d_manual.rs
- bevy.org/learn/migration-guides/0-17-to-0-18
- github.com/bevyengine/bevy/issues/22206 (retained render world mesh panic in 0.18)
- github.com/bevyengine/bevy/issues/18864 (RenderAssetUsages dynamic update fix)
- github.com/djeedai/bevy_hanabi README (version compatibility)
