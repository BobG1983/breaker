# Momentum

Assumes: #2 (`mutators/hazards/momentum/`). Shares the builder-migration concern with echo-cells and fracture but each mechanic's builder transition is separate.

## What's broken

1. **Split cells are INVISIBLE.** Momentum's split spawn at `system.rs:332-348` uses direct `commands.spawn((...))` bypassing `Cell::builder()`. No `Mesh2d`, no `MeshMaterial2d`, no `GameDrawLayer::Cell`. Cells exist mechanically but are invisible to the player.
2. **Split cells miss standard components.** No `CellTypeAlias`, no `RequiredToClear`, no standard collision setup. Follow-on hazard systems that assume `With<Cell>` + `CellTypeAlias` skip these entities silently.
3. **Dynamic hazard components may not attach.** `VolatilityTimer`, `MagneticField`, etc. are added by `attach_X_timers` systems each FixedUpdate. No test pins that newly-split Momentum cells actually pick up these hazards within one tick.

## Fix

### Migrate to `Cell::builder()`

`mutators/hazards/momentum/system.rs` around lines 332-348, replace the manual spawn with:

```rust
Cell::builder()
    .at_position(target_pos)
    .hp(Hp {
        current: parent_hp.starting,
        starting: parent_hp.starting,
        max: Some(parent_hp.starting * MOMENTUM_SPLIT_MULTIPLIER),
    })
    .spawn(&mut commands);
```

No new builder transition — the standard `.spawn()` produces a fully-rendered cell with all standard components (`Mesh2d`, `MeshMaterial2d`, `GameDrawLayer::Cell`, `CellTypeAlias`, `RequiredToClear`, `Cell`, `Position2D`, `Scale2D`, `Aabb2D`, `CellWidth`, `CellHeight`, `CollisionLayers`, `Hp`, `KilledBy`).

Momentum's split cells do not need a distinguishing marker (unlike Ghost or Debris) — they're just smaller/larger normal cells. No `.split()` transition needed.

Delete:
- Manual insertions of `Position2D`, `Scale2D`, `Aabb2D`, `CellWidth`, `CellHeight`, `CollisionLayers`, `CellTypeAlias`, `Cell`, `Hp`, `KilledBy`.
- Any hardcoded `MOMENTUM_SPLIT_WIDTH`/`MOMENTUM_SPLIT_HEIGHT` constants (grep to confirm — similar to Fracture/Echo Cells).

The builder owns "what a cell is." Future changes to cell shape (different base dimensions, different collision layers) only touch the builder — Momentum's call site stays stable.

### Dynamic hazard component inheritance

`attach_volatility_timers` (and any other `attach_X_timers`) queries `Query<Entity, (With<Cell>, Without<VolatilityTimer>)>` every FixedUpdate. After a Momentum split lands, the new cell is missing the hazard-specific component — the attach system picks it up on the next tick.

One-frame delay is acceptable: no hazard fires on the frame a cell spawns anyway (Momentum's split itself triggers from `Destroyed<Cell>`, and the attach systems run after spawn). Pin the delay explicitly in tests so it can't regress to longer.

### Design-doc updates

`docs/todos/detail/mod-system-design/hazards/momentum.md`:

§Messages / §Edge Cases — add:

> Split cells spawn via `Cell::builder()` and receive the standard cell component set (mesh, render layer, collision, required-to-clear). Dynamic hazard components (VolatilityTimer, etc.) attach on the next FixedUpdate via each hazard's `attach_X_timers` system. There is a one-tick delay between split spawn and full hazard attachment. This delay is acceptable — no hazard effect fires on a cell's spawn frame.

## Tests

`mutators/hazards/momentum/tests/split_cell_rendered.rs`:

1. **`split_cell_has_mesh_and_draw_layer`** — activate Momentum; drive a split (emit the trigger message, whatever it is today); query the newly-spawned cell; assert `Mesh2d`, `MeshMaterial2d`, `GameDrawLayer::Cell`, `CellTypeAlias`, `RequiredToClear`. This is the regression that catches the invisible-cells bug.
2. **`split_cell_has_collision_components`** — same setup; assert `CollisionLayers`, `Aabb2D`, `Scale2D` present and correct.
3. **`split_cell_hp_matches_config`** — parent cell with `Hp { starting: 2.0, .. }`; `MOMENTUM_SPLIT_MULTIPLIER == 0.5` (or whatever); trigger split; assert new cell's `Hp.starting == 2.0` and `Hp.max == Some(1.0)` (or whatever the formula yields).

`mutators/hazards/momentum/tests/split_cell_gets_volatility_within_one_tick.rs`:

4. **`split_cell_lacks_volatility_on_spawn_frame`** — activate Momentum AND Volatility; trigger split; do NOT advance FixedUpdate past the spawn tick; assert the spawned cell has NO `VolatilityTimer`.
5. **`split_cell_has_volatility_after_one_tick`** — continuation of test 4: advance one FixedUpdate tick; assert the cell NOW has `VolatilityTimer`. Pins the one-tick-delay invariant explicitly.

## Code changes summary

| File | Change |
|------|--------|
| `mutators/hazards/momentum/system.rs` | Replace `commands.spawn(...)` at the split path with `Cell::builder()...spawn()`; delete manual component inserts; delete `MOMENTUM_SPLIT_WIDTH` / `MOMENTUM_SPLIT_HEIGHT` constants if present |
| `mutators/hazards/momentum/tests/split_cell_rendered.rs` | NEW — tests 1-3 |
| `mutators/hazards/momentum/tests/split_cell_gets_volatility_within_one_tick.rs` | NEW — tests 4-5 |
| `docs/todos/detail/mod-system-design/hazards/momentum.md` | §Messages / §Edge Cases update per Fix section |

## Out of scope

- `attach_volatility_timers` implementation — already idempotent and running every tick; no change needed.
- Changing WHEN Momentum splits (the trigger condition) — unchanged.
- Split-cell visual distinction (different color/tint) — Phase 5i cell visuals.
