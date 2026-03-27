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
- **NOTE:** `breaker-game` does NOT directly depend on `bevy_common_assets` as of the SeedableRegistry feature. `rantzsoft_defaults` uses `RonAssetPlugin` internally via its `RonAssetLoader`. Do not add `bevy_common_assets` directly to `breaker-game` Cargo.toml.

## bevy_asset_loader 0.25

- **NOTE:** `breaker-game` NO LONGER uses `bevy_asset_loader` directly as of the SeedableRegistry feature. `DefaultsCollection` (which used `AssetCollection`) is DELETED. The `rantzsoft_defaults` plugin now handles all asset loading. Do NOT add `bevy_asset_loader` back to `breaker-game/Cargo.toml`.
- If needed in `rantzsoft_*` crates, see docs.rs/bevy_asset_loader/0.25.0 for API.

## iyes_progress 0.16 (verified — still used)

- `breaker-game/Cargo.toml` still depends on `iyes_progress = "0.16"` directly for `ProgressPlugin`.
- Must add `iyes_progress = "0.16"` directly (not re-exported by bevy_asset_loader)
- `Progress` struct: `pub done: u32, pub total: u32` — implements `Into<f32>` (0.0–1.0 ratio)
- `HiddenProgress(pub Progress)` — blocks transition but invisible to `get_global_progress()`
- `ProgressTracker<S>: Resource` — `get_global_progress() -> Progress`, `is_ready() -> bool`
- Systems returning `Progress`/`HiddenProgress` use `.track_progress::<S>()` or `.track_progress_and_stop::<S>()`
- `ProgressPlugin::<S>::new().with_state_transition(from, to)` drives the state change
- `ProgressPlugin` MUST be added BEFORE the `RantzDefaultsPlugin` in the app builder
- Clippy warning: `u32 as f32` cast triggers `cast_precision_loss` — use `Into::<f32>::into(progress)` instead
- Sources: docs.rs/iyes_progress/0.16.0, github.com/IyesGames/iyes_progress v0.16.0 full.rs example
