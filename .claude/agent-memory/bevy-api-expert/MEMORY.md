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

- `Window` component has `title: String`, `resolution: WindowResolution`, and `mode: WindowMode` fields
- `WindowMode` variants: `Windowed` (default), `BorderlessFullscreen(MonitorSelection)`, `Fullscreen(MonitorSelection, VideoModeSelection)`
- **There is NO `WindowMode::Maximized` variant** — use `Window::set_maximized(true)` method instead
- `set_maximized(true)` sets `internal.maximize_request = Some(true)` — there is no `maximized: bool` field
- To start maximized: set it in the primary_window config OR via a startup system querying `Query<&mut Window, With<PrimaryWindow>>`
- Configure at startup via DefaultPlugins:
  ```rust
  DefaultPlugins.set(WindowPlugin {
      primary_window: Some(Window {
          title: "My Game".into(),
          resolution: WindowResolution::new(1280.0, 720.0),
          ..default()
      }),
      ..default()
  })
  ```
- Query/mutate at runtime: `Query<&mut Window, With<PrimaryWindow>>`

## bevy_egui (verified)

- bevy_egui 0.39.1 is compatible with Bevy 0.18.x
- UI systems go in `EguiPrimaryContextPass` schedule (NOT Update)
- Also has `EguiPreUpdateSet` / `EguiPostUpdateSet` system sets

## bevy_common_assets 0.15 (verified against Bevy 0.18.0)

- Depends on `ron = "0.11"` (via `serde_ron` alias) — INCOMPATIBLE with `ron = "0.8"` in this project!
- `ron` must be upgraded from `"0.8"` to `"0.11"` in Cargo.toml when adding this crate
- Feature flag: `bevy_common_assets = { version = "0.15", features = ["ron"] }`
- `RonAssetPlugin<A>` requires: `for<'de> A: Deserialize<'de>` + `A: Asset`
- Constructor: `pub fn new(extensions: &[&'static str]) -> Self`
- Usage: `app.add_plugins(RonAssetPlugin::<MyData>::new(&["mydata.ron"]))`
- The `ron` crate is NOT bundled — user must add `ron = "0.11"` separately
- Import: `use bevy_common_assets::ron::RonAssetPlugin;`
- Source: docs.rs/bevy_common_assets/0.15.0, raw Cargo.toml confirmed `serde_ron = "0.11"`

## bevy_asset_loader 0.25 + iyes_progress 0.16 (verified)

- Feature flag: `bevy_asset_loader = { version = "0.25", features = ["progress_tracking"] }`
- Must also add `iyes_progress = "0.16"` directly (bevy_asset_loader does NOT register ProgressPlugin)
- `Progress` struct: `pub done: u32, pub total: u32` — implements `Into<f32>` (0.0–1.0 ratio)
- `HiddenProgress(pub Progress)` — blocks transition but invisible to `get_global_progress()`
- `ProgressTracker<S>: Resource` — `get_global_progress() -> Progress`, `is_ready() -> bool`
- Systems returning `Progress`/`HiddenProgress` use `.track_progress::<S>()` or `.track_progress_and_stop::<S>()`
- `ProgressEntry` system param: `set_progress(done, total)`, `set_total(u32)`, `set_done(u32)`, `add_progress(done, total)`, `add_total(u32)`, `add_done(u32)`, `is_ready() -> bool`, `is_global_ready() -> bool`, `get_global_progress() -> Progress` — no `.track_progress()` needed, registers itself
- `ProgressPlugin::<S>::new().with_state_transition(from, to)` drives the state change, NOT `LoadingState::continue_to_state`
- `ProgressPlugin` MUST be added BEFORE `LoadingState` plugin in the app builder
- `finally_init_resource::<R>()` on `LoadingState` does NOT count toward progress tracking; runs after assets loaded, before transition
- Check schedule: `Last` by default; override with `.check_progress_in(schedule)`
- `ProgressPlugin` + `LoadingState` together: assets auto-contribute to ProgressTracker when feature active
- Clippy warning: `u32 as f32` cast triggers `cast_precision_loss` — use `Into::<f32>::into(progress)` instead
- Sources: docs.rs/bevy_asset_loader/0.25.0, docs.rs/iyes_progress/0.16.0, github.com/IyesGames/iyes_progress v0.16.0 full.rs example

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

- Bloom canonical import: `bevy::post_process::bloom::Bloom` (NOT bevy::core_pipeline)
- BloomCompositeMode: `bevy::post_process::bloom::BloomCompositeMode`
- `Bloom` fields: `intensity: f32`, `low_frequency_boost: f32`, `low_frequency_boost_curvature: f32`,
  `high_pass_frequency: f32`, `prefilter: BloomPrefilter`, `composite_mode: BloomCompositeMode`,
  `max_mip_dimension: u32`, `scale: Vec2`
- `Bloom::default()` — works; also presets: `Bloom::NATURAL`, `Bloom::ANAMORPHIC`, `Bloom::OLD_SCHOOL`, `Bloom::SCREEN_BLUR`
- `bevy_post_process` IS included in `"2d"` feature (via `2d_bevy_render`)
- Tonemapping: `bevy::core_pipeline::tonemapping::Tonemapping`
- `Tonemapping::TonyMcMapface` REQUIRES `tonemapping_luts` Cargo feature — NOT included in `"2d"`!
  - Safe variants (no LUT): None, Reinhard, ReinhardLuminance, AcesFitted, SomewhatBoringDisplayTransform
  - LUT-required variants: AgX, TonyMcMapface, BlenderFilmic
