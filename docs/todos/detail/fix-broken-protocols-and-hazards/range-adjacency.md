# Range-based adjacency

Replaces the retired grid-orthogonal adjacency spec. Cells, bosses, and any other damage targets are queried by **range** via `rantzsoft_physics2d`'s quadtree — not by grid coordinates.

Assumes: #1 (`Destroyed<T>` is the generic death message), #2 (hazards live at `mutators/hazards/`).

Cross-cutting: drops `CellGridPosition` from runtime mechanics. Cascade, Diffusion, Fracture, Echo Strike, Iron Curtain, and Debt Collector all migrate to range queries.

## Why range, not grid

Bosses are made of cells but will animate off-grid — a translating / rotating / deforming cluster. `CellGridPosition`-based adjacency (`orthogonal_neighbors`) breaks the moment a cell leaves its slot. Range queries are the single code path that works for:

- Static grid cells (radius tuned to capture the 4 orthogonal neighbors exactly — same visual "grid feel")
- Animated boss cells (adjacency follows the body wherever it moves)
- Future layouts (non-uniform, sparse, rotated, polar)

One query primitive, no dual code path, no grid-membership gate.

## What's broken

Cascade and Diffusion currently use a hardcoded distance-squared check:

```rust
const ADJACENCY_RADIUS_SQ: f32 = 70.0 * 70.0;
if pos.0.distance_squared(victim_pos) < ADJACENCY_RADIUS_SQ { /* adjacent */ }
```

It works by accident for today's grid spacing. Problems:
- Linear scan over every cell per query — no spatial index.
- Hardcoded constant; not tunable per-mechanic.
- Not exposed as an API — every consumer reimplements the loop.

## Fix

### Adjacency via quadtree

Game code never iterates cells manually for spatial lookups. Use `rantzsoft_physics2d`'s quadtree:

```rust
// from rantzsoft_physics2d::quadtree::Quadtree
pub fn query_circle(&self, center: Vec2, radius: f32) -> Vec<Entity>;
pub fn query_circle_filtered(&self, center: Vec2, radius: f32, layers: CollisionLayers) -> Vec<Entity>;
```

Consumers query the quadtree and filter with Bevy queries as usual (`With<Cell>`, `Without<Dead>`, etc.). The quadtree is already rebuilt each tick by the physics plugin.

### Per-mechanic radii in RON

Each mechanic that queries "nearby cells" declares its radius in RON so tuning is data-driven:

- Cascade: `neighbor_radius: f32` — the visual heal-arc radius; tuned to catch orthogonal grid neighbors.
- Diffusion: `ring_radius: f32` — distance between concentric rings; BFS-style growth replaced with concentric range bands (ring N = `[N * ring_radius, (N+1) * ring_radius]`).
- Fracture: `debris_radius: f32` + offset set `[(+r, 0), (-r, 0), (0, +r), (0, -r)]` — debris spawn points, not slot lookups.
- Iron Curtain: already distance-based via `falloff_distance`; no change, just confirm it uses quadtree filtering.
- Echo Strike: `echo_radius: f32` — cells within radius of echo-network nodes.
- Debt Collector: on cash-out, damage is single-target; no adjacency needed (remove any prior grid-adjacency framing).

Radii are picked so static-grid layouts capture the 4 orthogonal neighbors exactly (`radius ≈ cell_spacing * 0.6`). Boss layouts use whatever radius makes the mechanic feel right.

### Cell position as world-space truth

Every cell has a `Position2D` (via `rantzsoft_spatial2d`) as its spatial truth. `CellGridPosition` — if it exists at all — is layout metadata (used by the layout generator and by debug UI), not by runtime mechanics.

If `CellGridPosition` has only layout use, it stays on the layout side and is NOT required on boss cells. If it's referenced nowhere else, delete it.

### Game-side `CellDestroyed` companion message

The crate's `Destroyed<Cell>` carries `entity: Entity` + `source: SourceId` (generic). Range consumers need the cell's world position AFTER despawn (the entity is gone by the time they react):

