# Echo Cells

Assumes: #2 (`mutators/hazards/echo_cells/`).

## What's broken

1. **Ghost cells are INVISIBLE.** Echo Cells spawns ghosts from its hazard system via direct `commands.spawn(...)` bypassing the cell builder. No `Mesh2d`, `MeshMaterial2d`, `GameDrawLayer::Cell`. Cells exist mechanically but invisible.
2. **Cross-domain spawn violation.** Hazard domain spawning `Cell` entities directly without going through the cell builder's spawn path — `plugins.md` would need an exception; builder migration removes the violation entirely.
3. **HP scaling uses a bare `f32` multiplier.** Current config carries `per_level_multiplier: f32` (geometric). Design spec'd `hp_doubles_per_level: bool`. User direction is a proper `HpScaling` enum supporting multiple modes.

## Fix

### Add `.ghost()` transition to the cell builder

`cells/builder/core/transitions.rs`:

```rust
impl<P, H, V, T> CellBuilder<P, H, V, T> {
    /// Mark the cell as a ghost (Echo Cells hazard). Inserts `GhostCell` marker at
    /// spawn; the standard rendered terminal still provides `Mesh2d`, `MeshMaterial2d`,
    /// `GameDrawLayer::Cell`.
    pub fn ghost(mut self) -> Self {
        self.optional.ghost = true;
        self
    }
}
```

Builder terminal reads `optional.ghost` at spawn and inserts `GhostCell` onto the spawned entity alongside the standard component set. Visual distinction (different tint or transparency) is deferred to Phase 5 — for this TODO, ghosts look like normal cells but carry the marker.

### `HpScaling` enum

`mutators/hazards/echo_cells/resources.rs` (or `system.rs` — match current layout):

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HpScaling {
    /// HP = base_hp * multiplier.powi(stacks - 1). Geometric.
    Geometric { multiplier: f32 },
    /// HP = base_hp + step * (stacks - 1). Linear.
    Linear { step: f32 },
    /// HP = base_hp * 2^(stacks - 1). Sugar for Geometric { multiplier: 2.0 }.
    Doubles,
}

#[derive(Resource, Debug, Clone)]
pub(crate) struct EchoCellsConfig {
    pub delay_secs: f32,
    pub base_hp:    f32,
    pub hp_scaling: HpScaling,
}

impl EchoCellsConfig {
    pub fn hp_for_stacks(&self, stacks: u32) -> f32 {
        let n = stacks.saturating_sub(1) as i32;
        match &self.hp_scaling {
            HpScaling::Geometric { multiplier } => self.base_hp * multiplier.powi(n),
            HpScaling::Linear    { step }       => self.base_hp + step * n as f32,
            HpScaling::Doubles                  => self.base_hp * 2_f32.powi(n),
        }
    }
}
```

`Doubles` is sugar for `Geometric { multiplier: 2.0 }` — makes RON self-documenting without losing flexibility.

### Migrate the ghost spawner

`mutators/hazards/echo_cells/system.rs` — at the `commands.spawn(...)` site around lines 224-239:

```rust
Cell::builder()
    .at_position(pending.position)
    .hp(Hp::new(config.hp_for_stacks(stacks)))
    .ghost()
    .spawn(&mut commands);
