---
name: Headless App Patterns
description: Verified API for building headless Bevy 0.18 apps — no window, no render, ScheduleRunnerPlugin, AppExit, LogPlugin
type: reference
---

# Headless App in Bevy 0.18.1 (verified from source + examples)

## The Canonical Headless Pattern (from headless_renderer.rs example)

```rust
use bevy::prelude::*;
use bevy::app::{ScheduleRunnerPlugin, RunMode, AppExit};
use bevy::winit::WinitPlugin;
use std::time::Duration;

App::new()
    .add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: None,
                exit_condition: ExitCondition::DontExit,
                ..default()
            })
            .disable::<WinitPlugin>()
    )
    .add_plugins(ScheduleRunnerPlugin::run_once()) // or run_loop(duration)
    .run();
```

Key points:
- `primary_window: None` — disables the primary window
- `exit_condition: ExitCondition::DontExit` — prevents auto-exit when no windows exist
- `.disable::<WinitPlugin>()` — REQUIRED; prevents panic on systems without a display server
- `ScheduleRunnerPlugin` replaces winit's event loop as the app driver
- Do NOT need to disable RenderPlugin — the official headless_renderer example leaves it enabled

## "2d" Feature INCLUDES WinitPlugin

The `"2d"` feature pulls in `default_platform` which includes `bevy_winit`. So even with
`default-features = false, features = ["2d"]`, `WinitPlugin` IS present and MUST be disabled
for headless operation.

## ExitCondition Variants (bevy::window::ExitCondition)

- `OnPrimaryClosed` — exit when primary window closed
- `OnAllClosed` — exit when all windows closed
- `DontExit` — never auto-exit; YOU must send AppExit manually

## ScheduleRunnerPlugin (bevy::app::ScheduleRunnerPlugin)

```rust
// Run exactly once (for scenario/test tools):
ScheduleRunnerPlugin::run_once()

// Run in a loop at fixed rate:
ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1.0 / 60.0))
```

RunMode enum (bevy::app::RunMode):
- `RunMode::Once` — no fields
- `RunMode::Loop { wait: Option<Duration> }` — wait is the inter-frame delay

ScheduleRunnerPlugin is included in MinimalPlugins but NOT DefaultPlugins.
DefaultPlugins uses winit's event loop instead.

## AppExit (bevy::app::AppExit)

```rust
#[derive(Message, Debug, Clone, Default, PartialEq, Eq)]
pub enum AppExit {
    #[default]
    Success,
    Error(NonZero<u8>),  // NOT NonZeroU8 — uses generic NonZero<u8>
}
```

Helper methods: `AppExit::error()` (code 1), `AppExit::from_code(u8)`, `.is_success()`, `.is_error()`

AppExit IS a Message (NOT an Event). Send it via MessageWriter<AppExit>:

```rust
fn stop_system(mut exits: MessageWriter<AppExit>) {
    exits.write(AppExit::Success);
}
```

Or from world directly: `app.world_mut().write_message(AppExit::Success)`

## LogPlugin::custom_layer (bevy::log::LogPlugin)

Full struct:
```rust
pub struct LogPlugin {
    pub filter: String,
    pub level: Level,
    pub custom_layer: fn(app: &mut App) -> Option<BoxedLayer>,
    pub fmt_layer: fn(app: &mut App) -> Option<BoxedFmtLayer>,
}
```

Type aliases (bevy_log crate):
```rust
pub type BoxedLayer = Box<dyn Layer<Registry> + Send + Sync + 'static>;
pub type BoxedFmtLayer = Box<dyn Layer<PreFmtSubscriber> + Send + Sync + 'static>;
```

`Registry` here is `tracing_subscriber::Registry`. `custom_layer` is a FUNCTION POINTER
(not a closure field) — you must pass a named `fn` item, not a closure.

