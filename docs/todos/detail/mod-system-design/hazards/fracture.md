# Hazard: Fracture

## Game Design

Destroyed cells split into adjacent empty cells. 2+1/level new cells spawned per death. When a cell is destroyed, low-HP debris cells spawn in nearby empty positions. This creates a "clear one, spawn more" dynamic — the board never truly gets cleaner. The player must manage debris efficiently, ideally using AoE or chain effects. At high stacks, a single kill spawns a ring of debris.

## Config Resource

```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct FractureConfig {
    /// Base number of debris cells spawned at stack 1.
    pub base_splits: u32,
    /// Additional debris cells per stack beyond the first.
    pub splits_per_level: u32,
}
```

Extracted from `HazardTuning::Fracture { base_splits, splits_per_level }` at activation time.

## Components

### `FractureDebris` (marker component)

```rust
/// Marks a cell as debris spawned by the Fracture hazard.
/// Debris cells are basic cells with low HP and no special behaviors.
#[derive(Component, Debug, Default)]
pub(crate) struct FractureDebris;
```

Attached to spawned debris cells so other systems can identify them (e.g., Fracture itself may choose not to spawn debris when debris is destroyed, or may allow recursive splits depending on design intent).

## Messages

**Reads**: Cell death message — notification that a cell was destroyed, including grid position. (The exact message depends on the effect refactor — todo #2. `DamageDealt<Cell>` is a placeholder.)
**Sends**: A cell spawn message to create debris in empty adjacent positions. This could be `SpawnGhostCell { position: Vec2, hp: f32 }` (reusing the Echo Cells message with a different marker) or a more general `SpawnDebrisCell { position: Vec2, hp: f32 }` message owned by the cells domain.

For consistency, use `SpawnGhostCell` if ghost and debris cells share the same spawn pathway (both are simple, low-HP, no-special-behavior cells). If they need distinct behaviors, a separate message is warranted. The cells domain decides — Fracture just requests spawns.

## Systems

### `fracture_on_death`

- **Schedule**: `FixedUpdate`
- **Run condition**: `hazard_active(HazardKind::Fracture).and(in_state(NodeState::Playing))`
- **Ordering**: After cell death processing, after `cascade_heal_on_death` (Cascade heals first, then Fracture spawns — so Cascade doesn't accidentally heal brand-new debris)
- **Behavior**: For each cell death event:
  1. Determine the grid position of the destroyed cell.
  2. Find all empty adjacent positions (orthogonal neighbors: up, down, left, right, and optionally diagonal — design decision TBD, orthogonal assumed).
  3. Compute spawn count: `min(base_splits + splits_per_level * (stack - 1), empty_positions.len())`.
  4. Select positions to fill (if more empty positions than spawn count, choose randomly from seeded RNG).
  5. Send spawn message for each debris cell with 1 HP (debris is always 1 HP regardless of stack — stack increases count, not HP).

### Debris HP

Debris cells always spawn with 1 HP. The design says "2+1/level" refers to the COUNT of splits, not the HP. Debris is meant to be easy to clear individually but overwhelming in quantity.

## Stacking Behavior

Linear count scaling: `split_count = base_splits + splits_per_level * (stack - 1)`

| Stack | Debris spawned | Notes |
|-------|---------------|-------|
| 1 | 2 | Two 1-HP debris cells per kill |
| 2 | 3 | Three debris per kill |
| 3 | 4 | Four debris per kill — fills all orthogonal neighbors |

At stack 4+, the spawn count exceeds the 4 orthogonal neighbors. Options:
- Cap at available empty positions (natural limit of grid)
- Include diagonal positions as overflow targets
- This is a design decision to resolve during implementation

## Cross-Domain Dependencies

| Domain | Direction | Message |
|--------|-----------|---------|
| `cells` | reads from | Cell death notification (cell destroyed at position) |
| `cells` | sends to | Spawn message — creates debris cells in empty positions |

Fracture needs to query the cell grid for empty adjacent positions. This requires reading the cells domain's grid data structure (cross-domain read — allowed).

## Expected Behaviors (for test specs)

1. **Two debris cells spawn at stack 1**
   - Given: Fracture active at stack 1, `base_splits=2`, `splits_per_level=1`, cell at (2,2) destroyed, empty positions at (1,2), (3,2), (2,1), (2,3)
   - When: `fracture_on_death` processes the death
   - Then: 2 spawn messages sent for 2 of the 4 empty positions, each with `hp: 1.0`

2. **Four debris cells spawn at stack 3**
   - Given: Fracture active at stack 3, same config, cell at (2,2) destroyed, 4 empty neighbors
   - When: `fracture_on_death` runs
   - Then: 4 spawn messages sent (all neighbors filled), each with `hp: 1.0`

3. **Spawn count capped by available empty positions**
   - Given: Fracture active at stack 3 (wants to spawn 4), cell at (2,2) destroyed, only 2 empty neighbors (others occupied)
   - When: `fracture_on_death` runs
   - Then: 2 spawn messages sent (only available positions)

4. **No spawns if no empty neighbors**
   - Given: Fracture active, cell destroyed, all neighbor positions occupied
   - When: `fracture_on_death` runs
   - Then: 0 spawn messages sent

5. **Multiple deaths in same frame each trigger fracture independently**
   - Given: 2 cells destroyed in same frame
   - When: `fracture_on_death` processes both
   - Then: Each death spawns its own debris (potentially overlapping positions resolved by cells domain)

## Edge Cases

- **Fracture + Momentum synergy**: Fracture spawns 1-HP debris. Momentum adds HP on non-lethal hits. If the player can't one-shot debris, Momentum adds HP, and at 2x starting HP the cell splits again (Momentum's split mechanic). This creates a chain reaction: kill a cell -> spawn debris -> hit debris non-lethally -> debris grows -> debris splits -> more debris. Terrifying at high stacks.
- **Fracture + Volatility synergy**: Debris spawns at 1 HP. Volatility grows cells not being hit. Uncleared debris becomes 2-3 HP debris over time, turning trivial cleanup into real work.
- **Recursive fracture**: Should destroying a `FractureDebris` cell trigger another Fracture event? Design intent leans yes (the mechanic description says "destroyed cells" without exception). This means debris can spawn more debris, creating chain reactions. If this proves too punishing, the `FractureDebris` marker can be used to filter out recursive splits.
- **Node-end cleanup**: `FractureConfig` resource removed at run end. Debris cells are regular entities and despawn with the node.
- **Position conflicts with Echo Cells**: Both Fracture and Echo Cells want to place cells in positions. System ordering (Cascade -> Fracture -> Echo Cells timer) prevents same-frame conflicts. Across frames, the cells domain handles occupied-position conflicts.
- **Seeded RNG for position selection**: When spawn count < available positions, position selection uses seeded `GameRng` for deterministic replay.
