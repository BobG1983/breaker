# Grid-orthogonal adjacency

Assumes: #1 (the crate's `Destroyed<T>` is a generic message and does NOT carry entity-kind-specific fields), #2 (hazards live at `mutators/hazards/`).

Cross-cutting: introduces new cells-domain API (`CellGridPosition`, `orthogonal_neighbors`, game-side `CellDestroyed` companion message). Cascade and Diffusion migrate to it. Fracture (see `fracture.md` in this folder) also consumes the same API.

## What's broken

Cascade and Diffusion currently use radius-based adjacency:

```rust
const ADJACENCY_RADIUS_SQ: f32 = 70.0 * 70.0;
// ...
if pos.0.distance_squared(victim_pos) < ADJACENCY_RADIUS_SQ { /* adjacent */ }
```

The radius is a shortcut because the cells domain never exposed grid coordinates. It works today by accident — cell spacing happens to make radius ≈ orthogonal-only — but it's fragile:

- Any change to cell-grid spacing breaks the approximation.
- Diagonal adjacency silently slips in when cells are closer than `70` diagonally.
- Future cell layouts (non-uniform grid, rotated grid, sparse fields) break the model entirely.

Design is orthogonal-only (N/S/E/W, no diagonals). The impl should match, explicitly.

## Fix

### Expose grid coordinates on every cell

`cells/components/grid.rs` (new):

```rust
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CellGridPosition {
    pub row: u32,
    pub col: u32,
}
```

Every cell spawned from a layout gets a `CellGridPosition` at spawn time. The cell builder's `.at_grid_position(row, col)` transition is the canonical entry point; existing layout/generator code calls it instead of (or alongside) `.at_position(world_xy)`.

For cells spawned WITHOUT a grid slot (e.g., Fracture debris at an arbitrary offset, Echo Cells ghosts placed by formula), the spawn path computes a grid position from the world position via `world_to_grid(Vec2) -> CellGridPosition`. If the cell is genuinely detached from the grid (a hypothetical "free-position" cell), it gets NO `CellGridPosition` and adjacency queries skip it.

### Adjacency API

`cells/queries/adjacency.rs` (new):

```rust
/// Returns cells orthogonally adjacent to `center` (row ± 1 OR col ± 1, never diagonal).
/// Only cells with `CellGridPosition` are considered — detached cells are skipped.
pub fn orthogonal_neighbors<'w>(
    center: CellGridPosition,
    cells: &'w Query<(Entity, &'w CellGridPosition), With<Cell>>,
) -> impl Iterator<Item = Entity> + 'w {
    let (r, c) = (center.row as i32, center.col as i32);
    let targets = [(r - 1, c), (r + 1, c), (r, c - 1), (r, c + 1)];
    cells.iter().filter_map(move |(entity, pos)| {
        let pr = pos.row as i32;
        let pc = pos.col as i32;
        targets.iter().any(|&t| t == (pr, pc)).then_some(entity)
    })
}

/// Convert grid coordinates to world position (uses the project's canonical
/// cell-size constants — match whatever the layout generator uses).
pub fn grid_to_world(grid: CellGridPosition) -> Vec2 { ... }

/// Inverse of `grid_to_world`. Returns `None` for positions outside the valid grid.
pub fn world_to_grid(world: Vec2) -> Option<CellGridPosition> { ... }
```

Re-exported from `cells/mod.rs` via `pub use queries::adjacency::{orthogonal_neighbors, grid_to_world, world_to_grid};`.

### Game-side `CellDestroyed` companion message

The crate's `Destroyed<Cell>` carries `entity: Entity` + `source: SourceId` (generic). Cascade/Fracture/etc. need the grid position after the cell has despawned, so:

`cells/messages.rs`:

```rust
#[derive(Message, Debug, Clone)]
pub struct CellDestroyed {
    pub entity:   Entity,
    pub grid_pos: Option<CellGridPosition>,  // None for detached cells
    pub position: Vec2,                       // world-space, for systems that still need it
}
```

Cells-domain consumer — `cells/systems/emit_cell_destroyed/system.rs` (new):

