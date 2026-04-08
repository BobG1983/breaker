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
Replace the current `CellBehavior` struct with a `CellBehavior` enum:
```rust
enum CellBehavior {
    Regen { rate: f32 },
}
```
Note: `Locked` is NOT a cell behavior — it's a node-layout concern (see "Lock targets" section below). `Guarded` variant added in Wave 4 when the guard cell design is implemented.

RON definition gets:
- `behaviors: Option<Vec<CellBehavior>>` — defaults to None, auto-unwrapped Some so RON writes `behaviors: [Regen(rate: 2.0)]` not `Some([...])`
- `effects: Option<Vec<RootEffect>>` — same pattern as bolts/walls. Defaults to None, auto-unwrapped Some.
- A cell can have multiple behaviors (e.g., regen + future behaviors)

This replaces the current flat `behavior: (locked: true, regen_rate: Some(2.0))` struct pattern.

### Component marker pattern
Each behavior uses a capability/state/data split:

**Lock:**
- `LockCell` — permanent marker: "this cell has lock capability"
- `Locked` — state marker: currently locked (damage query uses `Without<Locked>`)
- `Locks(Vec<Entity>)` — data: entities that must be destroyed to unlock
- `Unlocked` — state marker: was locked, now unlocked (future use: effects that trigger on unlock)
- System: queries `(With<LockCell>, With<Locks>, With<Locked>, Without<Unlocked>)` → if all Lock entities destroyed → remove `Locked`, add `Unlocked`

**Regen:**
- `RegenCell` — permanent marker: "this cell has regen capability"
- `Regen` — state marker: currently regenerating
- `RegenRate(f32)` — data: HP/sec
- `NoRegen` — state marker: regen disabled (future use: debuffs that stop regen)
- System: queries `(With<RegenCell>, With<Regen>, Without<NoRegen>)` with `CellHealth` → apply rate

CellBehavior enum maps to component bundles at spawn:
- `CellBehavior::Regen { rate: 2.0 }` → inserts `(RegenCell, Regen, RegenRate(2.0))`
- Lock (from NodeLayout) → inserts `(LockCell, Locked, Locks(entities))`

### Builder behavior API
- `.with_behavior(CellBehavior)` — add a single behavior (inserts capability + state + data components)
- `.with_behaviors(Vec<CellBehavior>)` — add multiple behaviors at once
- `.locked(Vec<Entity>)` — inserts `(LockCell, Locked, Locks(entities))`. Takes resolved entity IDs, not grid coordinates. The spawn pipeline resolves coordinates to entities, then passes them here.
- `definition(&def)` populates behaviors from the definition's `behaviors` field
- `.regen(rate)` — sugar for `.with_behavior(CellBehavior::Regen { rate })`

### Definition + manual + override layering
- `Cell::builder().definition(&def)` — sets hp, color, damage visuals, behaviors from the RON-loaded `CellTypeDefinition`
- Individual setters for things not in the definition: `.position(Vec2)`, `.dimensions(width, height)`, `.scale(f32)`
- Individual setters also work standalone for tests: `.hp(20.0)`, `.color(RED)`, `.locked(vec![entity])`
- **Override priority**: specific setter > definition > default. A `.hp(50.0)` call after `.definition(&def)` overrides the definition's hp.

### cells/behaviors/ folder structure
Each behavior is a fully self-contained package — ALL components and systems live under `cells/behaviors/<name>/`. Nothing behavior-specific lives in `cells/components/types.rs`. Cross-domain components are globbed into `prelude/components.rs` (e.g., `Locked` for damage query filters). Same pattern as effects.
```
cells/
  behaviors/
    locked/
      mod.rs
      components.rs   // LockCell, Locked, Locks(Vec<Entity>), Unlocked
      systems/
        check_lock_release/  // existing system, refactored for new components
    regen/
      mod.rs
      components.rs   // RegenCell, Regen, RegenRate(f32), NoRegen
      systems/
        tick_cell_regen.rs  // existing system, refactored for new components
    guarded/           // Wave 4 — guard cell redesign
      mod.rs
      components.rs   // GuardedCell, GuardianCell, GuardianSlot(usize)
      systems/
        slide_guardian_cells.rs
```

### Lock targets defined in node layout, not cell type
`Locked` means "you must destroy these specific cells to unlock me." The lock targets are NOT grid-adjacent neighbors — they're explicit coordinate pairs defined in the **node layout RON**, not the cell type RON.

**Node layout RON format:**
```ron
(
    grid: [
        ["S",".","."],
        [".","S","."],
        [".",".","L"],
    ],
    locks: {
        (2,2): [(0,0), (1,1)],  // cell at (2,2) is locked by cells at (0,0) and (1,1)
    },
)
```

