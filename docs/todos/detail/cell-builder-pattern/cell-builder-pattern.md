# Cell Builder Pattern

## Summary
Apply the typestate builder pattern to Cell entities, replacing manual tuple assembly. Restructure cell behaviors into `cells/behaviors/` folders.

## Context
The Bolt and Breaker entities use typestate builder patterns (`Bolt::builder()...build()/spawn()`, `Breaker::builder()...build()/spawn()`). Wall entities have `Wall::builder()`. Cell entities still use manual component assembly in `spawn_cells_from_grid`. Consistency across entity types simplifies the codebase and enforces component completeness at compile time.

Cells are more complex than other entities — they have multiple variant behaviors (locked, regen, shielded) that are conditionally applied based on `CellTypeDefinition` data loaded from RON.

## Decisions

### Hybrid typestate + runtime config

**4 typestate dimensions** (must be set before `build()`/`spawn()`):

| Dimension | States | What it gates |
|-----------|--------|---------------|
| **Position** | `NoPosition` → `HasPosition { pos: Vec2 }` | World position (x, y) |
| **Dimensions** | `NoDimensions` → `HasDimensions { width: f32, height: f32 }` | Cell width/height in world units |
| **Health** | `NoHealth` → `HasHealth { hp: f32 }` | Hit points (current = max at spawn) |
| **Visual** | `Unvisual` → `Rendered { mesh, material }` / `Headless` | Mesh+material for production, headless for tests |

**What's NOT available until each typestate is set:**
- `build()` / `spawn()` require ALL four dimensions resolved — compile error otherwise
- `definition(&def)` sets Health (from def.hp) but NOT Position or Dimensions (those come from grid layout math)
- `.hp()` after `.definition()` overrides the definition's hp

**Runtime config** (optional, available in any state):
- Variant behaviors via `CellBehavior` enum (see below)
- Effects, damage visuals, alias, required_to_clear — all optional with sensible defaults

### CellBehavior enum + effects in RON
Replace the current `CellBehavior` struct (with bool `locked`, `Option<f32>` regen_rate, `Option<ShieldBehavior>`) with a `CellBehavior` enum:
```rust
enum CellBehavior {
    Regen { rate: f32 },
    Shielded(ShieldBehavior),
}
```
Note: `Locked` is NOT a cell behavior — it's a node-layout concern (see "Lock targets" section below).
`ShieldBehavior` is spawn-time config for orbit children: `{ count, radius, speed, hp, color_rgb }`.

RON definition gets:
- `behaviors: Option<Vec<CellBehavior>>` — defaults to None, auto-unwrapped Some so RON writes `behaviors: [Locked, Regen(rate: 2.0)]` not `Some([...])`
- `effects: Option<Vec<RootEffect>>` — same pattern as bolts/walls. Defaults to None, auto-unwrapped Some.
- A cell can have multiple behaviors (e.g., locked + regen)

This replaces the current flat `behavior: (locked: true, regen_rate: Some(2.0))` struct pattern.

### Builder behavior API
- `.with_behavior(CellBehavior)` — add a single behavior
- `.with_behaviors(Vec<CellBehavior>)` — add multiple behaviors at once
- `definition(&def)` populates behaviors from the definition's `behaviors` field
- Individual convenience methods like `.locked()`, `.regen(rate)` are sugar for `.with_behavior(CellBehavior::Locked)` etc.

### Definition + manual + override layering
- `Cell::builder().definition(&def)` — sets hp, color, damage visuals, behaviors from the RON-loaded `CellTypeDefinition`
- Individual setters for things not in the definition: `.position(Vec2)`, `.dimensions(width, height)`, `.scale(f32)`
- Individual setters also work standalone for tests: `.hp(20.0)`, `.color(RED)`, `.locked(true)`
- **Override priority**: specific setter > definition > default. A `.hp(50.0)` call after `.definition(&def)` overrides the definition's hp.

