# Hazard: Cascade

## Game Design

Destroyed cell heals adjacent cells. +10 HP +5 HP/level. When a cell is destroyed, every adjacent (orthogonal) cell receives a heal. This creates a "whack-a-mole" dynamic where killing cells makes their neighbors tougher. The player must plan kill order — working from the edges inward to minimize the number of neighbors that benefit. At high stacks, destroying a cell in the middle of a cluster heals surrounding cells for massive amounts, potentially undoing recent damage.

## Config Resource

```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct CascadeConfig {
    /// Base HP healed to each adjacent cell at stack 1.
    pub base_heal: f32,
    /// Additional HP healed per stack beyond the first.
    pub heal_per_level: f32,
}
```

Extracted from `HazardTuning::Cascade { base_heal, heal_per_level }` at activation time.

## Components

None. Cascade is a reactive system — it responds to cell death events and sends heal messages. No persistent per-entity state needed.

## Messages

**Reads**: Cell death message — a notification that a cell was destroyed, including its grid position or entity ID so adjacency can be determined. (The exact message depends on the effect refactor — todo #2. Currently `DamageDealt<Cell>` is a placeholder; the actual name may be `CellDestroyedAt { position: IVec2 }` or similar.)
**Sends**: `HealCell { cell: Entity, amount: f32 }` — new message, owned by `cells` domain. One message per adjacent cell per death event.

## Systems

### `cascade_heal_on_death`

- **Schedule**: `FixedUpdate`
- **Run condition**: `hazard_active(HazardKind::Cascade).and(in_state(NodeState::Playing))`
- **Ordering**: After cell death processing. Must run after the system that emits cell destroyed messages but before the next physics step.
- **Behavior**: For each cell death event:
  1. Determine the grid position of the destroyed cell.
  2. Find all living adjacent cells (orthogonal neighbors: up, down, left, right).
  3. Compute heal amount: `base_heal + heal_per_level * (stack - 1)`.
  4. Send `HealCell { cell, amount }` for each adjacent living cell.
- **Adjacency query**: Reads the cell grid/spatial structure to find neighbors. This may require a query into the cells domain's grid data structure or a component-based neighbor lookup.

## Stacking Behavior

Linear scaling: `heal_amount = base_heal + heal_per_level * (stack - 1)`

| Stack | Heal per adjacent cell | Notes |
|-------|----------------------|-------|
| 1 | 10 HP | Noticeable — neighbors gain durability |
| 2 | 15 HP | Significant — center cells become resilient |
| 3 | 20 HP | Major — destroying a cell in a cluster backfires hard |

At stack 5, each death heals adjacents for 30 HP. A cell in the middle of a 3x3 cluster with 4 neighbors heals the cluster for 120 HP total when destroyed.

## Cross-Domain Dependencies

| Domain | Direction | Message |
|--------|-----------|---------|
| `cells` | reads from | Cell death notification (cell destroyed at position) |
| `cells` | sends to | `HealCell` — heals adjacent cells |

Cascade needs to query cell adjacency. This requires either:
1. Reading a grid resource from the cells domain (cross-domain read — allowed), or
2. The cell death message includes enough positional information for the hazard to determine neighbors.

The cells domain owns cell HP and applies the heal.

## Expected Behaviors (for test specs)

1. **Adjacent cells healed on death at stack 1**
   - Given: Cascade active at stack 1, `base_heal=10.0`, `heal_per_level=5.0`, cell at grid (2,2) destroyed, living cells at (1,2), (3,2), (2,1), (2,3)
   - When: `cascade_heal_on_death` processes the death
   - Then: 4x `HealCell { cell, amount: 10.0 }` sent, one per neighbor

2. **Adjacent cells healed more at stack 3**
   - Given: Cascade active at stack 3, same config, cell at (2,2) destroyed, 4 living neighbors
   - When: `cascade_heal_on_death` processes the death
   - Then: 4x `HealCell { cell, amount: 20.0 }` sent (10.0 + 5.0 * 2)

3. **Corner cell has fewer neighbors**
   - Given: Cascade active at stack 1, cell at grid (0,0) destroyed, living cells at (1,0) and (0,1) only
   - When: `cascade_heal_on_death` processes the death
   - Then: 2x `HealCell` sent (only existing neighbors)

4. **Dead neighbors are not healed**
   - Given: Cascade active, cell destroyed, one adjacent position is empty (already destroyed)
   - When: `cascade_heal_on_death` runs
   - Then: No `HealCell` sent for the empty position

5. **Multiple deaths in same frame each trigger cascade**
   - Given: 2 cells destroyed in the same frame, sharing a common neighbor
   - When: `cascade_heal_on_death` processes both
   - Then: The common neighbor receives 2 separate `HealCell` messages (one from each death)

## Edge Cases

- **Cascade + Tether synergy**: Tether spreads non-lethal damage to linked partners. That damage doesn't kill but weakens. When the weakened cell finally dies, Cascade heals its neighbors — including possibly the Tether partner that just took spread damage. The heal undoes the Tether's work, creating a frustrating loop for the player.
- **Cascade + Fracture interaction**: Fracture spawns new cells when one dies. If Fracture spawns a cell adjacent to the death position, does Cascade heal it? Depends on system ordering — if Fracture spawns run after Cascade, the new cells won't be healed this frame. Ordering should be: Cascade heals first, then Fracture spawns.
- **Cascade does NOT cascade**: The name is about healing adjacent cells, not about chain reactions. Healing a cell to full HP does not trigger another Cascade event. Only cell DEATH triggers Cascade.
- **Ghost cells (Echo Cells)**: Ghost cell deaths do trigger Cascade — they are destroyed cells. Their neighbors get healed. This makes Echo Cells + Cascade a mild synergy: cleaning up ghosts feeds heals to surviving cells.
- **Node-end cleanup**: `CascadeConfig` resource removed at run end. No per-entity state to clean up.
- **HP over-healing**: If Cascade heals a cell beyond its starting HP, behavior depends on the cells domain. Typically, HP is capped at starting HP (or some maximum). Cascade doesn't need to enforce this — the cells domain handles the cap when processing `HealCell`.
