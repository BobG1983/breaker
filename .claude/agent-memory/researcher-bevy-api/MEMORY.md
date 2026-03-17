# Bevy API Expert Memory — Brickbreaker Project

## Project: Bevy 0.18.1, `default-features = false, features = ["2d"]`

## Verified Feature Flags (v0.18.0/0.18.1 — same feature set)

- `default = ["2d", "3d", "ui"]`
- `"2d"` profile includes: default_app, default_platform, 2d_bevy_render, ui, scene, audio, picking
  - This means `"2d"` ALREADY includes bevy_ui, bevy_audio, bevy_scene, bevy_sprite, picking
  - Do NOT need to add `"bevy_ui"` separately when using `features = ["2d"]`
- `dynamic_linking = ["dep:bevy_dylib", "bevy_internal/dynamic_linking"]` — dev only, never release

## Fast Compile — macOS (verified from bevy.org setup guide)

- macOS uses `ld-prime` (Xcode) by default — NO custom linker config needed
- Linux: `linker = "clang"`, `rustflags = ["-C", "link-arg=-fuse-ld=lld"]`
- Windows: `linker = "rust-lld.exe"`

## Fast Compile — Cargo.toml (canonical settings)

```toml
[profile.dev]
opt-level = 1

[profile.dev.package."*"]
opt-level = 3       # Most important — makes Bevy renderer usable in dev

[profile.release]
codegen-units = 1
lto = "thin"
```

## dynamic_linking Pattern

- Pass as `--features bevy/dynamic_linking` at CLI, NOT in Cargo.toml features list
- Alias in .cargo/config.toml: `dev = "run --features bevy/dynamic_linking"`
- NEVER add dynamic_linking to Cargo.toml features — breaks release builds

## Key API Facts

- No SpriteBundle/NodeBundle — use required components + tuples
- `commands.spawn(Camera2d)` — not CameraBundle; Camera2d is a zero-sized marker component; required components (Camera, Projection, Frustum) auto-inserted
- To override Camera2d's default projection, include `Projection::from(OrthographicProjection { scaling_mode: ScalingMode::AutoMin { min_width: 1920.0, min_height: 1080.0 }, ..OrthographicProjection::default_2d() })` in the spawn tuple
- `OrthographicProjection` fields: `near: f32`, `far: f32`, `viewport_origin: Vec2`, `scaling_mode: ScalingMode`, `scale: f32`, `area: Rect`
- `OrthographicProjection::default_2d()` — sets near to a negative value (enables z-layering with positive z coords)
- `ScalingMode` variants: `WindowSize`, `Fixed { width, height }`, `AutoMin { min_width, min_height }`, `AutoMax { max_width, max_height }`, `FixedVertical { viewport_height }`, `FixedHorizontal { viewport_width }`
- Import path: `bevy::camera::OrthographicProjection` and `bevy::camera::ScalingMode` (also in prelude)
- `Sprite::from_image(handle)` or `Sprite::from_atlas_image(handle, atlas)`
- `Sprite::from_color(color, Vec2)` — solid-color rectangle; requires NO image asset
- Sprite fields: `image`, `texture_atlas`, `color`, `flip_x`, `flip_y`, `custom_size`, `rect`, `image_mode`
- Paddle/brick pattern (verified from official breakout example):
  - `Sprite::from_color(COLOR, Vec2::ONE)` + `Transform { scale: size.extend(1.0), translation: pos.extend(0.0), ..default() }`
  - OR `Sprite { color: COLOR, ..default() }` + `Transform { scale: Vec3::new(w, h, 1.0), translation: ..., ..default() }`
  - z-scale of 2D objects MUST always be 1.0
- Ball/bolt pattern: `Mesh2d(meshes.add(Circle::default()))` + `MeshMaterial2d(materials.add(color))` + `Transform`
- Colored 2D shapes (non-sprite): `Mesh2d(handle)` + `MeshMaterial2d(materials.add(color))` + `Transform`
- `ButtonInput<KeyCode>`: `Res<ButtonInput<KeyCode>>` — `.pressed(KeyCode::ArrowLeft)`, `.just_pressed()`, `.just_released()`
- `FixedUpdate`: valid schedule label, use `app.add_systems(FixedUpdate, system)` — runs at 64 Hz default
- `run_if(in_state(S::Variant))` works with FixedUpdate — no known gotchas in 0.18.1
- `Transform::from_xyz(x, y, z)` or `Transform::from_translation(Vec3)` for positioning
- Messages: `#[derive(Message)]`, `MessageWriter<T>`, `MessageReader<T>`, `app.add_message::<M>()`
  - `add_message` lives directly on `App` (not a separate extension trait)
  - Signature: `pub fn add_message<M>(&mut self) -> &mut App where M: Message`
  - Inserts `Messages<M>` resource and schedules `message_update_system` in `First`
- Events (observable/triggered only): `#[derive(Event)]` — NOT for game messages

## States API (verified 0.18.1)

