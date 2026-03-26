---
name: Core API Facts
description: Verified Bevy 0.18.1 API patterns — spawning, queries, messages, states, UI, camera, assets
type: reference
---

## Key API Facts

- No SpriteBundle/NodeBundle — use required components + tuples
- `commands.spawn(Camera2d)` — Camera2d is a zero-sized marker; required components auto-inserted
- Override Camera2d projection: include `Projection::from(OrthographicProjection { scaling_mode: ScalingMode::AutoMin { min_width: 1920.0, min_height: 1080.0 }, ..OrthographicProjection::default_2d() })`
- `OrthographicProjection::default_2d()` — sets near to negative value (enables z-layering)
- `ScalingMode` variants: `WindowSize`, `Fixed { width, height }`, `AutoMin`, `AutoMax`, `FixedVertical`, `FixedHorizontal`
- `Sprite::from_image(handle)`, `Sprite::from_color(color, Vec2)`, `Sprite::from_atlas_image(handle, atlas)`
- Paddle/brick: `Sprite::from_color(COLOR, Vec2::ONE)` + `Transform { scale: size.extend(1.0), .. }`
- Ball/bolt: `Mesh2d(meshes.add(Circle::default()))` + `MeshMaterial2d(materials.add(color))` + `Transform`
- `ButtonInput<KeyCode>`: `.pressed()`, `.just_pressed()`, `.just_released()`
- `FixedUpdate`: valid schedule label, 64 Hz default
- `run_if(in_state(S::Variant))` works with FixedUpdate
- `Transform::from_xyz(x, y, z)` or `Transform::from_translation(Vec3)`

## Messages
- `#[derive(Message)]`, `MessageWriter<T>`, `MessageReader<T>`, `app.add_message::<M>()`
- `add_message` lives directly on `App` (not extension trait)
- `writer.write(msg)` returns `MessageId<E>` (NOT `send`)
- `reader.read()` returns `MessageIterator` yielding `&'a M`
- `reader.read_with_id()` yields `(&'a M, MessageId<M>)` pairs
- Events (observable/triggered only): `#[derive(Event)]` — NOT for game messages

## States API
- `#[derive(States)]` — requires Clone + PartialEq + Eq + Hash + Debug + Default
- `app.init_state::<S>()` — from `AppExtStates` trait; bound: `S: FreelyMutableState + FromWorld`
- `OnEnter<S>(pub S)` and `OnExit<S>(pub S)` — schedule label structs
- `in_state(s: S)` — run condition; in prelude

## PluginGroupBuilder
- `PluginGroupBuilder::start::<PG>()` — constructor
- `.add<T>()`, `.add_before::<Target>()`, `.add_after::<Target>()`, `.disable::<T>()`

## MinimalPlugins
- Includes: TaskPoolPlugin, FrameCountPlugin, TimePlugin, ScheduleRunnerPlugin
- Good for headless tests

## Window Configuration
- `Window` fields: `title`, `resolution: WindowResolution`, `mode: WindowMode`
- **NO `WindowMode::Maximized`** — use `window.set_maximized(true)` instead
- Configure at startup: `DefaultPlugins.set(WindowPlugin { primary_window: Some(Window { .. }), .. })`

## Bloom + Tonemapping
- `bevy::post_process::bloom::Bloom` (NOT bevy::core_pipeline)
- Presets: `Bloom::NATURAL`, `ANAMORPHIC`, `OLD_SCHOOL`, `SCREEN_BLUR`
- `bevy_post_process` IS included in `"2d"` feature
- `Tonemapping::TonyMcMapface` REQUIRES `tonemapping_luts` feature — NOT in `"2d"`
- Safe (no LUT): None, Reinhard, ReinhardLuminance, AcesFitted, SomewhatBoringDisplayTransform

## Mesh2d + MeshMaterial2d + ColorMaterial
- All in `bevy::prelude`
- `ColorMaterial::from_color(color)` — color field accepts HDR values >1.0
- `Circle::new(radius)` and `Rectangle::new(width, height)` — both in prelude
- `ColorMaterial` struct fields (verified from `bevy_sprite_render` 0.18.1 source):
  - `color: Color`
  - `alpha_mode: AlphaMode2d`
  - `uv_transform: Affine2`
  - `texture: Option<Handle<Image>>`
