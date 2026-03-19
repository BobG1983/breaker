---
name: Full Tree Validation
description: Complete source tree validation against docs/architecture/ — findings, confirmed patterns, docs drift
type: project
---

## Last validated: 2026-03-19

### Summary
16 domains reviewed. Architecture is well-maintained. 6 documentation drift items found. 2 structural observations (not violations). No boundary violations in production code.

### Key Findings

**Documentation Drift (messages.md)**
5 messages exist in code but are not documented in `docs/architecture/messages.md`:
- `BreakerSpawned` (breaker/messages.rs) — sent by spawn_breaker, consumed by check_spawn_complete
- `BoltSpawned` (bolt/messages.rs) — sent by spawn_bolt, consumed by check_spawn_complete
- `CellsSpawned` (run/node/messages.rs) — sent by spawn_cells_from_layout, consumed by check_spawn_complete
- `WallsSpawned` (wall/messages.rs) — sent by spawn_walls, consumed by check_spawn_complete
- `SpawnNodeComplete` (run/node/messages.rs) — sent by check_spawn_complete, consumed by scenario runner

**Documentation Drift (ordering.md)**
- `BreakerSystems::Reset` exists in code (sets.rs, plugin.rs:43) but not in ordering.md's "Defined sets" table
- `BoltSystems::InitParams` exists in code (sets.rs, plugin.rs:38) but not in ordering.md's "Defined sets" table
- `NodeSystems` (5 variants: Spawn, TrackCompletion, TickTimer, ApplyTimePenalty, InitTimer) not in ordering.md's table
- The OnEnter chain in ordering.md is outdated — does not include BreakerSystems::Reset or BoltSystems::InitParams

**Cross-domain reads that are architecturally sound (not violations):**
- Physics reads bolt, breaker, cell, wall components (necessary for collision)
- Bolt reads breaker components (hover_bolt, prepare_bolt_velocity for MinAngleFromHorizontal)
- Behaviors reads/writes BreakerConfig (apply_archetype_config_overrides — init-time only, documented pattern)
- Behaviors reads UI StatusPanel component (spawn_lives_display — for HUD parenting)
- Run/node reads cells components (init_clear_remaining reads RequiredToClear, spawn_cells_from_layout creates cells)
- UI reads run/node NodeTimer resource (timer display)

### Confirmed Good Patterns
- All domain mod.rs files are routing-only (shared/ is not a domain, so its inline types are fine)
- All systems/mod.rs files are routing-only
- All messages use Bevy 0.18 #[derive(Message)] correctly
- All system sets are in sets.rs files
- All queries are in queries.rs files
- All filters are in filters.rs files
- Plugin.rs files are the only files wiring to App
- Debug domain cross-domain access is properly gated behind #[cfg(feature = "dev")]
- Cleanup markers (CleanupOnNodeExit, CleanupOnRunEnd) consistently applied to spawned entities
- Schedule placement correct: FixedUpdate for gameplay, Update for visuals, OnEnter for state transitions
- run_if(in_state(PlayingState::Active)) consistently applied to FixedUpdate gameplay systems