**Lock chains**: Locks can chain — a locked cell can be a lock target for another locked cell (e.g., Cell A locked by Cell B, Cell B locked by Cell C). The spawn pipeline must:
1. Spawn all non-locked cells first, collecting `HashMap<(usize,usize), Entity>`
2. Topological sort the `locks` entries by dependency order (cells whose lock targets are all non-locked spawn first)
3. Spawn locked cells in dependency order, calling `.locked(resolved_entities)` on each
4. **Circular lock handling**: detect circular dependencies during spawn. Log a debug warning ("ignoring lock X — would create circular lock") and skip the circular connection. Don't panic, don't reject the layout — just don't wire the final connection that would create the cycle.

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
- In: `Cell::builder()...spawn()` builder with typestate for base properties and runtime config for behaviors
- In: `.definition(&def)` convenience that populates from `CellTypeDefinition`
- In: Replace manual tuple assembly in `spawn_cells_from_grid` and test helpers
- In: Handle existing cell variants (standard, locked, regen) + redesign guard cells (replaces shielded/orbit)
- In: Refactor `CellBehavior` from struct to enum (Regen initially, Guarded added in final wave)
- In: Multi-char grid aliases (`alias: char` → `alias: String`, `grid: Vec<Vec<char>>` → `Vec<Vec<String>>`) — needed for `Gu`/`gu` guard cell aliases
- In: Guard cell redesign: 3x3 grid-based model replacing orbit model. Guarded Cell (parent, damageable, NOT locked) + Guardian Cells (square children, slide between ring positions, ChildOf parent). Builder API: `.guarded(vec![(-1,0),(1,0)])` with relative grid offsets. Layout controls guardian count via `gu` positions in the 3x3 ring around `Gu`.
- In: Add `effects: Option<Vec<RootEffect>>` to `CellTypeDefinition` (same pattern as bolts/walls)
- In: Update RON cell files for new behavior/effects format
- In: Create `assets/examples/cell.example.ron` documenting all fields
- In: Restructure into `cells/behaviors/` folders, move existing components and systems
- In: Move lock targets from cell type RON (`behavior: (locked: true)`) to node layout RON (`locks: { (r,c): [(r,c), ...] }`)
- In: Update `NodeLayout` definition to include `locks` field
- In: Update `spawn_cells_from_grid` to resolve lock coordinates to entity IDs
- In: Remove `build()` entirely from ALL entity builders (Cell, Wall, Bolt, Breaker) — make private, `spawn()` is the only terminal
- In: Fix shield/second_wind wall spawning to use `spawn()` instead of `build()` + manual spawn
- In: Architecture doc `docs/architecture/builders/cell.md` — cell builder API, typestate dimensions, definition layering
- In: Architecture doc `docs/architecture/cell-behaviors.md` — how to create a new cell behavior (folder structure, components, systems, CellBehavior enum variant, RON format, builder integration)
- Out: Toughness enum + HP scaling formula (separate todo)
- Out: New cell modifiers — volatile, sequence, survival, armored, phantom, magnetic, portal (separate todo)
- Out: Wall builder (already done)
- Out: Rendering changes (placeholder rectangles for now)
- Out: `HealthShield` effect implementation and `DamageDealt<T>` pipeline

## Design Files

| File | Contents |
|------|----------|
| [typestate-dimensions.md](typestate-dimensions.md) | Dimension table, transition methods, optional methods, terminal method availability |
| [structs.md](structs.md) | All struct definitions: typestate markers, CellBehavior enum, OptionalCellData, CellBuilder |
| [build-vs-spawn.md](build-vs-spawn.md) | What build() vs spawn() do, why spawn() must be primary, removing build() from public API, migration plan |
| [cell-modifiers.md](cell-modifiers.md) | New cell modifiers (Volatile, Sequence, Survival, Armored, Phantom, Magnetic, Portal), builder API, RON format, behavior folders, graphics needs |

## Dependencies
- Depends on: Bolt builder (done), Breaker builder (done), Wall builder (done) — establishes the pattern
- Blocks: Toughness + HP scaling todo, New cell modifiers todo, Rendering refactor

## Notes
- Follow the pattern from `bolt/builder/` and `wall/builder/`
- `CellTypeAlias` component tracks which definition alias spawned a cell (used by hot-reload). Builder should accept this. Changes from `CellTypeAlias(char)` to `CellTypeAlias(String)` with the multi-char alias migration.
- Locked and Regen are refactored as part of this todo (folder restructure, component moves, system moves, lock-target migration). New modifiers (Volatile → Portal) are a separate todo — see [cell-modifiers.md](cell-modifiers.md) for their designs.
- Shielded/orbit model is REPLACED by the new Guard Cell model (3x3 grid, sliding guardians). Old components (`ShieldParent`, `OrbitCell`, `OrbitAngle`, `OrbitConfig`) and old systems (`rotate_shield_cells`, `sync_orbit_cell_positions`) are removed.
- **Vocabulary**: Guarded Cell = parent, Guardian Cell = child. Replaces Shield/Orbit terminology.
- **Guardian dimensions**: square (`cell_height × cell_height`), centered in grid slot. Gap on each side = `(cell_width - cell_height) / 2`. Bolts can squeeze past sides.
- **Guard cell design** happens in parallel with early implementation waves, implemented as final wave.
- **Existing bug**: `effect/effects/shield/system.rs` and `effect/effects/second_wind/system.rs` use `Wall::builder()...build()` instead of `.spawn()`, skipping effect dispatch. These are exclusive World systems so they can't use `Commands` directly — needs a design decision (command flush, refactor to Commands, or `spawn_world()` variant). See [build-vs-spawn.md](build-vs-spawn.md).
- Builder uses `.hp(value)` directly — no toughness enum yet (separate todo).

## Status
`ready`
