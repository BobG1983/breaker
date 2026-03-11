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

- `Window` component has `title: String` and `resolution: WindowResolution` fields
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

- `Mesh2d` canonical path: `bevy_mesh::Mesh2d` — re-exported via `bevy::prelude::*`
  - Tuple struct: `Mesh2d(pub Handle<Mesh>)`
  - Spawn: `Mesh2d(meshes.add(Circle::new(50.0)))`
- `MeshMaterial2d<M>` canonical path: `bevy_sprite_render::MeshMaterial2d` — re-exported via `bevy::prelude::*`
  - Tuple struct: `MeshMaterial2d<M: Material2d>(pub Handle<M>)`
  - Spawn: `MeshMaterial2d(materials.add(ColorMaterial::from_color(RED)))`
- `ColorMaterial` canonical path: `bevy_sprite_render::ColorMaterial` — re-exported via `bevy::prelude::*`
  - Fields: `color: Color`, `alpha_mode: AlphaMode2d`, `uv_transform: Affine2`, `texture: Option<Handle<Image>>`
  - Constructor shortcut: `ColorMaterial::from_color(color)`
  - HDR support: `color` field is `bevy::color::Color`; use `Color::srgb(7.5, 0.0, 7.5)` for HDR values > 1.0
    (Color::srgb accepts f32 values > 1.0 — no clamping at assignment; used this way in official bloom_2d example)
  - All three types are in `bevy::prelude` — `use bevy::prelude::*` is sufficient
- Spawn pattern for colored 2D circle with optional bloom:
  ```rust
  use bevy::prelude::*;
  commands.spawn((
      Mesh2d(meshes.add(Circle::new(50.0))),
      MeshMaterial2d(materials.add(ColorMaterial::from_color(Color::srgb(7.5, 0.0, 7.5)))),
      Transform::from_xyz(0.0, 0.0, 0.0),
  ));
  ```

## Primitive Shapes (verified v0.18.1)

- `Circle` in `bevy::prelude` (via bevy_math::primitives)
  - Field: `radius: f32`
  - Constructor: `Circle::new(radius: f32) -> Circle` (const fn)
  - Default: `Circle::default()` — radius 0.5
- `Rectangle` in `bevy::prelude` (via bevy_math::primitives)
  - Field: `half_size: Vec2`
  - Constructor: `Rectangle::new(width: f32, height: f32) -> Rectangle`
  - Stores half-sizes internally

## Testing Time<Fixed> / FixedUpdate Systems (verified v0.18.0 source)

Two verified approaches — both confirmed from Bevy source tests:

### Approach 1: Run system in Update schedule, inject delta via advance_by (recommended)
Register system in `Update` in tests; call `app.world_mut().resource_mut::<Time<Fixed>>().advance_by(duration)` before `app.update()`. This directly sets `delta()` without touching overstep or the FixedUpdate scheduler.

```rust
app.world_mut()
    .resource_mut::<Time<Fixed>>()
    .advance_by(Duration::from_secs_f32(1.0 / 64.0));
app.update();
```

### Approach 2: Force FixedUpdate to run via accumulate_overstep (Bevy-documented test helper)
`accumulate_overstep` is explicitly documented as "provided for use in tests". Call it with at least one full timestep's worth of time, then call `app.update()` — the scheduler will see enough overstep and run FixedUpdate.

```rust
// Set a known timestep first (optional but makes assertions deterministic)
app.world_mut()
    .resource_mut::<Time<Fixed>>()
    .set_timestep_hz(64.0);

// Accumulate enough overstep to trigger one FixedUpdate run
let timestep = app.world().resource::<Time<Fixed>>().timestep();
app.world_mut()
    .resource_mut::<Time<Fixed>>()
    .accumulate_overstep(timestep);
app.update(); // FixedUpdate will now run once
```

### Key facts
- `advance_by(&mut self, delta: Duration)` — sets delta and elapsed directly on the clock
- `accumulate_overstep(&mut self, delta: Duration)` — documented test helper; scheduler reads this to decide how many FixedUpdate ticks to run
- `expend(&mut self) -> bool` — PRIVATE in user code; called internally by scheduler per tick
- Default timestep: 64 Hz (15625 microseconds) — `Time::<Fixed>::DEFAULT_TIMESTEP`
- `delta()` on `Time<Fixed>` always equals `timestep()` when a tick fires (not variable)
- Both `Time<Fixed>` and `Time<Virtual>` must exist in the world; `MinimalPlugins` includes `TimePlugin` which inserts all three `Time<T>` variants
- Sources: `crates/bevy_time/src/fixed.rs` v0.18.0, `crates/bevy_app/src/main_schedule.rs` v0.18.0

## KeyboardInput (see keyboard_input.md for full details)

- `KeyboardInput` is a `Message` (NOT Event) — use `MessageReader<KeyboardInput>`, never `EventReader`
- Fields: `key_code: KeyCode`, `logical_key: Key`, `state: ButtonState`, `text: Option<SmolStr>`, `repeat: bool`, `window: Entity`
- `text` field IS present — include `text: None` when constructing
- System set is `InputSystems` (plural), NOT `InputSystem`
- Send in tests: `app.world_mut().write_message(KeyboardInput { ... })` — NOT `send_event()`
- `KeyboardInput`/`Key`/`ButtonState` NOT in prelude — import from `bevy::input::keyboard`

## Sources

- Feature flags: https://docs.rs/bevy/0.18.1/bevy/index.html#cargo-features
- Raw Cargo.toml: https://raw.githubusercontent.com/bevyengine/bevy/v0.18.0/Cargo.toml
- Fast compile guide: https://bevy.org/learn/quick-start/getting-started/setup/
- Message API: https://docs.rs/bevy_ecs/0.18.1/bevy_ecs/message/index.html
- States API: https://docs.rs/bevy/0.18.1/bevy/state/app/trait.AppExtStates.html
- PluginGroupBuilder: https://docs.rs/bevy/0.18.1/bevy/app/struct.PluginGroupBuilder.html
- WindowPlugin: https://docs.rs/bevy/0.18.1/bevy/window/struct.WindowPlugin.html
- bevy_egui: https://docs.rs/bevy_egui/latest/bevy_egui/index.html