`cells/messages.rs`:

```rust
#[derive(Message, Debug, Clone)]
pub struct CellDestroyed {
    pub entity:   Entity,
    pub position: Vec2,   // world-space snapshot, captured pre-despawn
}
```

`cells/systems/emit_cell_destroyed/system.rs`:

```rust
pub(crate) fn emit_cell_destroyed(
    mut reader: MessageReader<Destroyed<Cell>>,
    cells: Query<&Position2D, With<Cell>>,
    mut writer: MessageWriter<CellDestroyed>,
) {
    for msg in reader.read() {
        let Ok(pos) = cells.get(msg.entity) else { continue };
        writer.write(CellDestroyed { entity: msg.entity, position: pos.0 });
    }
}
```

Schedule: `FixedUpdate`, `.after(DeathPipelineSystems::EmitKill)` so the snapshot captures before `ApplyKill` / `ProcessDespawn` removes the entity.

### Cascade migration

```rust
pub(crate) fn cascade_heal_on_death(
    mut reader: MessageReader<CellDestroyed>,
    quadtree: Res<Quadtree>,
    cells: Query<Entity, (With<Cell>, Without<Dead>)>,
    config: Res<CascadeConfig>,
    hazards: Option<Res<ActiveHazards>>,
    mut writer: MessageWriter<HealDealt<Cell>>,
) {
    let stack = hazards.and_then(|h| h.stack(HazardKind::Cascade)).unwrap_or(0);
    if stack == 0 { return; }
    let heal_amount = config.base_heal + config.heal_per_level * (stack as f32 - 1.0);

    for destroyed in reader.read() {
        for entity in quadtree.query_circle(destroyed.position, config.neighbor_radius) {
            if entity == destroyed.entity { continue; }
            if cells.get(entity).is_err() { continue; }
            writer.write(HealDealt {
                target: entity,
                amount: heal_amount,
                cap: HealCap::Starting,
                source: Some("hazard:cascade".into()),
                ..default()
            });
        }
    }
}
```

### Diffusion migration

Replace the grid-BFS with concentric range bands. For each `DamageDealt<Cell>`:

1. Compute `share`, `depth` as before.
2. For ring N in 1..=depth: `cells_in_ring = quadtree.query_circle(victim_pos, (N+1) * ring_radius) - quadtree.query_circle(victim_pos, N * ring_radius)`.
3. Divide the ring's share uniformly across `cells_in_ring` (same flat-share-per-ring semantics as the grid version).

### Fracture migration

Spawn debris at fixed offset positions around the kill point. Each offset candidate is checked by range query: if any cell already overlaps the spawn point (`query_circle(offset_pos, cell_half_size)` non-empty), skip that slot.

### Delete `ADJACENCY_RADIUS_SQ`

`cells/components/adjacency.rs:12`: delete the constant + file if nothing else uses it.

### Design-doc updates

`docs/architecture/cells.md` — add or update §Adjacency:

> Cell adjacency is **range-based**. Mechanics that query "nearby cells" use `rantzsoft_physics2d::Quadtree::query_circle(center, radius)` with a per-mechanic radius from RON tuning. This applies uniformly to static grid cells and animated boss cells. `CellGridPosition` (if retained) is layout metadata — never consulted by runtime mechanics.
>
> `CellDestroyed { entity, position }` is the post-despawn companion to the crate's `Destroyed<Cell>`. Hazards that need a position after the cell is gone read `CellDestroyed`; systems that only need the entity ID read `Destroyed<Cell>`.

Canonical design docs at `docs/design/hazards/{cascade,diffusion,fracture,echo_cells}.md` and `docs/design/protocols/{echo_strike,iron_curtain}.md` reference the range-query mechanism and the per-mechanic radius tuning.

## Tests

`cells/systems/emit_cell_destroyed/tests.rs`:

1. **`emits_position_snapshot`** — destroy cell at `Position2D((42.0, 17.0))`; assert `CellDestroyed.position == (42.0, 17.0)`.
2. **`emits_one_per_death`** — 3 cells destroyed same tick; 3 `CellDestroyed` messages.
3. **`skips_entity_already_despawned`** — if the cell's `Position2D` is unreadable, no message emitted (no panic).

`mutators/hazards/cascade/tests/range_adjacency.rs`:

4. **`heals_cells_within_radius`** — Cascade active; 3×3 grid at spacing 60; `neighbor_radius: 70`; destroy center; assert 4 `HealDealt<Cell>` messages targeting the 4 orthogonally-adjacent cells (distance 60 < 70), NONE targeting diagonals (distance ≈ 85 > 70).
5. **`heals_boss_cells_following_body`** — spawn 4 cells in a translated cluster (no grid positions); destroy one; assert the other 3 are healed (all within radius).
6. **`self_excluded`** — destroyed cell not a target of its own heal.
7. **`dead_excluded`** — cells with `Dead` marker not targeted.

`mutators/hazards/diffusion/tests/range_bands.rs`:

8. **`ring_1_catches_orthogonals_at_grid_spacing`** — 3×3 grid at spacing 60, `ring_radius: 70`; hit center; assert ring 1 contains the 4 orthogonals and NOT the diagonals.
9. **`ring_2_catches_distance_two_cells`** — 5×5 grid; depth 2; assert ring-2 share reaches cells ~120 away.

`mutators/hazards/fracture/tests/range_spawn.rs`:

10. **`skips_slot_if_occupied`** — destroy a cell at position P; one of the 4 offset slots already has a cell; that slot skipped, remaining slots filled.
11. **`all_four_empty`** — 4 empty offset slots; stack 3 (wants 4): 4 debris spawned.

## Code changes summary

| File | Change |
|------|--------|
| `cells/messages.rs` | Add `CellDestroyed { entity, position: Vec2 }` |
| `cells/plugin.rs` | Register `CellDestroyed` + `emit_cell_destroyed` system |
| `cells/systems/emit_cell_destroyed/system.rs` | NEW — post-`Destroyed<Cell>` companion |
| `cells/systems/emit_cell_destroyed/tests.rs` | NEW — tests 1-3 |
| `cells/components/adjacency.rs` | DELETE `ADJACENCY_RADIUS_SQ` (or file if otherwise unused) |
| `cells/components/grid.rs` | OPTIONAL: keep `CellGridPosition` only if layout needs it; otherwise delete |
| `mutators/hazards/cascade/system.rs` | Use `Quadtree::query_circle`; read `CellDestroyed`; drop any `CellGridPosition` reference |
| `mutators/hazards/cascade/config.rs` | Add `neighbor_radius: f32` |
| `mutators/hazards/diffusion/system.rs` | Replace BFS with concentric-range bands |
| `mutators/hazards/diffusion/config.rs` | Replace depth logic with `ring_radius: f32` + `max_depth: u32` |
| `mutators/hazards/fracture/system.rs` | Fixed-offset world positions + `query_circle` occupancy check |
| `mutators/hazards/fracture/config.rs` | Add `debris_radius: f32` |
| `mutators/protocols/echo_strike/system.rs` | Use `query_circle` for echo-network reach |
| `mutators/protocols/iron_curtain/system.rs` | Already distance-based; confirm uses quadtree |
| Test files | Migrate `returns_four_orthogonals` etc. to radius-based equivalents |
| Layout/generator code | Layout still places cells at grid slots — no change |
| `docs/architecture/cells.md` | §Adjacency per Fix |

## Out of scope

- Alternate query shapes (AABB, cone): add when a mechanic needs them.
- Grid-as-constraint layouts: layouts can still be grid-authored; runtime doesn't care.
- `CellGridPosition` itself: retaining it as layout metadata is fine, dropping it entirely is fine — both choices are local to this TODO.
