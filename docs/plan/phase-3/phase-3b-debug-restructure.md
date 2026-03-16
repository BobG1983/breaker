# Phase 3b: Debug Domain Restructure

**Goal**: Split the debug domain into sub-domains by concern, preparing homes for hot-reload and other dev tooling.

---

## Target Layout

```
game/src/debug/
├── mod.rs
├── plugin.rs          # DebugPlugin adds child plugins
├── resources.rs       # DebugOverlays (shared toggle state)
├── overlays/          # Gizmo drawing: hitboxes, velocity vectors
│   ├── plugin.rs      # OverlaysPlugin
│   └── systems/
├── telemetry/         # EguiPrimaryContextPass UI panels: bolt, breaker, input, bump
│   ├── plugin.rs      # TelemetryPlugin
│   └── systems/
└── hot_reload/        # Empty initially — systems added in 3c
    └── plugin.rs      # HotReloadPlugin (stub)
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

- [ ] Create `overlays/` sub-domain with OverlaysPlugin
- [ ] Create `telemetry/` sub-domain with TelemetryPlugin
- [ ] Create `hot_reload/` sub-domain with stub HotReloadPlugin
- [ ] Move existing systems into appropriate sub-domains
- [ ] DebugPlugin adds child plugins
- [ ] All tests pass, `cargo dev` debug UI unchanged