- `#[derive(States)]` — requires `Clone + PartialEq + Eq + Hash + Debug + Default` on the type
- `app.init_state::<S>()` — from `AppExtStates` trait; bound: `S: FreelyMutableState + FromWorld`
- `app.insert_state(value)` — for a specific initial value
- `OnEnter<S>(pub S)` and `OnExit<S>(pub S)` — schedule label structs; parameterized by S: States
- `in_state(s: S) -> impl FnMut(Option<Res<State<S>>>) + Clone` — run condition; in prelude as `in_state`

## PluginGroupBuilder (verified 0.18.1)

- `PluginGroupBuilder::start::<PG>() -> PluginGroupBuilder` — CORRECT constructor
- `.add<T: Plugin>(self, plugin: T)`, `.add_before::<Target>`, `.add_after::<Target>`, `.disable::<T>()`
- `.finish(self, app: &mut App)` — called internally by Bevy

## MinimalPlugins (verified 0.18.1)

- Exists and is in the prelude
- Includes: TaskPoolPlugin, FrameCountPlugin, TimePlugin, ScheduleRunnerPlugin
- Good for headless tests — no window, no renderer overhead

## Window Configuration (verified 0.18.1)

- `Window` fields: `title: String`, `resolution: WindowResolution`, `mode: WindowMode`
- `WindowMode` variants: `Windowed` (default), `BorderlessFullscreen(MonitorSelection)`, `Fullscreen(MonitorSelection, VideoModeSelection)`
- **NO `WindowMode::Maximized`** — use `Window::set_maximized(true)` instead
- Configure at startup: `DefaultPlugins.set(WindowPlugin { primary_window: Some(Window { .. }), .. })`
- Query/mutate at runtime: `Query<&mut Window, With<PrimaryWindow>>`

## Third-Party Crate Compatibility

See [third_party_crates.md](third_party_crates.md) for full details. Key facts:
- bevy_egui 0.39.1: UI systems in `EguiPrimaryContextPass` schedule (NOT Update)
- bevy_common_assets 0.15: requires `ron = "0.11"` — BREAKS if project uses `ron = "0.8"`
- bevy_asset_loader 0.25 + iyes_progress 0.16: `ProgressPlugin` MUST be added BEFORE `LoadingState`
- iyes_progress: `ProgressEntry` system param is the idiomatic way (no `.track_progress()` needed)

## Easing API (verified v0.18.1)

See [easing_api.md](easing_api.md) for full details. Key facts:
- `EaseFunction`: 39 variants, implements `Curve<f32>` directly — `.sample_clamped(t)` works on it
- Parametric variants: `Steps(usize, JumpAt)` and `Elastic(f32)`
- Derives `Serialize + Deserialize` — RON-serializable
- `EasingCurve::new(start, end, ease_fn)` — full typed animation curve for any `T: Ease + Clone`
- `Ease` implemented for: all VectorSpace types (Vec2, Vec3, f32), Rot2, Quat, Dir2/3, Isometry2d/3d, tuples
- No `Power(f32)` variant — use `.map(|v| v.powf(n))` on a `Curve<f32>` instead
- Curve composition: `.chain()`, `.zip()`, `.map()`, `.reparametrize()`, `.repeat()`, `.ping_pong()` on `CurveExt`

## Bloom + Tonemapping (verified v0.18.0 from official bloom_2d.rs example)

- Bloom import: `bevy::post_process::bloom::Bloom` (NOT bevy::core_pipeline)
- `Bloom` fields: `intensity`, `low_frequency_boost`, `low_frequency_boost_curvature`, `high_pass_frequency`, `prefilter`, `composite_mode`, `max_mip_dimension`, `scale`
- Presets: `Bloom::NATURAL`, `Bloom::ANAMORPHIC`, `Bloom::OLD_SCHOOL`, `Bloom::SCREEN_BLUR`
- `bevy_post_process` IS included in `"2d"` feature
- `Tonemapping::TonyMcMapface` REQUIRES `tonemapping_luts` feature — NOT in `"2d"`!
  - Safe (no LUT): None, Reinhard, ReinhardLuminance, AcesFitted, SomewhatBoringDisplayTransform
  - LUT-required: AgX, TonyMcMapface, BlenderFilmic
- Spawn: `(Camera2d, Tonemapping::ReinhardLuminance, Bloom::default(), DebandDither::Enabled)`

## Mesh2d + MeshMaterial2d + ColorMaterial (verified v0.18.1)

All three in `bevy::prelude`. Spawn: `(Mesh2d(meshes.add(Circle::new(r))), MeshMaterial2d(materials.add(ColorMaterial::from_color(color))), Transform::from_xyz(x,y,z))`.
- `ColorMaterial::from_color(color)` — shortcut; `color` field accepts HDR values >1.0 via `Color::srgb(7.5, 0.0, 7.5)`
- `Circle::new(radius)` and `Rectangle::new(width, height)` — both in `bevy::prelude`; Rectangle stores half-sizes internally

## Testing Time<Fixed> / FixedUpdate Systems (verified v0.18.0 source)

