# Headless Test Infrastructure

## Summary

Replace `MinimalPlugins` with headless `DefaultPlugins` everywhere we need a headless Bevy app — both `TestAppBuilder` and the game's own `headless_app()` integration test helper.

## Problem

Two places use `MinimalPlugins` and manually add plugins:

1. **`TestAppBuilder`** (`shared/test_utils/builder.rs`) — uses `MinimalPlugins`, forcing every test to manually register resources/systems. Tests don't match real runtime conditions.

2. **`headless_app()` in `app.rs`** (line 143) — the integration test helper uses `MinimalPlugins` with manually added `StatesPlugin`, `AssetPlugin`, `InputPlugin`. This is fragile and incomplete compared to what the game actually runs.

Both share the same root issue: `MinimalPlugins` is too bare, leading to fragile test setups that diverge from production.

## Solution

Use `DefaultPlugins` with rendering disabled:

```rust
use bevy::render::{
    settings::{RenderCreation, WgpuSettings},
    RenderPlugin,
};

DefaultPlugins.set(RenderPlugin {
    synchronous_pipeline_compilation: true,
    render_creation: RenderCreation::Automatic(WgpuSettings {
        backends: None,
        ..default()
    }),
    ..default()
})
```

This loads everything the game normally has (assets, time, transforms, schedules, etc.) but skips the GPU/rendering pipeline entirely.

## Migration

### TestAppBuilder
1. Update `TestAppBuilder::new()` to use headless `DefaultPlugins` instead of `MinimalPlugins`
2. Remove `with_*` methods that only exist to manually register things `DefaultPlugins` provides (keep domain-specific ones like registries)
3. Run full test suite and fix any tests that relied on the minimal environment
4. Consider whether `with_state_hierarchy()` is still needed or if states come from a game plugin

### headless_app() in app.rs
1. Replace the manual `MinimalPlugins` + `StatesPlugin` + `AssetPlugin` + `InputPlugin` assembly with headless `DefaultPlugins`
2. Keep the `Game::default().build().disable::<DebugPlugin>()` pattern
3. The `headless_app()` should be nearly identical to `build_app()` but with rendering disabled instead of the window/log setup

### Future: CLI headless mode
Consider using the same pattern for a `--headless` CLI flag on the actual game binary (useful for CI, scenario runner, automated testing).

## Source

Pattern from Tainted Coders Bevy guide.
