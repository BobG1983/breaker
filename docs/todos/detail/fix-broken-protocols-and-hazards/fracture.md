# Fracture

Assumes: #1 (`rantzsoft_dmg` — `Destroyed<Cell>` message + `DeathPipelineSystems::HandleKill`/`ApplyHeal`), #2 (`mutators/hazards/fracture/`). Couples with `grid.md` (this TODO's `grid-orthogonal-adjacency` item) because Fracture uses grid positions for debris placement.

## What's broken

1. **No empty-position check.** Debris spawns overlap existing cells. Behaviors 3 ("cap at available positions") and 4 ("no spawns if no empty neighbors") fail silently.
2. **Direct cell spawn bypasses the builder.** Same invisibility class as Echo Cells and Momentum — debris lacks `Mesh2d`, `MeshMaterial2d`, `GameDrawLayer::Cell`.
3. **Recursive fracture is unguarded.** When a `FractureDebris` cell itself gets destroyed, Fracture runs again and spawns more debris. Exponential growth at high stacks. User direction: debris does NOT re-fracture.
4. **No Cascade ordering.** Cascade and Fracture both react to `Destroyed<Cell>`. Without ordering, Fracture can spawn debris before Cascade's iteration, and Cascade then heals the brand-new debris — wrong.

## Fix

### Add `.debris()` transition to the cell builder

`cells/builder/core/transitions.rs`:

```rust
impl<P, H, V, T> CellBuilder<P, H, V, T> {
    /// Mark the cell as Fracture debris. Inserts `FractureDebris` marker alongside
    /// the standard rendered cell component set.
    pub fn debris(mut self) -> Self {
        self.optional.debris = true;
        self
    }
}
```

Terminal reads `optional.debris` and inserts `FractureDebris` onto the spawned entity. Visual distinction (smaller scale, dimmer tint) deferred to Phase 5.

### Recursion guard

`mutators/hazards/fracture/system.rs` — at the top of the `for destroyed in reader.read()` loop:

```rust
// Debris does not spawn more debris. Prevents exponential cell growth.
if debris_query.get(destroyed.victim).is_ok() {
    continue;
}
```

`debris_query: Query<(), With<FractureDebris>>`. Cheap marker presence check; no mutation.