- `ColorMaterial` lives in `bevy_sprite_render` crate (NOT `bevy_sprite`) in 0.18.1
- Batching key for 2D meshes: `(Material2dBindGroupId, AssetId<Mesh>)` — entities sharing the same material handle AND same mesh handle batch into one draw call
- Unique material per entity = unique pipeline bind group per entity = NO batching

## Annulus Mesh (verified from 2d_shapes.rs example + bevy_mesh source, v0.18.1)
- `Annulus` is in `bevy::prelude`; struct has `inner_circle: Circle` and `outer_circle: Circle`
- Constructor: `Annulus::new(inner_radius: f32, outer_radius: f32) -> Annulus`
- Default: inner_radius=0.5, outer_radius=1.0
- Implements `Meshable` → returns `AnnulusMeshBuilder { annulus: Annulus, resolution: u32 }`
- `AnnulusMeshBuilder` methods: `.resolution(u32)` setter, `.build() -> Mesh`
- `From<AnnulusMeshBuilder> for Mesh` — so `meshes.add(builder)` works directly
- Idiomatic shorthand: `meshes.add(Annulus::new(inner, outer))` — `Assets<Mesh>::add` accepts anything `Into<Mesh>`, and `Annulus` itself (via its `AnnulusMeshBuilder`) converts via the same blanket impl
- **Confirmed pattern from example**: `meshes.add(Annulus::new(25.0, 50.0))` — no explicit `.mesh().build()` needed

## AlphaMode2d (verified from bevy_sprite_render docs, v0.18.1)
- Three variants:
  - `AlphaMode2d::Opaque` — alpha values overridden to 1.0 (fully opaque)
  - `AlphaMode2d::Mask(f32)` — pixels below threshold are transparent, above are opaque
  - `AlphaMode2d::Blend` — standard alpha blending; fragment alpha controls opacity
- `AlphaMode2d` is NOT in `bevy::prelude` — must import explicitly (from `bevy::sprite` or `bevy_sprite_render`)
- For a fading shockwave: set `alpha_mode: AlphaMode2d::Blend` and vary the alpha of the `Color`

## Vertex Colors on Mesh (ATTRIBUTE_COLOR)
- `Mesh::ATTRIBUTE_COLOR: MeshVertexAttribute` — `VertexFormat::Float32x4` (RGBA f32)
- Insert: `mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vec![[r, g, b, a], ...])`
  — one `[f32; 4]` entry per vertex; use `LinearRgba::to_f32_array()`
- Presence of `ATTRIBUTE_COLOR` on a mesh **automatically triggers** `#define VERTEX_COLORS` in the `color_material.wgsl` shader (set during pipeline specialization when the mesh vertex layout contains the color attribute)
- In the shader: `output_color = output_color * mesh.color;` — vertex color MULTIPLIES the material's uniform color
- Sharing one mesh handle + one material handle: all instances batch, but all share identical vertex colors
- Unique mesh per entity + vertex colors: no batching (each entity has its own mesh asset ID)
- Vertex colors do NOT break batching by themselves — batching depends on mesh asset ID + material bind group ID

## AssetEvent
- `AssetEvent<A>` derives `Message` — use `MessageReader<AssetEvent<A>>`, never `EventReader`
- Five variants: `Added`, `Modified`, `Removed`, `Unused`, `LoadedWithDependencies` — all with `id: AssetId<A>`
- Helper methods: `is_added(handle)`, `is_modified(handle)`, etc.
- `resource_changed::<T>()` and `resource_exists_and_changed::<T>()` run conditions

## System Output Discarding
- Systems added via `add_systems` must return `()` — use `.map(drop)` to discard
- No `ignore` helper — `.map(drop)` is canonical

## Entities Counting
- `world.entities().count_spawned() -> u32` — live entities; O(n), diagnostics only
- `world.entities().len() -> u32` — allocated slots (includes reserved); WRONG for counting
- `total_count()` does NOT exist in 0.18.1