Usage:
```rust
LogPlugin {
    filter: "warn".to_string(),
    level: Level::WARN,
    custom_layer: |_app| Some(Box::new(my_layer)),
    ..default()
}
```

Note: LogPlugin can only be registered ONCE globally per process. If running multiple
apps in tests, disable it on all but the first: `.disable::<LogPlugin>()`.

## WgpuSettings / RenderCreation — No-GPU Headless (verified from Bevy 0.18 source)

To disable GPU initialization entirely (for CI servers with no GPU):

```rust
use bevy::render::{RenderPlugin, settings::{RenderCreation, WgpuSettings}};

RenderPlugin {
    render_creation: RenderCreation::Automatic(WgpuSettings {
        backends: None,  // skips initialize_renderer() entirely — no GPU adapter request
        ..default()
    }),
    ..default()
}
```

**Verified from `crates/bevy_render/src/lib.rs` source (v0.18.0):**
- `RenderPlugin::build()` has: `if let Some(backends) = render_creation.backends { ... }`
- When `backends` is `None`, the ENTIRE block is skipped: no `FutureRenderResources`, no `initialize_renderer()`, no `initialize_render_app()`
- `RenderPlugin::finish()` checks `if let Some(future) = app.world_mut().remove_resource::<FutureRenderResources>()` — returns `None`, so no GPU resources inserted
- `PipelinedRenderingPlugin::build()` checks `if app.get_sub_app(RenderApp).is_none() { return; }` — gracefully exits
- Result: NO "Unable to find a GPU" panic, no GPU access attempted at all

**WgpuSettings::default() has backends = Some(Backends::all())** — this is why not configuring RenderPlugin causes the panic.

The official headless_renderer example does NOT use `backends: None` because it DOES need a GPU (renders to texture). That example is about windowless rendering, not no-GPU rendering.

**The "Unable to find a GPU" panic** originates in `crates/bevy_render/src/renderer/mod.rs`:
```rust
let adapter = selected_adapter.expect(GPU_NOT_FOUND_ERROR_MESSAGE);
```
This is inside `initialize_renderer()`, which is only called when `backends` is `Some`.

## Complete No-GPU, No-Display Pattern (CI server with no GPU, no display server)

This is the correct pattern when you need AssetPlugin + StatesPlugin + TimePlugin + LogPlugin
but have NO GPU and NO display server (e.g., a CI runner):

```rust
use bevy::{
    prelude::*,
    render::{RenderPlugin, settings::{RenderCreation, WgpuSettings}},
    winit::WinitPlugin,
    window::ExitCondition,
};
use bevy::app::ScheduleRunnerPlugin;

let mut defaults = DefaultPlugins
    .set(WindowPlugin {
        primary_window: None,
        exit_condition: ExitCondition::DontExit,
        ..default()
    })
    .set(RenderPlugin {
        render_creation: RenderCreation::Automatic(WgpuSettings {
            backends: None,  // skip GPU init entirely
            ..default()
        }),
        ..default()
    })
    .disable::<WinitPlugin>();

app.add_plugins(defaults)
   .add_plugins(ScheduleRunnerPlugin::run_loop(...));
```

**Why two changes are needed:**
1. `.disable::<WinitPlugin>()` — prevents winit from panicking without a display server
2. `.set(RenderPlugin { backends: None })` — prevents wgpu from panicking without a GPU
   (disabling WinitPlugin alone does NOT stop RenderPlugin from trying to find a GPU)

**What you keep** (all still work with backends=None):
- `AssetPlugin` — file I/O only, no GPU required
- `StatesPlugin` — pure ECS, no GPU required
- `TimePlugin` — tick-based, no GPU required
- `LogPlugin` — tracing only, no GPU required
- `TransformPlugin`, `InputPlugin`, etc. — all CPU-side