This check runs against the PRE-DESPAWN entity state — at the moment `Destroyed<Cell>` fires, the entity still exists in the world (the crate's pipeline emits `Destroyed<T>` before `ProcessDespawn` runs the actual despawn). The query works.

### Empty-position check (using the grid-orthogonal API from `grid.md`)

`mutators/hazards/fracture/system.rs`:

```rust
pub(crate) fn fracture_on_death(
    mut reader: MessageReader<CellDestroyed>,           // grid-extended companion from grid.md
    debris_query: Query<(), With<FractureDebris>>,
    cells: Query<(Entity, &CellGridPosition), With<Cell>>,
    config: Res<FractureConfig>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for destroyed in reader.read() {
        if debris_query.get(destroyed.entity).is_ok() { continue }
        let Some(victim_grid) = destroyed.grid_pos else { continue };

        for offset in FRACTURE_OFFSETS.iter().take(config.max_debris as usize) {
            let target_grid = CellGridPosition {
                row: victim_grid.row.wrapping_add_signed(offset.dr),
                col: victim_grid.col.wrapping_add_signed(offset.dc),
            };

            let occupied = cells.iter().any(|(_, pos)| *pos == target_grid);
            if occupied { continue }

            // Compute world position from grid position via the cells-domain helper.
            let target_world = grid_to_world(target_grid);

            Cell::builder()
                .at_position(target_world)
                .hp(Hp::new(1.0))
                .debris()
                .spawn(&mut commands);
        }
    }
}
```

Behavior 3 ("cap at available positions") and Behavior 4 ("no spawns if no empty neighbors") emerge naturally from the skip-on-occupied logic — no explicit handling needed.

`FRACTURE_OFFSETS` becomes a const array of `(dr, dc)` grid-space deltas (not world-space). Adjust to match Fracture's 4 or 8 target positions.

`grid_to_world(grid_pos)` is a helper exposed by the cells domain (introduced alongside `orthogonal_neighbors` in `grid.md`).

### Ordering with Cascade

Both Cascade's `cascade_heal_on_death` and Fracture's `fracture_on_death` read `Destroyed<Cell>` (or the `CellDestroyed` companion). Correct order:

1. `cascade_heal_on_death` — iterates cell query with Fracture's spawns NOT yet present. Emits `HealDealt<Cell>` for pre-existing neighbors.
2. `fracture_on_death` — spawns new debris after Cascade's iteration.
3. `apply_heal::<Cell>` / `apply_damage::<Cell>` from the crate's pipeline — runs after both.

Registration:

```rust
fracture_on_death
    .after(DeathPipelineSystems::HandleKill)
    .after(cascade_heal_on_death)
    .before(DeathPipelineSystems::ApplyHeal)
```

### Delete

- `FRACTURE_DEBRIS_WIDTH`, `FRACTURE_DEBRIS_HEIGHT`, or similar hardcoded dimension constants.
- Manual `Position2D`, `Scale2D`, `Aabb2D`, `CollisionLayers`, `CellWidth`, `CellHeight`, `Hp`, `KilledBy`, `Cell`, `FractureDebris` insertions at the Fracture spawn site.
- Any radius-based adjacency distance checks (replaced by grid-space comparison).

### Design-doc updates

`docs/todos/detail/mod-system-design/hazards/fracture.md`:
- §Messages — delete any `SpawnDebrisCell` proposal. Fracture uses `Cell::builder()` directly.
- §Edge Cases — "Empty-position check: if the target grid slot is occupied, skip it. Zero debris spawns if every offset is occupied."
- §Edge Cases — add "Recursive fracture: debris cells DO NOT fracture on their own destruction."
- §Ordering — add "Runs after `cascade_heal_on_death` so Cascade does not heal newly-spawned debris."

`docs/todos/detail/mod-system-design/hazards/cascade.md` (ride with Fracture's TODO):
- §Ordering — add "Runs before `fracture_on_death` so Fracture-spawned cells are not included in the same-frame heal set."

## Tests

`cells/builder/tests/debris.rs`:

1. **`debris_transition_inserts_marker_and_renders`** — `.debris().spawn()`; assert `Cell`, `FractureDebris`, `Mesh2d`, `MeshMaterial2d`, `GameDrawLayer::Cell` present.

`mutators/hazards/fracture/tests/skip_occupied_positions.rs`:

2. **`skips_occupied_neighbor_slots`** — grid with cell at (1, 2). Drive `CellDestroyed { grid_pos: Some((1, 1)) }` (neighbor at (1, 2)). Assert zero debris spawned at (1, 2); debris at (1, 0), (0, 1), (2, 1) if those are empty.
3. **`no_empty_neighbors_no_spawn`** — surround victim with cells at every offset. Drive destruction. Assert zero new debris entities.
4. **`partial_fill_partial_spawn`** — half the offsets occupied, half empty. Drive destruction. Assert debris spawned ONLY at empty offsets.

`mutators/hazards/fracture/tests/no_recursion_on_debris.rs`:

5. **`debris_death_spawns_no_more_debris`** — spawn a `FractureDebris` cell via `.debris()`. Drive `CellDestroyed { entity: debris_entity, .. }`. Assert zero new debris entities.

`mutators/hazards/fracture/tests/ordering_with_cascade.rs`:

6. **`cascade_does_not_heal_fracture_spawned_cells`** — activate Cascade AND Fracture. Grid with empty orthogonal neighbors around victim. Drive destruction. Collect `HealDealt<Cell>` messages. Assert target set includes only pre-existing neighbors — no heal targets an entity that Fracture spawned this frame.

`mutators/hazards/cascade/tests/ordering_with_fracture.rs`:

7. **`heal_set_excludes_debris`** — mirror of test 6 from Cascade's side.

`mutators/hazards/fracture/tests/debris_visibility.rs`:

8. **`debris_has_mesh_and_draw_layer`** — drive destruction; query spawned debris; assert `Mesh2d`, `MeshMaterial2d`, `GameDrawLayer::Cell`, `FractureDebris`. Regression for invisible-debris.

## Code changes summary

| File | Change |
|------|--------|
| `cells/builder/core/types.rs` | Add `debris: bool` to `OptionalCellData` |
| `cells/builder/core/transitions.rs` | Add `.debris()` method |
| `cells/builder/core/terminal.rs` | When `optional.debris`, insert `FractureDebris` on spawned entity |
| `cells/builder/tests/debris.rs` | NEW — test 1 |
| `mutators/hazards/fracture/system.rs` | REWRITE `fracture_on_death` per Fix section (grid-offset logic + occupied check + builder call + recursion guard); DELETE manual component inserts + hardcoded dimension constants |
| `mutators/hazards/fracture/register.rs` | Add `.after(DeathPipelineSystems::HandleKill).after(cascade_heal_on_death).before(DeathPipelineSystems::ApplyHeal)` |
| `mutators/hazards/fracture/tests/skip_occupied_positions.rs` | NEW — tests 2-4 |
| `mutators/hazards/fracture/tests/no_recursion_on_debris.rs` | NEW — test 5 |
| `mutators/hazards/fracture/tests/ordering_with_cascade.rs` | NEW — test 6 |
| `mutators/hazards/cascade/tests/ordering_with_fracture.rs` | NEW — test 7 |
| `mutators/hazards/fracture/tests/debris_visibility.rs` | NEW — test 8 |
| `docs/todos/detail/mod-system-design/hazards/fracture.md` | Design-doc updates per Fix section |
| `docs/todos/detail/mod-system-design/hazards/cascade.md` | §Ordering update |

## Out of scope

- Debris visual distinction (smaller scale, dimmer tint) — Phase 5i cell visuals.
- Cascade's heal tuning (RON values) — `ron-tuning-values.md`.
- Multi-hop fracture (debris that fractures into more debris) — user direction is "no," and the recursion guard pins it.