See [fixed_update_testing.md](fixed_update_testing.md) for full details. Key facts:
- Use `accumulate_overstep` (NOT `advance_by`) to trigger FixedUpdate ticks in tests
- `advance_by` does NOT deposit into the overstep accumulator — FixedUpdate will silently skip
- Register systems in `FixedUpdate` in tests, matching production — do NOT move to `Update` as workaround
- Clear inputs in `FixedPostUpdate`, NOT `PreUpdate` — prevents input loss on frames FixedUpdate skips

## KeyboardInput (see [keyboard_input.md](keyboard_input.md) for full details)

- `KeyboardInput` is a `Message` (NOT Event) — use `MessageReader<KeyboardInput>`, never `EventReader`
- Send in tests: `app.world_mut().write_message(KeyboardInput { ... })` — NOT `send_event()`
- System set is `InputSystems` (plural), NOT `InputSystem`

## System Output Discarding (verified v0.18.1)

- Systems added via `add_systems` must return `()` — use `.map(drop)` to discard any return value
- No `ignore` helper exists — `.map(drop)` is canonical

## AssetEvent (verified v0.18.1 source)

- `AssetEvent<A>` derives `Message` (NOT `Event`) — use `MessageReader<AssetEvent<A>>`, never `EventReader`
- Five variants, all with a single `id: AssetId<A>` field:
  `Added { id }`, `Modified { id }`, `Removed { id }`, `Unused { id }`, `LoadedWithDependencies { id }`
- Helper methods: `is_added(handle)`, `is_modified(handle)`, `is_removed(handle)`, `is_unused(handle)`, `is_loaded_with_dependencies(handle)` — all take `impl Into<AssetId<A>>`
- **No `.id()` method on AssetEvent** — access via helpers or pattern match `{ id }` field directly
- `is_modified(handle.id())` is the idiomatic hot-reload check
- `resource_changed::<T>()` run condition: `pub fn resource_changed<T: Resource>(res: Res<T>) -> bool`
- `resource_exists_and_changed::<T>()` — safe variant for resources that may not always exist
- `Res<T>::is_changed()` — from `DetectChanges` trait; true if added or mutably dereferenced since last system run
- `Res<T>::is_added()` — true if resource was newly inserted since last system run

## Observers, Triggers, One-Shot Systems (verified v0.18.1)

See [observers_and_oneshot.md](observers_and_oneshot.md) for full details. Key facts:
- `#[derive(Event)]` for observer-triggered events ONLY; game messages use `#[derive(Message)]`
- `commands.trigger(MyEvent{..})` — deferred; `world.trigger()` — immediate
- `app.add_observer(...)` global; `commands.entity(id).observe(...)` entity-local
- One-shot: `world.register_system(fn) -> SystemId`, `world.run_system(id)`

## Hierarchy API (verified v0.18.1)

See [hierarchy.md](hierarchy.md) for full details. Key facts:
- Parent component is `ChildOf` (NOT `Parent` — renamed); in `bevy::prelude`
- `bevy_hierarchy` crate no longer exists — merged into `bevy_ecs`
- `ChildOf(pub Entity)` — `.parent()` method or `.0` field
- Spawn: `commands.entity(parent).with_child(bundle)` or `world.spawn(ChildOf(parent_entity))`
- `with_children` closure param types: `ChildSpawnerCommands<'_>` (commands) / `ChildSpawner<'w>` (world)
- Import: `bevy::ecs::hierarchy::{ChildSpawner, ChildSpawnerCommands}` (NOT in prelude)
- `ChildBuilder` does NOT exist in 0.18

## Transform Interpolation (verified v0.18.1)

See [transform_interpolation.md](transform_interpolation.md). No built-in support — manual pattern or `bevy_transform_interpolation` crate.

## Headless App / ScheduleRunnerPlugin / AppExit / LogPlugin (verified v0.18.1)

See [headless_app.md](headless_app.md) for full details. Key facts:
- Headless: `WindowPlugin { primary_window: None, exit_condition: ExitCondition::DontExit }` + `.disable::<WinitPlugin>()`
- `"2d"` feature INCLUDES `bevy_winit` (via default_platform) — MUST disable WinitPlugin for headless
- `ScheduleRunnerPlugin::run_once()` for single-run; `::run_loop(Duration)` for continuous
- `AppExit` is a Message (NOT Event) — send via `MessageWriter<AppExit>` or `world.write_message(AppExit::Success)`
- `AppExit::Error(NonZero<u8>)` — NOT `NonZeroU8`; helper: `AppExit::error()` = code 1
- `LogPlugin::custom_layer` is `fn(&mut App) -> Option<BoxedLayer>` (fn pointer, not closure field)
- `BoxedLayer = Box<dyn Layer<Registry> + Send + Sync + 'static>` where Registry = tracing_subscriber::Registry

## Entities Counting API (verified v0.18.1)

- `world.entities().count_spawned() -> u32` — count of live spawned entities; O(n), for diagnostics/tests only
- `world.entities().len() -> u32` — count of allocated slots (includes reserved but unspawned); WRONG for entity counting
- `total_count()` does NOT exist on `Entities` in 0.18.1
- `&World` is valid as a readonly `SystemParam` alongside `Res<>` and `ResMut<>` params

## Session History

See [ephemeral/](ephemeral/) — not committed.
