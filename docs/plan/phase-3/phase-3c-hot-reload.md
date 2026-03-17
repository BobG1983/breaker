# Phase 3c: RON Hot-Reload

**Goal**: Change a RON file, see the update immediately in-game without rebuilding. Critical for tuning gameplay in Phase 4.

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
- Content assets (NodeLayout, CellTypeDefinition, ArchetypeDefinition) also auto-update

### Layer 2: Defaults → Config

- `propagate_defaults` systems (one per config domain): detect `AssetEvent<T>::Modified`, re-seed the *Config resource
- Covers: BoltConfig, BreakerConfig, CellConfig, PlayfieldConfig, InputConfig, TimerUiConfig, MainMenuConfig, ChipSelectConfig
- **Breaker special case**: after re-seeding config, re-apply archetype stat overrides via `apply_stat_overrides` helper

### Layer 3: Config → Components

- `propagate_config` systems: detect `Res<T>::is_changed()`, force-overwrite entity components
- Covers: bolt components (8), breaker components (~22 + archetype multipliers)
- **No cell config propagation**: cell config changes affect grid layout, so trigger full despawn+respawn instead

### Content Hot-Reload

- **Cell type definitions**: rebuild registry, update health/visuals/color on matching live cells via `CellTypeAlias` tracking component
- **Node layouts**: rebuild registry, despawn current cells directly (skip destruction pipeline), re-spawn from updated layout
- **Archetypes**: rebuild registry, reset BreakerConfig from defaults + re-apply overrides, re-stamp consequence components, rebuild ActiveBehaviors

---

## Upgrade-Aware Config Propagation

Rather than a generic modifier stack (premature — chips aren't implemented yet), reuse the existing init pipeline:

```
RON changes → re-seed Config from new Defaults
  → re-apply archetype stat overrides (apply_stat_overrides helper)
    → re-stamp entity components (force-overwrite)
      → re-stamp archetype consequence components (multipliers, lives)
```

When chips arrive in Phase 4, they add one more step at the end of this chain.

---

## Checklist

- [ ] Add `CellTypeAlias` component to track which cell type definition spawned each cell
- [ ] Enable Bevy `file_watcher` feature in dev builds
- [ ] Hot-reload module structure + system sets (`HotReloadSystems`)
- [ ] Extract `apply_stat_overrides` helper from `apply_archetype_config_overrides`
- [ ] Defaults → Config propagation systems (8 systems)
- [ ] Config → Component propagation systems (bolt + breaker)
- [ ] Cell type definition hot-reload
- [ ] Node layout hot-reload (despawn + re-spawn cells on change)
- [ ] Archetype hot-reload
- [ ] `--level` CLI flag for dev play-testing
