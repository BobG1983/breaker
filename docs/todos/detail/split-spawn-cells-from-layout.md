# Split spawn_cells_from_layout Into Separate Concerns

## Summary

`spawn_cells_from_layout/system.rs` is 520 lines of production code doing four distinct jobs in one file. After the node sequencing refactor reshapes how layouts are generated and consumed, split this file by concern so each piece is testable and modifiable independently.

**Do this AFTER the node sequencing refactor** — that refactor will change what data flows into cell spawning (per-tier batching, volatile nodes, frame/block generation), so the shape of these concerns will shift. Splitting now would mean splitting twice.

## Current Concerns (as of today)

These are the responsibilities currently tangled in `system.rs`. Post-node-sequencing, the boundaries may shift — use this as a starting guide, not a rigid prescription.

### 1. Grid Geometry (`compute_grid_scale`, `grid_extent`, `ScaledGridDims`)

Pure math: given a grid size and playfield config, compute the uniform scale factor, cell dimensions, and step sizes. No ECS, no entities, no side effects.

- ~80 lines today
- Should be a standalone module (or even a method on a config type)
- Easily unit-testable with no App needed

### 2. Grid-to-World Position Mapping (`tile_position` logic within `.spawn_pass1()`)

Converts (col, row) grid coordinates into world-space Position2D values using the scaled grid dims and playfield boundaries. Pure coordinate math.

- Currently inlined in the spawning loop
- Should be an extractable function: `(col, row, ScaledGridDims, PlayfieldConfig) -> Vec2`
- The node sequencing refactor may change grid origins or introduce variable-size grids — this concern may grow

### 3. Lock Resolution

Resolves `LockMap` entries: which cells are locked, which cells are keys, cross-referencing grid positions. Produces a mapping of entity → lock relationships before spawning.

- Currently ~100 lines interleaved with spawning
- Pure data transform: `(NodeLayout, LockMap) -> HashMap<GridPos, LockInfo>`
- Already has its own test subdirectory (`tests/lock_resolution/`) — a sign it wants to be its own module

### 4. Entity Spawning & Wiring (`spawn_cells_from_layout`, `spawn_cells_from_grid`, `CellSpawnContext`, `GridCellContext`)

The actual ECS work: iterating the grid, calling the cell builder, wiring up components (RequiredToClear, behaviors, guardian configs), dispatching CellsSpawned message.

- The core system — this stays in `system.rs` (or gets renamed to `spawn.rs`)
- Should consume the outputs of concerns 1-3, not compute them inline
- Currently uses a `SystemParam` struct (`CellSpawnContext`) — that's good, keep it

## Expected Shape After Split

```
spawn_cells_from_layout/
  mod.rs              // pub(crate) mod grid; pub(crate) mod locks; pub(crate) mod spawn;
  grid.rs             // ScaledGridDims, compute_grid_scale, grid_extent, tile_to_world
  locks.rs            // lock resolution logic
  spawn.rs            // CellSpawnContext, spawn_cells_from_layout, spawn_cells_from_grid
  tests/
    mod.rs
    grid_tests.rs     // pure math tests (no App)
    lock_resolution/  // existing tests, re-parented
    position2d.rs     // existing
    behaviors.rs      // existing
    basic_spawning.rs // existing
```

## What the Node Sequencing Refactor Might Change

- Grid dimensions may become variable per-frame/block instead of per-node
- New `FrameDef`/`BlockDef` types may replace or wrap `NodeLayout`
- Portal cells may need special grid position logic
- Volatile nodes may need partial grid spawning

All of these would change the interface between concerns 1-3 and concern 4. That's why we wait.

## Acceptance Criteria

- [ ] No production file over 200 lines in the module
- [ ] Grid geometry is testable without an App
- [ ] Lock resolution is testable without an App
- [ ] All existing tests pass without modification (only import paths change)
- [ ] `spawn_cells_from_layout` system signature unchanged (callers unaffected)

## Test Inventory (2,114 lines across 6 files today)

| File | Lines | Concern |
|------|-------|---------|
| `tests/basic_spawning.rs` | 609 | Entity spawning |
| `tests/behaviors.rs` | 609 | Entity spawning + behaviors |
| `tests/position2d.rs` | ~300 | Grid-to-world mapping |
| `tests/lock_resolution/` | ~400 | Lock resolution |
| `tests/toughness.rs` | ~200 | Entity spawning + HP |

After the split, each test file should import from the specific concern module, not `super::*` reaching across all concerns.