```rust
pub(crate) fn emit_cell_destroyed(
    mut reader: MessageReader<Destroyed<Cell>>,
    cells: Query<(&Position2D, Option<&CellGridPosition>), With<Cell>>,
    mut writer: MessageWriter<CellDestroyed>,
) {
    for msg in reader.read() {
        // At this point the entity still exists (pipeline emits Destroyed BEFORE ProcessDespawn).
        let Ok((pos, grid)) = cells.get(msg.entity) else { continue };
        writer.write(CellDestroyed {
            entity:   msg.entity,
            grid_pos: grid.copied(),
            position: pos.0,
        });
    }
}
```

Schedule: `FixedUpdate`, `.after(DeathPipelineSystems::EmitKill)` so the snapshot captures the entity's state before `ApplyKill` / `ProcessDespawn` remove it.

Hazards (Cascade, Diffusion, Fracture) consume `CellDestroyed` instead of the crate's `Destroyed<Cell>` when they need grid-aware information. Systems that need only the entity ref (generic pipeline consumers) can still consume `Destroyed<Cell>`.

### Cascade migration

`mutators/hazards/cascade/system.rs`:

```rust
pub(crate) fn cascade_heal_on_death(
    mut reader: MessageReader<CellDestroyed>,
    cells: Query<(Entity, &CellGridPosition), With<Cell>>,
    config: Res<CascadeConfig>,
    mut writer: MessageWriter<HealDealt<Cell>>,
) {
    for destroyed in reader.read() {
        let Some(center) = destroyed.grid_pos else { continue };
        for neighbor in orthogonal_neighbors(center, &cells) {
            writer.write(HealDealt {
                target: neighbor,
                amount: config.heal_amount,
                source: SourceId::from("hazard:cascade"),
            });
        }
    }
}
```

Delete the distance-squared loop. Delete the `ADJACENCY_RADIUS_SQ` reference.

### Diffusion migration

Mirror Cascade's migration in `mutators/hazards/diffusion/system.rs` (or wherever Diffusion's chain-of-neighbors walk lives after #1/#2 migrate it to a `MessageMutator<DamageDealt<Cell>>` in the damage chain). Replace distance-squared loops with `orthogonal_neighbors` calls.

### Delete `ADJACENCY_RADIUS_SQ`

