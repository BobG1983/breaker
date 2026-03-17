# Phase 3b: Debug Domain Restructure

**Goal**: Split the debug domain into sub-domains by concern, preparing homes for hot-reload and other dev tooling.

---

## Target Layout

```
breaker-game/src/debug/
├── mod.rs
├── plugin.rs          # DebugPlugin adds child plugins
├── resources.rs       # DebugOverlays (shared toggle state)
├── overlays/          # Gizmo drawing: hitboxes, velocity vectors
│   ├── plugin.rs      # OverlaysPlugin
│   └── systems/
├── telemetry/         # EguiPrimaryContextPass UI panels: bolt, breaker, input, bump
│   ├── plugin.rs      # TelemetryPlugin
│   └── systems/
├── hot_reload/        # RON hot-reload systems (added in 3c)
│   ├── plugin.rs      # HotReloadPlugin
│   └── systems/
└── recording/         # Live input capture for scripted scenario playback (added in 3d)
    ├── plugin.rs      # RecordingPlugin
    └── systems/
```

## What Moves

- `debug/systems/draw_hitboxes.rs`, `draw_velocity_vectors.rs` → `debug/overlays/systems/`
- `debug/systems/debug_ui_system.rs`, `bolt_info_ui.rs`, `breaker_state_ui.rs`, `input_actions_ui.rs`, `track_bump_result.rs` → `debug/telemetry/systems/`
- `debug/resources.rs` stays at the parent level (shared across sub-domains)

## Rules

- All sub-domain plugins added by `DebugPlugin`, gated behind `#[cfg(feature = "dev")]`
- No behavior change — pure structural refactor
- Sub-domains may import parent's `resources.rs` (same domain, not a boundary violation)

---

## Checklist

- [x] Create `overlays/` sub-domain with OverlaysPlugin
- [x] Create `telemetry/` sub-domain with TelemetryPlugin
- [x] Create `hot_reload/` sub-domain with stub HotReloadPlugin
- [x] Move existing systems into appropriate sub-domains
- [x] DebugPlugin adds child plugins
- [x] All tests pass, `cargo dev` debug UI unchanged

> **Addition**: A `recording/` sub-domain was also created (captures live inputs to `.scripted.ron` for scenario playback). Added alongside 3d/3e.
