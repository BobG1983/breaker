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
- `commands.spawn(Camera2d)` — not CameraBundle
- `Sprite::from_image(handle)` or `Sprite::from_atlas_image(handle, atlas)`
- Messages: `#[derive(Message)]`, `MessageWriter<T>`, `MessageReader<T>`, `app.add_message::<T>()`
- Events (observable/triggered only): `#[derive(Event)]` — NOT for game messages

## Sources

- Feature flags: https://docs.rs/bevy/0.18.1/bevy/index.html#cargo-features
- Raw Cargo.toml: https://raw.githubusercontent.com/bevyengine/bevy/v0.18.0/Cargo.toml
- Fast compile guide: https://bevy.org/learn/quick-start/getting-started/setup/