`cells/components/adjacency.rs:12`: delete the constant. Grep for any remaining consumer — if anything still uses it for a non-adjacency purpose (e.g., Tether's redirect radius), rename it to the specific use and keep that reference. Otherwise delete the file.

### Design-doc updates

`docs/architecture/cells.md` — add or update §Adjacency:

> Cell adjacency is grid-orthogonal. `CellGridPosition` is attached to every grid-placed cell at spawn time; `orthogonal_neighbors(center, cells)` is the canonical query helper. Only four neighbors per center (N/S/E/W); no diagonals. Cells without a `CellGridPosition` (detached / free-position cells) are excluded from adjacency queries entirely.
>
> `CellDestroyed { entity, grid_pos, position }` is the post-despawn companion to the crate's `Destroyed<Cell>`. Hazards that need grid-aware information after a cell is gone read `CellDestroyed`; systems that only need the entity ID read `Destroyed<Cell>`.

`docs/todos/detail/mod-system-design/hazards/cascade.md`: §Adjacency "Orthogonal via `orthogonal_neighbors`; radius-based adjacency retired."

`docs/todos/detail/mod-system-design/hazards/diffusion.md`: same.

## Tests

`cells/queries/adjacency/tests.rs`:

1. **`returns_four_orthogonals_in_full_grid`** — spawn 3×3 grid with each cell at its grid slot; assert `orthogonal_neighbors((1, 1))` returns exactly 4 cells: (0,1), (2,1), (1,0), (1,2).
2. **`corner_returns_two_orthogonals`** — 3×3 grid; assert `orthogonal_neighbors((0, 0))` returns exactly 2 cells: (0,1) and (1,0).
3. **`edge_returns_three_orthogonals`** — 3×3 grid; `orthogonal_neighbors((0, 1))` returns (0,0), (0,2), (1,1).
4. **`skips_detached_cells`** — spawn a mix of grid-placed cells and detached cells (no `CellGridPosition`). Assert the detached cells are never returned.
5. **`no_diagonals`** — 3×3 grid; assert (0,0) is NOT in the result for `orthogonal_neighbors((1,1))`.

`cells/systems/emit_cell_destroyed/tests.rs`:

6. **`emits_grid_pos_when_present`** — spawn grid cell at (2, 3); emit `Destroyed<Cell> { entity, .. }`; tick; assert one `CellDestroyed` message with `grid_pos == Some((2, 3))`.
7. **`emits_none_for_detached`** — spawn cell without `CellGridPosition`; emit `Destroyed<Cell>`; assert `CellDestroyed.grid_pos == None`.
8. **`emits_position_for_all_cells`** — regardless of grid status, `CellDestroyed.position` matches the cell's `Position2D`.

`mutators/hazards/cascade/tests/orthogonal_adjacency.rs`:

9. **`heals_four_orthogonal_neighbors`** — activate Cascade; 3×3 grid; destroy (1,1); assert four `HealDealt<Cell>` messages targeting (0,1), (2,1), (1,0), (1,2) — not the center, not the diagonals.
10. **`heals_two_for_corner_death`** — 3×3 grid; destroy (0,0); assert two heals targeting (0,1) and (1,0).

`mutators/hazards/diffusion/tests/orthogonal_adjacency.rs`:

11. **`diffuses_to_orthogonals_only`** — mirror of Cascade's orthogonal pattern for Diffusion's ring-share mechanic.

## Code changes summary

| File | Change |
|------|--------|
| `cells/components/grid.rs` | NEW — `CellGridPosition` component |
| `cells/builder/core/transitions.rs` | Add `.at_grid_position(row, col)` method; terminal inserts `CellGridPosition` |
| `cells/queries/adjacency.rs` | NEW — `orthogonal_neighbors`, `grid_to_world`, `world_to_grid` |
| `cells/mod.rs` | Re-export adjacency helpers |
| `cells/messages.rs` | Add `CellDestroyed { entity, grid_pos, position }` |
| `cells/plugin.rs` | Register `CellDestroyed` message + `emit_cell_destroyed` system |
| `cells/systems/emit_cell_destroyed/system.rs` | NEW — post-`Destroyed<Cell>` companion-emission |
| `cells/systems/emit_cell_destroyed/tests.rs` | NEW — tests 6-8 |
| `cells/queries/adjacency/tests.rs` | NEW — tests 1-5 |
| `cells/components/adjacency.rs` | DELETE `ADJACENCY_RADIUS_SQ` (or rename if a non-adjacency consumer exists) |
| `mutators/hazards/cascade/system.rs` | Replace distance-squared loop with `orthogonal_neighbors`; consume `CellDestroyed` instead of `Destroyed<Cell>` |
| `mutators/hazards/diffusion/system.rs` | Same as Cascade for Diffusion's ring walk |
| `mutators/hazards/cascade/tests/orthogonal_adjacency.rs` | NEW — tests 9-10 |
| `mutators/hazards/diffusion/tests/orthogonal_adjacency.rs` | NEW — test 11 |
| `docs/architecture/cells.md` | §Adjacency section per Fix |
| `docs/todos/detail/mod-system-design/hazards/cascade.md` | §Adjacency update |
| `docs/todos/detail/mod-system-design/hazards/diffusion.md` | §Adjacency update |
| Layout/generator code | Call `.at_grid_position(row, col)` when spawning grid cells |

## Out of scope

- Sparse / non-uniform / rotated grid layouts (future feature; `CellGridPosition` supports sparse grids by design since adjacency is a lookup, not a linear scan).
- Extending adjacency to diagonals (design is orthogonal-only; if a future hazard needs 8-neighborhood, add `king_move_neighbors` as a separate helper).
- Grid-position visualization in debug UI — Phase 5 debug-render work.