### cells/behaviors/ folder structure
Each behavior gets its own folder with components and systems:
```
cells/
  behaviors/
    locked/
      mod.rs
      components.rs   // Locked, LockAdjacents
      systems/
        check_lock_release/  // existing system, moved here
    regen/
      mod.rs
      components.rs   // CellRegen
      systems/
        tick_cell_regen.rs  // existing system, moved here
    shielded/
      mod.rs
      components.rs   // ShieldParent, OrbitCell, OrbitAngle, OrbitConfig
      systems/
        rotate_shield_cells.rs        // existing, moved
        sync_orbit_cell_positions.rs  // existing, moved
        spawn_orbit_children.rs       // extracted from spawn_cells_from_layout
```

### Lock targets defined in node layout, not cell type
`Locked` means "you must destroy these specific cells to unlock me." The lock targets are NOT grid-adjacent neighbors — they're explicit coordinate pairs defined in the **node layout RON**, not the cell type RON.

**Node layout RON format:**
```ron
(
    grid: [
        ['S','.','.'],
        ['.','S','.'],
        ['.','.','L'],
    ],
    locks: {
        (2,2): [(0,0), (1,1)],  // cell at (2,2) is locked by cells at (0,0) and (1,1)
    },
)
```

- `locks` is `Option<HashMap<(usize, usize), Vec<(usize, usize)>>>` — `#[serde(default)]`, defaults to None. Most node layouts won't have locks so you don't write it at all
- Key = (row, col) of the locked cell, value = list of (row, col) key cells
- At spawn time, `spawn_cells_from_grid` resolves coordinates to entity IDs and populates `LockAdjacents(Vec<Entity>)`
- `CellBehavior::Locked` in the cell type RON is **removed** — a cell is locked because the node layout says so, not because its type says so
- `check_lock_release` system stays the same — still checks if all entities in `LockAdjacents` are despawned

**Shielded cells are different:** `Shielded(ShieldBehavior)` still auto-locks the parent with orbit children as lock targets. The builder/spawn system populates `LockAdjacents` with orbit child entity IDs. This is implicit (orbit children are always the lock keys for a shielded cell), not defined in the node layout.

**Orbit children use Bevy parent-child hierarchy:** Orbit cells are spawned as Bevy children of the shield parent (via `ChildOf`). This gives automatic despawn-on-parent-despawn for free. `GlobalPosition2D` stays truly global (world-space) — `rantzsoft_spatial2d`'s `derive_transform` already counteracts the parent's global transform for child entities so Bevy's `TransformPropagate` produces the correct result.

**No spatial2d changes needed for orbiting.** Orbit children use `PositionPropagation::Relative`. A `rotate_shield_cells` system (already exists) updates each child's `Position2D` each tick based on the orbit angle, radius, and parent position. The propagation system sees `Relative` and computes `global = parent_pos + local_pos` as normal. Simple and doesn't require new propagation variants.

**Quadtree works correctly with children.** `maintain_quadtree` reads `GlobalPosition2D` (not `Position2D`) — tested in `rantzsoft_physics2d/src/systems/maintain_quadtree.rs` (`entity_inserted_at_global_position_not_local`). `compute_globals` correctly computes `GlobalPosition2D` for `ChildOf` entities with `Relative` propagation — tested in `rantzsoft_spatial2d/src/systems/compute_globals/tests/relative_propagation.rs`. Each link is proven individually.

**Missing test: end-to-end orbit child in quadtree.** Add an integration test in `cells/behaviors/shielded/tests/` that spawns a shield parent + orbit child with `ChildOf`, runs `rotate_shield_cells` + `compute_globals` + `maintain_quadtree`, and verifies the orbit child appears at the correct `GlobalPosition2D` in the quadtree after rotation.

**Migration:** Remove `behavior: (locked: true)` from `lock.cell.ron`. Add `locks` section to any node layout RON that uses lock cells.

