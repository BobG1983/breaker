---
name: Monitor and winit 0.30 API
description: Monitor query API in Bevy 0.18.1/winit 0.30.13 — what works, what doesn't, and the correct pattern for cross-process window tiling
type: reference
---

## winit 0.30.13 Monitor Query Facts

- `EventLoop` (pre-run_app) has NO `available_monitors()` method — confirmed from docs.rs
- `ActiveEventLoop` HAS `available_monitors() -> impl Iterator<Item=MonitorHandle>` and `primary_monitor() -> Option<MonitorHandle>` — but requires being inside a `run_app()` callback; no public constructor
- `MonitorHandle` methods: `size() -> PhysicalSize<u32>`, `position() -> PhysicalPosition<i32>`, `name() -> Option<String>`, `scale_factor() -> f64`, `refresh_rate_millihertz() -> Option<u32>`
- There is NO way to query monitor dimensions from a CLI binary that never calls `run_app()`

## Bevy 0.18.1 Monitor Types (bevy::window)

- `Monitor` is a **Component** (not a Resource): `physical_width: u32`, `physical_height: u32`, `physical_position: IVec2`, `scale_factor: f64`, `name: Option<String>`
- `PrimaryMonitor` is a **marker Component** on the primary monitor entity
- `MonitorSelection` enum — used in `Window::position` to reference a monitor entity
- Populated at runtime by bevy_winit AFTER windowing initializes — not available in Startup before winit runs

## CoreGraphics FFI (macOS only)

- `core-graphics 0.23.2` is a transitive dep (via winit → bevy_winit)
- `CGMainDisplayID() -> CGDirectDisplayID` — unsafe extern "C"
- `CGDisplayPixelsWide(display: CGDirectDisplayID) -> size_t` — unsafe extern "C"
- `CGDisplayPixelsHigh(display: CGDirectDisplayID) -> size_t` — unsafe extern "C"
- BLOCKED by `unsafe_code = "deny"` in workspace lints — cannot use this approach

## Correct Pattern for Parent-CLI + Child-Bevy Window Tiling

The parent CLI cannot query the screen. The correct pattern:

1. Pass `--tile-index N --tile-count M` as CLI args to each child subprocess
2. Each child, inside a Bevy startup system that runs AFTER winit initializes, queries `PrimaryMonitor` entity → `Monitor` component to get screen dimensions
3. Child computes its tile rect and applies it via `WindowPosition::At(...)` and `WindowResolution`

This uses no unsafe code, works cross-platform, and each child independently knows its own screen.