- Camera bloom setup pattern (from official example):
  ```rust
  commands.spawn((Camera2d, Tonemapping::TonyMcMapface, Bloom::default(), DebandDither::Enabled));
  // BUT: TonyMcMapface needs features = ["2d", "tonemapping_luts"] in Cargo.toml!
  ```

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

## KeyboardInput (see keyboard_input.md for full details)

- `KeyboardInput` is a `Message` (NOT Event) — use `MessageReader<KeyboardInput>`, never `EventReader`
- Send in tests: `app.world_mut().write_message(KeyboardInput { ... })` — NOT `send_event()`
- System set is `InputSystems` (plural), NOT `InputSystem`

## System Output Discarding (verified v0.18.1 source)

- `add_systems` requires `ScheduleSystem = BoxedSystem<(), ()>` — output MUST be `()`
- `.map(drop)` on any system discards its return value: `my_system.map(drop)`
- Method on `IntoSystem` trait: `fn map<T, F>(self, f: F) -> IntoAdapterSystem<F, Self>`
  where `F: Send + Sync + 'static + FnMut(Out) -> T`
- `drop` is `fn drop<T>(_: T)` — satisfies `FnMut(Progress) -> ()` exactly
- Verified in source: `bevy_ecs-0.18.1/src/system/mod.rs:224` and the official doc example
- No `ignore` helper exists — `.map(drop)` is the canonical pattern, shown in Bevy's own docs

## Observers, Triggers, One-Shot Systems (verified v0.18.1)

See [observers_and_oneshot.md](observers_and_oneshot.md) for full details. Key facts:
- `#[derive(Event)]` + `commands.trigger(MyEvent{..})` — deferred (at cmd flush); `world.trigger()` — immediate
- `#[derive(EntityEvent)]` — targets specific entities; use `commands.trigger_targets(e, entity)`
- `app.add_observer(|e: On<MyEvent>, ...| {...})` — global; `commands.entity(id).observe(...)` — entity-local
- `Observer::new(fn).with_entities([a,b])` — multi-entity observer, built BEFORE spawn; cannot retarget after
- Observers run synchronously, outside the schedule, in registration order; can chain via `commands.trigger()`
- `commands.trigger()` fires at next cmd flush; `world.trigger()` fires immediately (needs exclusive world)
- `On<E>`: implements `Deref<Target=E>`, `e.observer()` returns observer entity, `e.propagate(bool)` for EntityEvent
- Component hooks: `#[component(on_add = fn)]` — one hook per lifecycle, runs before observers
- Built-in `On<Add, C>` / `On<Remove, C>` observers — multiple observers CAN watch same lifecycle
- One-shot: `world.register_system(fn) -> SystemId`, `world.run_system(id)`, `world.run_system_with(id, input)`
- `register_system_cached` / `run_system_cached` — for zero-sized fn pointers, no manual id storage needed
- Dynamic schedule addition NOT recommended at runtime — use one-shot systems instead
- `#[derive(Event)]` is for observer-triggered events ONLY; game messages use `#[derive(Message)]`

## Hierarchy API (verified v0.18.1 source: bevy_ecs-0.18.1/src/hierarchy.rs)

- The parent component is `ChildOf`, NOT `Parent` — `Parent` does not exist in 0.18.1
- `ChildOf` is in `bevy::prelude`; `bevy_hierarchy` crate no longer exists (merged into `bevy_ecs`)
- Definition: `pub struct ChildOf(#[entities] pub Entity);` — tuple struct wrapping the parent `Entity`
- `#[doc(alias = "Parent")]` on `ChildOf` — confirms `Parent` was renamed to `ChildOf`
- Method: `pub fn parent(&self) -> Entity` — returns the parent Entity
- Direct field access also works: `child_of.0`
- In queries: `Query<&ChildOf, With<MyMarker>>`; call `.parent()` on the result
- `Children` component: lives on the PARENT, contains `Vec<Entity>` of child entity ids
- Hierarchy is maintained automatically via component hooks — never manually mutate `Children`
- `ChildOf` self-removes if parent is despawned or if entity tries to parent itself (hooks validate)
- Spawn pattern: `world.spawn(ChildOf(parent_entity))` or via `commands.entity(parent).with_child(bundle)`
- `with_child(bundle)` on `EntityCommands` spawns one child and inserts `ChildOf` automatically
- `children![]` macro: `world.spawn((Name::new("Root"), children![Name::new("Child1")]))`

## Sources

- Feature flags: https://docs.rs/bevy/0.18.1/bevy/index.html#cargo-features
- Raw Cargo.toml: https://raw.githubusercontent.com/bevyengine/bevy/v0.18.0/Cargo.toml
- Fast compile guide: https://bevy.org/learn/quick-start/getting-started/setup/
- Message API: https://docs.rs/bevy_ecs/0.18.1/bevy_ecs/message/index.html
- States API: https://docs.rs/bevy/0.18.1/bevy/state/app/trait.AppExtStates.html
- PluginGroupBuilder: https://docs.rs/bevy/0.18.1/bevy/app/struct.PluginGroupBuilder.html
- WindowPlugin: https://docs.rs/bevy/0.18.1/bevy/window/struct.WindowPlugin.html
- bevy_egui: https://docs.rs/bevy_egui/latest/bevy_egui/index.html
