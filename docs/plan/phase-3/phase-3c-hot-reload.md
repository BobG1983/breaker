# Phase 3c: RON Hot-Reload

**Goal**: Change a RON file, see the update immediately in-game without rebuilding.

---

## Pipeline

```
RON file on disk
  → (Bevy file_watcher) → *Defaults asset updated
    → (propagate_defaults) → *Config resource updated
      → (propagate_config) → entity components updated
```

All systems dev-only (`#[cfg(feature = "dev")]`), centralized in `debug/hot_reload/`.

### Layer 1: File Watching → Defaults

- Enable Bevy's `file_watcher` feature in dev builds only
- *Defaults assets auto-update when RON files change — Bevy handles this natively
- Content assets (NodeLayout, CellTypeDefinition) also auto-update

### Layer 2: Defaults → Config

- `propagate_defaults` systems (one per config domain): detect `AssetEvent<T>::Modified`, re-seed the *Config resource
- Covers: BreakerConfig, BoltConfig, CellConfig, PlayfieldConfig, InputConfig

### Layer 3: Config → Components

- `propagate_config` systems (one per domain): detect `Res<T>::is_changed()`, update entity components
- Same logic as `init_*_params` but gated on change detection, runs every frame
- Covers: breaker components, bolt components, cell components

### Content Hot-Reload

- **Node layouts**: despawn current cells, re-spawn from updated layout immediately
- **Cell type definitions**: update registry, re-apply to existing cells

---

## CLI Level Spawning

- `--level <name>` on the game binary (dev builds only): skip menus, spawn directly into a specific level
- Separate from the scenario runner — this is for manual play-testing

---

## Checklist

- [ ] Enable Bevy `file_watcher` feature in dev builds
- [ ] Defaults → Config propagation systems (per domain)
- [ ] Config → Component propagation systems (per domain)
- [ ] Node layout hot-reload (despawn + re-spawn cells on change)
- [ ] Cell type definition hot-reload
- [ ] `--level` flag on game binary