```

Delete:
- `GHOST_WIDTH` / `GHOST_HEIGHT` constants at `system.rs:17-19`.
- Manual inserts of `Position2D`, `Scale2D`, `Aabb2D`, `CollisionLayers`, `CellWidth`, `CellHeight`, `Hp`, `KilledBy`, `Cell`, `GhostCell`.
- Any `GameDrawLayer::Cell` insertion (builder handles it).

The builder owns what "a ghost cell is." Future shape changes only touch the builder.

### `HazardTuning::EchoCells` enum variant

`hazard/tuning.rs` (or wherever the tuning enum lives post-#2):

```rust
HazardTuning::EchoCells {
    delay_secs: f32,
    base_hp:    f32,
    hp_scaling: HpScaling,  // was `per_level_multiplier: f32`
}
```

`activate` unpacks and constructs `EchoCellsConfig`.

### RON shape

`assets/hazards/echo_cells.hazard.ron`:

```ron
(
    delay_secs: 1.5,
    base_hp:    1.0,
    hp_scaling: Doubles,
)
```

Current shipped RON uses `per_level_multiplier: 2.0`, which equals `Doubles` — behavior unchanged after migration.

Verify RON's tagged-enum deserialization pattern matches the project's existing enum-in-RON conventions (`CellKind`, etc.) — use the same syntax.

### Design-doc updates

`docs/todos/detail/mod-system-design/hazards/echo_cells.md`:
- §Config Resource — document `HpScaling` enum + the three variants' formulas.
- §Expected Behaviors 2 — rename to "HP scales by `hp_scaling` mode"; worked examples for each variant.
- §Messages — delete any `SpawnGhostCell` reference. Echo Cells spawns via the builder directly.
- §Systems — `echo_cells_spawn_ghosts` calls `Cell::builder().at_position(pos).hp(Hp::new(config.hp_for_stacks(stacks))).ghost().spawn(&mut commands)`.

## Tests

`cells/builder/tests/ghost.rs`:

1. **`ghost_transition_inserts_marker_and_renders`** — `.ghost().spawn()`; assert spawned entity has `Cell`, `GhostCell`, `Mesh2d`, `MeshMaterial2d`, `GameDrawLayer::Cell`, `Position2D`, `Hp`, `CollisionLayers`.
2. **`ghost_without_transition_has_no_marker`** — standard `.spawn()` without `.ghost()`; assert `GhostCell` absent (regression for builder accidentally always inserting it).

`mutators/hazards/echo_cells/tests/hp_scaling.rs`:

3. **`geometric_scaling`** — `HpScaling::Geometric { multiplier: 1.5 }`, `base_hp: 2.0`. `hp_for_stacks(1) == 2.0`; `hp_for_stacks(3) == 4.5` (`2.0 * 1.5^2`).
4. **`linear_scaling`** — `HpScaling::Linear { step: 2.0 }`, `base_hp: 1.0`. `hp_for_stacks(1) == 1.0`; `hp_for_stacks(3) == 5.0` (`1.0 + 2.0 * 2`).
5. **`doubles_scaling`** — `HpScaling::Doubles`, `base_hp: 1.0`. `hp_for_stacks(1) == 1.0`; `hp_for_stacks(3) == 4.0` (`1.0 * 2^2`).
6. **`saturating_at_stacks_zero`** — `hp_for_stacks(0) == base_hp` for all three variants (no underflow on `stacks - 1`).

`mutators/hazards/echo_cells/tests/ghost_spawn.rs`:

7. **`ghost_spawn_produces_rendered_entity`** — activate Echo Cells; drive the ghost-spawn trigger; query the spawned ghost; assert `GhostCell` present, `Mesh2d` present, `MeshMaterial2d` present. Regression catch for the invisible-ghost bug.

## Code changes summary

| File | Change |
|------|--------|
| `cells/builder/core/types.rs` | Add `ghost: bool` to `OptionalCellData` (or equivalent) |
| `cells/builder/core/transitions.rs` | Add `.ghost()` method |
| `cells/builder/core/terminal.rs` | When `optional.ghost`, insert `GhostCell` on spawned entity |
| `cells/builder/tests/ghost.rs` | NEW — tests 1-2 |
| `mutators/hazards/echo_cells/resources.rs` | Add `HpScaling` enum + `hp_for_stacks()` method; `EchoCellsConfig` uses `hp_scaling: HpScaling` |
| `mutators/hazards/echo_cells/system.rs` | Replace `commands.spawn(...)` with `Cell::builder()...ghost().spawn()`; delete manual inserts and `GHOST_WIDTH/_HEIGHT` constants |
| `hazard/tuning.rs` | `HazardTuning::EchoCells` variant carries `hp_scaling: HpScaling` instead of `per_level_multiplier: f32` |
| `assets/hazards/echo_cells.hazard.ron` | `per_level_multiplier: 2.0` → `hp_scaling: Doubles` |
| `mutators/hazards/echo_cells/tests/hp_scaling.rs` | NEW — tests 3-6 |
| `mutators/hazards/echo_cells/tests/ghost_spawn.rs` | NEW — test 7 |
| `docs/todos/detail/mod-system-design/hazards/echo_cells.md` | Design-doc updates per Fix section |

## Out of scope

- Visual distinction between ghost and normal cells (tint/transparency) — Phase 5i cell visuals.
- Echo Cells RON tuning values beyond the `hp_scaling` enum conversion — if shipped values are wrong, goes to `ron-tuning-values.md`.
- New `HpScaling` variants beyond the three defined here — add when a concrete caller needs them.