## Scope
- In: `Cell::builder()...build()/spawn()` builder with typestate for base properties and runtime config for behaviors
- In: `.definition(&def)` convenience that populates from `CellTypeDefinition`
- In: Replace manual tuple assembly in `spawn_cells_from_grid` and test helpers
- In: Handle all cell variants (standard, locked, regen, shielded + orbit children)
- In: Refactor `CellBehavior` from struct to enum, update `CellTypeDefinition` and RON files
- In: Add `effects: Option<Vec<RootEffect>>` to `CellTypeDefinition` (same pattern as bolts/walls)
- In: Update RON cell files for new behavior/effects format
- In: Create `assets/examples/cell.example.ron` documenting all fields
- In: Restructure into `cells/behaviors/` folders, move existing components and systems
- In: Move lock targets from cell type RON (`behavior: (locked: true)`) to node layout RON (`locks: { (r,c): [(r,c), ...] }`)
- In: Update `NodeLayout` definition to include `locks` field
- In: Update `spawn_cells_from_grid` to resolve lock coordinates to entity IDs
- In: Remove `Locked` from `CellBehavior` enum (locking is a node-layout concern, not a cell-type concern)
- In: Remove `build()` from public API of ALL entity builders (Cell, Wall, Bolt, Breaker) — `spawn()` only
- In: Fix shield/second_wind wall spawning to use `spawn()` instead of `build()` + manual spawn
- In: Architecture doc `docs/architecture/builders/cell.md` — cell builder API, typestate dimensions, definition layering (follow pattern of existing bolt/breaker/wall builder docs)
- In: Architecture doc `docs/architecture/cell-behaviors.md` — how to create a new cell behavior (folder structure, components, systems, CellBehavior enum variant, RON format, builder integration)
- Out: Wall builder (separate todo, already done)
- Out: Rendering changes (placeholder rectangles for now)
- Out: `HealthShield` effect implementation and `DamageDealt<T>` pipeline (todo #7)

## Design Files

| File | Contents |
|------|----------|
| [typestate-dimensions.md](typestate-dimensions.md) | Dimension table, transition methods, optional methods, terminal method availability |
| [structs.md](structs.md) | All struct definitions: typestate markers, CellBehavior enum, OptionalCellData, CellBuilder |
| [build-vs-spawn.md](build-vs-spawn.md) | What build() vs spawn() do, why spawn() must be primary, removing build() from public API, migration plan |
| [cell-modifiers.md](cell-modifiers.md) | New cell modifiers (Volatile, Sequence, Survival, Armored, Phantom, Magnetic, Portal), builder API, RON format, behavior folders, graphics needs |

## Dependencies
- Depends on: Bolt builder (done), Breaker builder (done), Wall builder (done) — establishes the pattern
- Blocks: Rendering refactor (builders own visual setup)

## Notes
- Follow the pattern from `bolt/builder/` and `wall/builder/`
- Shielded cells spawn orbit children as separate entities. Extract `spawn_orbit_children` from `spawn_cells_from_layout` into `cells/behaviors/shielded/systems/`.
- `CellTypeAlias` component tracks which definition alias spawned a cell (used by hot-reload). Builder should accept this.
- All cell "types" are reframed as **modifiers**: every cell is a standard cell with HP, modifiers add behavior. A cell can have multiple modifiers. **Locked**, **Regen**, and **Shielded** are modifiers in the `CellBehavior` enum. Locked's key cells are defined in the node layout RON (not the cell type RON), but Locked itself is a modifier on the cell. New modifiers (Volatile, Sequence, Survival, Armored, Phantom, Magnetic, Portal) are designed — see [cell-modifiers.md](cell-modifiers.md).
- Defining and refactoring ALL behaviors (locked, regen, shielded) is in scope — including the folder restructure, component moves, system moves, and the lock-target migration from cell type to node layout.
- **Existing bug**: `effect/effects/shield/system.rs` and `effect/effects/second_wind/system.rs` use `Wall::builder()...build()` instead of `.spawn()`, skipping effect dispatch. These are exclusive World systems so they can't use `Commands` directly — needs a design decision (command flush, refactor to Commands, or `spawn_world()` variant). See [build-vs-spawn.md](build-vs-spawn.md).

## Status
`ready`
