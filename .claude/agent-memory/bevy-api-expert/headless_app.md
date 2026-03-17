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

## WgpuSettings / RenderCreation (for reference)

If you DO need to disable rendering entirely (not just the window):
```rust
use bevy::render::{RenderPlugin, settings::{RenderCreation, WgpuSettings, Backends}};

RenderPlugin {
    render_creation: RenderCreation::Automatic(WgpuSettings {
        backends: None,  // no GPU backend = no rendering
        ..default()
    }),
    ..default()
}
```

But the official headless_renderer example does NOT do this — it leaves rendering on
and just removes the window/winit layer.

## MinimalPlugins for Pure Logic Tests

For tests with no rendering at all, MinimalPlugins is simpler:
```rust
App::new()
    .add_plugins(MinimalPlugins)  // includes ScheduleRunnerPlugin, TimePlugin, etc.
    // No WindowPlugin, no RenderPlugin
    .run();
```
