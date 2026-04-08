---
name: Bevy 0.18.1 UI and Rendering
description: GlobalZIndex full-screen overlays, Val variants, UiScale, TextFont, Screenshot API
type: reference
---

# UI Z-Ordering — GlobalZIndex and ZIndex (Bevy 0.18.1)

Verified from `docs.rs/bevy/0.18.0/bevy/prelude/struct.GlobalZIndex.html` and `bevy_dev_tools` FPS overlay source.

## `GlobalZIndex(i32)` — cross-hierarchy overlay ordering

```rust
use bevy::prelude::GlobalZIndex;

// Render above ALL other UI nodes globally:
GlobalZIndex(i32::MAX - 1)
// FPS overlay uses i32::MAX - 32 "so you can render on top of it if you really need to"
```

- `GlobalZIndex` allows a Node to escape the implicit draw ordering of the UI layout tree
- Positive values render ON TOP of nodes without GlobalZIndex or lower values
- Negative values render BELOW nodes without GlobalZIndex or higher values
- For siblings with same GlobalZIndex: the one with greater local `ZIndex` wins
- `ZIndex` alone only affects ordering among siblings — use `GlobalZIndex` for cross-hierarchy overlays

## Full-screen overlay spawn pattern (confirmed working)

```rust
commands.spawn((
    Node {
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        position_type: PositionType::Absolute,
        ..default()
    },
    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
    GlobalZIndex(i32::MAX - 1),  // covers all other UI including HUD
));
```

---

# UI Scaling — Val variants, UiScale, TextFont (Bevy 0.18.1)

Verified from: `docs.rs/bevy/0.18.1`, `crates/bevy_ui/src/layout/convert.rs`,
`crates/bevy_ui/src/update.rs`, `crates/bevy_ui/src/widget/text.rs`,
`examples/ui/ui_scaling.rs` all at v0.18.1.

## Val variants

```rust
pub enum Val {
    Auto,
    Px(f32),        // scaled by: camera.target_scaling_factor() * ui_scale.0
    Percent(f32),   // % of parent node dimension — NOT scaled by UiScale
    Vw(f32),        // % of physical_size.x — NOT scaled by UiScale
    Vh(f32),        // % of physical_size.y — NOT scaled by UiScale
    VMin(f32),      // % of physical_size.min_element() — NOT scaled by UiScale
    VMax(f32),      // % of physical_size.max_element() — NOT scaled by UiScale
}
```

Conversion is in `into_length_percentage_auto` in `convert.rs`:
- `Val::Px(v)` → `scale_factor * v`  (scale_factor = camera factor × ui_scale.0)
- `Val::Vw(v)` → `physical_size.x * v / 100.`  (raw physical pixels, no UiScale)

**Gotcha**: Do NOT mix Px and Vw/Vh in the same layout when UiScale != 1.0 — the two unit
systems are on different scales.

## UiScale resource

```rust
pub struct UiScale(pub f32);  // Default: 1.0
// Applied: layout_scale_factor = camera.target_scaling_factor() * ui_scale.0
```

**What UiScale scales**: `Val::Px` sizing AND `TextFont::font_size`.

**What UiScale does NOT scale**: `Val::Vw/Vh/VMin/VMax`, `Val::Percent`.

## TextFont — font_size field

```rust
pub struct TextFont {
    pub font: Handle<Font>,
    pub font_size: f32,  // physical pixels; multiplied by scale_factor (which includes UiScale)
}
```

No built-in responsive font size in Bevy 0.18.1. Setting UiScale scales fonts globally.

## Recommended pattern: UiScale driven by window dimensions

```rust
fn sync_ui_scale(
    mut ui_scale: ResMut<UiScale>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    if let Ok(window) = windows.get_single() {
        // Design resolution: 1920×1080. Use min to letterbox.
        let scale = (window.width() / 1920.0).min(window.height() / 1080.0);
        ui_scale.0 = scale;
    }
}
// Run in Update. All Val::Px and font_size designed for 1920×1080 scale automatically.
```

**UiScale is global** — cannot be per-node.

Full research report: `docs/todos/detail/scenario-runner-verbose-violation-log/research/bevy-ui-scaling.md`

---

# Screenshot API — `bevy::render::view::screenshot` (Bevy 0.18.1)

Verified from `crates/bevy_render/src/view/window/screenshot.rs` at tag `v0.18.1` and
`examples/window/screenshot.rs`.

## Pattern: spawn an entity with `Screenshot` component + observer

```rust
use bevy::render::view::screenshot::{save_to_disk, Capturing, Screenshot};

fn take_screenshot(mut commands: Commands) {
    commands
        .spawn(Screenshot::primary_window())
        .observe(save_to_disk("screenshot.png"));
}
```

## Key types

- `Screenshot(pub RenderTarget)` — Component. Constructors: `::primary_window()`,
  `::window(entity)`, `::image(handle)`, `::texture_view(handle)`
- `ScreenshotCaptured { entity: Entity, image: Image }` — EntityEvent, triggered on the
  screenshot entity when the GPU readback completes
- `Capturing` — marker Component, present while capture is in flight
- `Captured` — marker Component, added when image is ready (entity despawned next First tick)
- `save_to_disk(path: impl AsRef<Path>) -> impl FnMut(On<ScreenshotCaptured>)` — free fn

## Critical: observer trigger type is `On<>`, NOT `Trigger<>`

In 0.18.1: `impl FnMut(On<ScreenshotCaptured>)`
In 0.15.x: `impl FnMut(Trigger<ScreenshotCaptured>)` — WRONG for 0.18.1

## Async — spans at least 2 frames

Frame N: spawn entity. Frame N (render): GPU captures. Async task maps buffer and sends over
mpsc. Frame N+1+ (Update): `trigger_screenshots` polls channel, fires observer. Not same-frame.

## ScreenshotPlugin is auto-included with DefaultPlugins

Path: `DefaultPlugins -> RenderPlugin -> ViewPlugin -> WindowRenderPlugin -> ScreenshotPlugin`.
No manual registration needed. Not present in `MinimalPlugins` (headless) — must guard.

## save_to_disk format

Format inferred from file extension. PNG recommended. Image saved as RGB8 (alpha stripped).
Directory must exist before the observer fires (create_dir_all before spawning the entity).

## One screenshot per render target per frame

`extract_screenshots` skips duplicates and despawns the duplicate entity. Only one
`Screenshot::primary_window()` per frame will be captured.

Full research report: `.claude/research/scenario-runner-bevy-screenshot-api.md`