**What breaks** (don't use these without a GPU):
- Any sprite/mesh rendering, materials, cameras that actually render
- `bevy_egui` rendering (egui logic still works, render does not)

## MinimalPlugins for Pure Logic Tests

For tests with no rendering at all, MinimalPlugins is simpler:
```rust
App::new()
    .add_plugins(MinimalPlugins)  // includes ScheduleRunnerPlugin, TimePlugin, etc.
    // No WindowPlugin, no RenderPlugin
    .run();
```

## Headless Warnings with backends: None (Bevy 0.18, "2d" feature)

When using `backends: None` (no GPU), three spurious messages appear. All are **harmless** — they do not indicate broken behavior.

### Warning 1: extract_resource for ClearColor

```
bevy_render::extract_resource: Render app did not exist when trying to add extract_resource for ClearColor
```

**Source:** `ExtractResourcePlugin::<ClearColor>::build()` in `bevy_render/src/extract_resource.rs`
**Level:** `error!` wrapped in `once!()` — fires exactly once, not repeated
**Cause:** `ClearColor` uses `ExtractResource`. With `backends: None`, `initialize_render_app()` is skipped,
so `RenderApp` sub-app never exists. `ExtractResourcePlugin` finds no `RenderApp` and emits this.
**Safe to ignore:** Yes. ClearColor extraction is a no-op in headless — nothing renders.
**Silence via LogPlugin filter:** Add `bevy_render::extract_resource=off` to the filter string.

### Warning 2: GizmoRenderPlugin RenderApp not detected

```
bevy_gizmos_render: bevy_render feature is enabled but RenderApp was not detected. Are you sure you loaded GizmoPlugin after RenderPlugin?
```

**Source:** `GizmoRenderPlugin::build()` in `bevy_gizmos_render/src/lib.rs`
**Level:** `warn!`
**Cause:** The `"2d"` feature enables `bevy_gizmos_render` (via `2d_bevy_render` → `bevy_gizmos_render`).
`GizmoRenderPlugin` checks `app.get_sub_app_mut(RenderApp)` and finds nothing (because `backends: None`
skips `initialize_render_app()`), so it emits this warning.
**Safe to ignore:** Yes. Gizmo rendering is entirely irrelevant in headless.
**Options:**
1. Silence via LogPlugin filter: `bevy_gizmos_render=off` (recommended — lowest friction)
2. Disable BOTH plugins separately — they are independent entries in `DefaultPlugins`:
   ```rust
   .disable::<bevy::gizmos::GizmoPlugin>()
   .disable::<bevy_gizmos_render::GizmoRenderPlugin>()
   ```
   **IMPORTANT (verified from `crates/bevy_internal/src/default_plugins.rs`):**
   `GizmoPlugin` (#[cfg(feature = "bevy_gizmos")]) and `GizmoRenderPlugin`
   (#[cfg(feature = "bevy_gizmos_render")]) are SEPARATE entries in DefaultPlugins.
   Disabling `GizmoPlugin` alone does NOT remove `GizmoRenderPlugin` — the warning persists.
   Both must be disabled if you want the structural fix.

### Warning 3: CompressedImageFormatSupport not found

```
bevy_render::texture: CompressedImageFormatSupport resource not found
```

**Source:** `TexturePlugin::finish()` in `bevy_render/src/texture/mod.rs`
**Level:** `warn!`
**Cause:** `TexturePlugin` is added by `RenderPlugin`. In its `finish()` phase, it checks for
`CompressedImageFormatSupport` (normally inserted during GPU init). With `backends: None`, GPU
init is skipped, so the resource is missing. `TexturePlugin` falls back to
`CompressedImageFormats::NONE` — a safe default for headless.
**Safe to ignore:** Yes. Image format detection is only relevant when actually decoding GPU-compressed textures.
**Silence via LogPlugin filter:** Add `bevy_render::texture=off` to the filter string.

### Recommended: Silence all three via LogPlugin filter

```rust
LogPlugin {
    filter: "warn,bevy_egui=error,breaker=info,\
             bevy_render::extract_resource=off,\
             bevy_gizmos_render=off,\
             bevy_render::texture=off".to_owned(),
    ..default()
}
```

This is the **lowest-risk approach** — no structural changes, no disabled plugins,
just log-level silencing of verified-harmless messages.

### Alternative: Disable both gizmo plugins to eliminate warning 2 structurally

```rust
defaults = defaults
    .set(RenderPlugin { render_creation: RenderCreation::Automatic(WgpuSettings { backends: None, ..default() }), ..default() })
    .disable::<WinitPlugin>()
    .disable::<bevy::gizmos::GizmoPlugin>()
    .disable::<bevy_gizmos_render::GizmoRenderPlugin>();
```

BOTH must be disabled. `GizmoPlugin` (#[cfg(bevy_gizmos)]) and `GizmoRenderPlugin`
(#[cfg(bevy_gizmos_render)]) are independent entries in DefaultPlugins — disabling one
does NOT remove the other. Disabling only GizmoPlugin leaves GizmoRenderPlugin running,
which is the source of the warning. Warnings 1 and 3 still require filter silencing.

### Can you .disable::<RenderPlugin>() entirely?

**Not recommended** for this project. With the `"2d"` feature, `RenderPlugin` is the parent of:
- `TexturePlugin` (image asset loading hooks)
- `CameraPlugin` (camera component registration)
- `MeshRenderAssetPlugin`
- and more

**However**, `MeshPlugin` is listed **separately** in `DefaultPlugins` under `#[cfg(feature = "bevy_mesh")]`
and does NOT depend on `RenderPlugin`. So `Mesh` as an asset type survives `RenderPlugin` being disabled.

**Color** types (`Color`, `LinearRgba`, etc.) are in `bevy_color` — no render dependency.

**Sprite** as an ECS component (`bevy_sprite`) has CPU-side bounds systems that survive without a GPU.

**The risk** is that disabling `RenderPlugin` entirely may break plugins in the game crate that add
render-specific systems (e.g., `Camera2d`, material handles, extract systems). Use `backends: None`
+ log filter silencing instead — it is the safer, verified pattern.

## Plugin-by-plugin RenderApp dependency analysis (verified from 0.18.1 source)

Which plugins in DefaultPlugins (with `"2d"` feature) use `get_sub_app_mut(RenderApp)` vs panic if absent:

| Plugin | RenderApp behavior | Safe without GPU? |
|---|---|---|
| `TextPlugin` | No RenderApp access at all | YES — pure CPU (cosmic_text, font atlas, asset loading) |
| `SpritePlugin` (bevy_sprite) | No RenderApp access | YES — CPU-side bounds/AABB systems only |
| `SpriteRenderPlugin` (bevy_sprite_render) | `if let Some(render_app) = ...` — graceful skip | YES — skips render setup if no RenderApp |
| `CorePipelinePlugin` | `let Some(render_app) = ... else { return; }` — graceful skip | YES — skips FullscreenShader init |
| `CameraPlugin` (bevy_camera) | No RenderApp access at all | YES — registers components and systems |
| `ImagePlugin` (bevy_image) | No RenderApp access | YES — asset registration only |
| `RenderPlugin` itself | `backends: None` → skips ALL GPU init including `initialize_render_app()` | YES with `backends: None` |
| `WinitPlugin` | Panics without display server | MUST `.disable::<WinitPlugin>()` |

## TextPlugin — verified: NO RenderApp dependency (0.18.1)

`TextPlugin::build()` (from `bevy_text-0.18.1/src/lib.rs`) does exactly:
1. `app.init_asset::<Font>()` — asset registration
2. `app.init_asset_loader::<FontLoader>()` — asset loader
3. `app.init_resource::<FontAtlasSet>()` — CPU resource
4. `app.init_resource::<TextPipeline>()` — CPU resource
5. `app.init_resource::<CosmicFontSystem>()` — CPU resource (wraps cosmic_text::FontSystem)
6. `app.init_resource::<SwashCache>()` — CPU resource (wraps cosmic_text::SwashCache)
7. `app.init_resource::<TextIterScratch>()` — CPU resource
8. Adds `free_unused_font_atlases_system` to PostUpdate
9. Adds `trim_cosmic_cache` to Last

**Zero RenderApp access.** TextPlugin is completely safe to load without RenderPlugin/GPU.

## CosmicFontSystem and SwashCache — verified: NO GPU dependency

Both are pure CPU resources:
- `CosmicFontSystem` wraps `cosmic_text::FontSystem` — font database + locale, initialized from system locale/empty DB, no GPU
- `SwashCache` wraps `cosmic_text::SwashCache::new()` — CPU glyph rasterizer, no GPU

Neither struct has any wgpu/GPU-related field or initialization.

## bevy::hierarchy::HierarchyPlugin — does NOT exist in 0.18.1

**Verified**: There is no `HierarchyPlugin` type anywhere in `bevy_ecs-0.18.1` or any other 0.18.1 crate.

The hierarchy module (`bevy_ecs::hierarchy`) is a plain module containing `ChildOf`, `Children`, `ChildSpawner`,
`ChildSpawnerCommands` — no plugin struct. Hierarchy is registered as part of `bevy_ecs` itself (built in),
not through a separate plugin.

Path for hierarchy types: `bevy::ecs::hierarchy::ChildOf` or via `bevy::prelude::*`.
There is no `bevy::hierarchy` module path — `bevy_ecs` is re-exported as `bevy::ecs`, not `bevy::hierarchy`.

## bevy::transform::TransformPlugin — exists, correct path

**Verified** from `bevy_transform-0.18.1/src/plugins.rs`:
```rust
pub struct TransformPlugin;
impl Plugin for TransformPlugin { ... }
```

Module path: `bevy::transform::TransformPlugin`
(because `bevy_internal` re-exports `bevy_transform as transform`, no feature gate — always present)

Also in prelude: `bevy_transform::prelude::TransformPlugin` re-exports it.

`TransformPlugin::build()` only adds CPU-side systems — no RenderApp access.

## Which DefaultPlugins plugins PANIC if RenderPlugin is completely absent (not loaded at all)

With the `"2d"` feature, if you `.disable::<RenderPlugin>()` entirely (i.e., no RenderPlugin loaded,
not even with `backends: None`):

**Will panic or error:**
- `WindowRenderPlugin`, `ViewPlugin`, `MeshRenderAssetPlugin`, `TexturePlugin`, `SyncWorldPlugin`, etc.
  — these are added by `RenderPlugin::build()` UNCONDITIONALLY even with `backends: None`.
  If RenderPlugin itself is not loaded, these sub-plugins don't run. But other plugins that were compiled
  expecting those resources/types may panic at runtime.

**More precisely**: With the `"2d"` feature, `SpriteRenderPlugin` uses
`bevy_render::sync_world::SyncToRenderWorld` in `app.register_required_components`. This type comes
from `bevy_render`. The plugin compiles against it regardless. If `RenderPlugin` is not loaded,
`SyncToRenderWorld` as a required component registration may fail at runtime since the render world
sync infrastructure was never set up.

**The safest verified approach remains**: Keep `RenderPlugin` present with `backends: None`.
This avoids all runtime panics while skipping GPU initialization entirely.

**Plugins that are definitely safe even with `.disable::<RenderPlugin>()`** (pure CPU, no render dep):
- `TextPlugin` — verified, no render dep
- `TransformPlugin` — verified, no render dep
- `InputPlugin` — pure CPU
- `TimePlugin` — pure CPU
- `AssetPlugin` — file I/O
- `StatesPlugin` — pure ECS
- `LogPlugin` — tracing only
- `CameraPlugin` (bevy_camera) — verified, no RenderApp access in its own build()
