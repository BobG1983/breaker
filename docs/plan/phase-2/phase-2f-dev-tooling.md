# Phase 2f: Dev Tooling

**Goal**: Fast iteration without rebuilds. Hot-reload RON files, propagate changes to configs and entities live, and restructure the debug domain for growth.

---

## RON Hot-Reload Pipeline

Three-layer propagation chain, all dev-only (`#[cfg(feature = "dev")]`):

```
RON file on disk
  → (Bevy file_watcher) → *Defaults asset updated
    → (propagate_defaults) → *Config resource updated
      → (propagate_config) → entity components updated
```

### Layer 1: File Watching → Defaults

- Enable Bevy's `file_watcher` feature in dev builds only
- *Defaults assets (BreakerDefaults, BoltDefaults, etc.) auto-update when their RON files change on disk — Bevy handles this natively
- Content assets (NodeLayout, CellTypeDefinition) also auto-update

### Layer 2: Defaults → Config

- **`propagate_defaults` systems** (one per config domain): detect when a *Defaults asset has changed (`AssetEvent<T>::Modified`), re-seed the corresponding *Config resource
- Covers: BreakerConfig, BoltConfig, CellConfig, PlayfieldConfig, InputConfig
- Lives in `debug/hot_reload/`

### Layer 3: Config → Components

- **`propagate_config` systems** (one per domain): detect when a *Config resource has changed (`Res<T>::is_changed()`), update entity components to match
- Same logic as `init_*_params` but runs every frame (gated on change detection) instead of once on enter
- Covers: breaker components, bolt components, cell components
- Lives in `debug/hot_reload/`

### Content Hot-Reload

- **Node layouts**: when a NodeLayout asset changes mid-play, despawn current cells and re-spawn from the updated layout immediately — jarring but instant feedback for level design iteration
- **Cell type definitions**: when a CellTypeDefinition asset changes, update the cell type registry and re-apply to existing cells
- Lives in `debug/hot_reload/`

---

## CLI Test-Level Spawning

- **Command-line argument** (dev/debug mode only): skip menus and spawn directly into a specific test level layout
- Lives in `debug/hot_reload/` or as a standalone system in the debug domain

---

## Debug Domain Restructure

Split the debug domain into sub-domains by concern:

```
src/debug/
├── mod.rs
├── plugin.rs          # DebugPlugin adds child plugins
├── resources.rs       # DebugOverlays (shared toggle state)
├── overlays/          # Gizmo drawing: hitboxes, velocity vectors
│   ├── plugin.rs      # OverlaysPlugin
│   └── systems/
├── telemetry/         # EguiPrimaryContextPass UI panels: bolt, breaker, input, bump
│   ├── plugin.rs      # TelemetryPlugin
│   └── systems/
└── hot_reload/        # RON watching, config propagation, layout reload
    ├── plugin.rs      # HotReloadPlugin
    └── systems/
```

- All sub-domain plugins are added by `DebugPlugin`, gated behind `#[cfg(feature = "dev")]`
- Existing debug systems move into `overlays/` and `telemetry/` — no behavior change
- All hot-reload systems live in `hot_reload/` — centralized, not scattered across production domains

---

## Checklist

- [ ] Enable Bevy `file_watcher` feature in dev builds
- [ ] Defaults → Config propagation systems (per domain, in hot_reload/)
- [ ] Config → Component propagation systems (per domain, in hot_reload/)
- [ ] Node layout hot-reload (despawn + re-spawn cells on change)
- [ ] Cell type definition hot-reload
- [ ] CLI test-level spawning (debug/dev mode)
- [ ] Restructure debug domain into overlays/, telemetry/, hot_reload/ sub-domains
- [ ] Move existing debug systems into overlays/ and telemetry/
