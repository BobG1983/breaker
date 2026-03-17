---
name: Third-Party Crate Compatibility (Bevy 0.18)
description: Verified compatibility and usage patterns for bevy_egui, bevy_common_assets, bevy_asset_loader, iyes_progress
type: reference
---

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
