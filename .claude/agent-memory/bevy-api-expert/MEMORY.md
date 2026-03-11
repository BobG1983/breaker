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

## Sources

- Feature flags: https://docs.rs/bevy/0.18.1/bevy/index.html#cargo-features
- Raw Cargo.toml: https://raw.githubusercontent.com/bevyengine/bevy/v0.18.0/Cargo.toml
- Fast compile guide: https://bevy.org/learn/quick-start/getting-started/setup/
- Message API: https://docs.rs/bevy_ecs/0.18.1/bevy_ecs/message/index.html
- States API: https://docs.rs/bevy/0.18.1/bevy/state/app/trait.AppExtStates.html
- PluginGroupBuilder: https://docs.rs/bevy/0.18.1/bevy/app/struct.PluginGroupBuilder.html
- WindowPlugin: https://docs.rs/bevy/0.18.1/bevy/window/struct.WindowPlugin.html
- bevy_egui: https://docs.rs/bevy_egui/latest/bevy_egui/index.html
