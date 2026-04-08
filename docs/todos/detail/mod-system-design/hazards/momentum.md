# Hazard: Momentum

## Game Design

Non-lethal hits give the cell HP. When a cell reaches 2x its starting HP, it splits into 2 cells at 1x starting HP each, placed in adjacent empty cells. This punishes chip effects that deal repeated small damage (e.g., damage-over-time, low-damage multi-hits) and rewards one-shot kills. The player must build for burst damage or accept that every non-lethal hit feeds the problem.

**Stacking formula**: `+10 HP + 10 HP * (stack - 1)` per non-lethal hit. At stack=1, each non-lethal hit adds 10 HP. At stack=3, each adds 30 HP.

## Config Resource

```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct MomentumConfig {
    pub base_hp_per_hit: f32,           // 10.0
    pub hp_per_level: f32,              // 10.0 per additional stack
    pub split_threshold_multiplier: f32, // 2.0 (split at 2x starting HP)
}
```

## Components

```rust
/// Records the cell's HP at spawn time. Used to compute the 2x split threshold.
/// Shared with Volatility hazard -- if already exists, reuse.
#[derive(Component, Debug)]
pub(crate) struct CellStartingHp(pub f32);
```

No additional per-cell component needed. Momentum reads `DamageDealt<Cell>` (non-lethal) reactively and responds with `HealCell` + potential split. The cell's current HP is queried directly.

## Messages

**Reads**: `DamageDealt<Cell>` (filters for non-lethal -- cell survived the hit)
**Sends**: `HealCell { cell: Entity, amount: f32 }`, cell spawn message for splits (e.g., `SpawnCell` or direct cell spawning within the cells domain)

## Systems

1. **`attach_starting_hp`**
   - Schedule: After cell spawn (per node start)
   - Run if: `hazard_active(HazardKind::Momentum)` AND cells exist without `CellStartingHp`
   - Behavior: Insert `CellStartingHp(current_hp)` on all cell entities
   - Note: Shared with Volatility. If Volatility already attaches this, Momentum reuses it. A shared "hazard cell setup" system could handle this.

2. **`momentum_heal_on_nonlethal`**
   - Schedule: `FixedUpdate`
   - Run if: `hazard_active(HazardKind::Momentum)` AND `in_state(NodeState::Playing)`
   - Ordering: After `apply_damage` (need to know which cells survived)
   - Behavior:
     1. Read `DamageDealt<Cell>` entries
     2. For each message where the target cell is still alive (survived the hit):
        a. Compute heal amount: `base_hp_per_hit + hp_per_level * (stack - 1)`
        b. Send `HealCell { cell, amount }`

3. **`momentum_split_check`**
   - Schedule: `FixedUpdate`
   - Run if: `hazard_active(HazardKind::Momentum)` AND `in_state(NodeState::Playing)`
   - Ordering: After `momentum_heal_on_nonlethal` and after `HealCell` is processed
   - Behavior:
     1. For each cell with `CellStartingHp`, check if current HP >= `starting_hp * split_threshold_multiplier`
     2. If threshold reached:
        a. Find up to 2 adjacent empty grid positions
        b. Spawn new cells at those positions, each with `starting_hp` HP (1x, not the inflated amount)
        c. Set the original cell's HP back to `starting_hp` (it "splits" -- doesn't keep the excess)
        d. New cells get their own `CellStartingHp` (set to `starting_hp` of the parent)
     3. If fewer than 2 empty adjacent positions exist, spawn as many as possible (1 or 0)

## Stacking Behavior

| Stack | HP per non-lethal hit | Hits to split (base 10 HP cell) |
|-------|----------------------|--------------------------------|
| 1     | 10                   | 1 hit (10 HP cell gains 10, reaches 20 = 2x) |
| 2     | 20                   | 1 hit (immediately at or past threshold) |
| 3     | 30                   | 1 hit (well past threshold) |

**Design note**: At stack 1, a 10 HP cell that takes non-lethal damage gains 10 HP, reaching 20 HP (2x threshold), and immediately splits. This means ANY non-lethal hit on a low-HP cell triggers a split. For higher HP cells (e.g., 50 HP), it takes `ceil(50 / 10) = 5` non-lethal hits to split at stack 1.

The key dynamic: low-HP cells split easily, high-HP cells resist. Stacking makes even high-HP cells split quickly.

## Cross-Domain Dependencies

| Domain | Interaction | Message |
|--------|------------|---------|
| `cells` | Reads cell HP and alive status | Direct query |
| `cells` | Heals cells | `HealCell` message (send) |
| `cells` | Spawns new cells on split | Cell spawn mechanism (message or direct, depends on cells domain API) |
| `cells` | Reads damage events | `DamageDealt<Cell>` (read) |

**Cell spawning**: Split cells are basic cells (no special rules, no locks, no special cell types). They inherit the parent's starting HP but not any special behaviors. The cells domain must provide a way to spawn a basic cell at a grid position mid-node.

## Expected Behaviors (for test specs)

1. **Non-lethal hit heals cell at stack=1**
   - Given: Cell with 10 HP (starting HP 10), bolt deals 5 damage (cell survives at 5 HP), stack=1
   - When: `momentum_heal_on_nonlethal` runs
   - Then: `HealCell { cell, amount: 10.0 }` sent. Cell goes from 5 HP to 15 HP.

2. **Cell splits at 2x starting HP**
   - Given: Cell with 20 HP (starting HP 10), 2 empty adjacent positions
   - When: `momentum_split_check` runs
   - Then: Original cell HP set to 10. Two new cells spawned at adjacent positions, each with 10 HP and `CellStartingHp(10.0)`.

3. **Lethal hit does NOT trigger heal**
   - Given: Cell with 10 HP, bolt deals 15 damage (cell dies)
   - When: `momentum_heal_on_nonlethal` runs
   - Then: No `HealCell` sent (cell is dead)

4. **HP per hit scales with stack at stack=3**
   - Given: Cell with 50 HP (starting HP 50), bolt deals 10 damage (survives at 40 HP), stack=3
   - When: `momentum_heal_on_nonlethal` runs
   - Then: `HealCell { cell, amount: 30.0 }` sent. Cell goes from 40 to 70 HP.

5. **Split with limited empty positions**
   - Given: Cell at 2x HP, only 1 empty adjacent position
   - When: `momentum_split_check` runs
   - Then: Only 1 new cell spawned. Original HP reset to starting HP.

6. **Split with zero empty positions**
   - Given: Cell at 2x HP, no empty adjacent positions (surrounded)
   - When: `momentum_split_check` runs
   - Then: No split occurs. Cell retains its current HP (does not reset). Will re-check next frame.

## Edge Cases

- **Momentum + Diffusion synergy**: Diffusion bleeds damage to neighbors, preventing one-shot kills. Non-lethal hits then feed Momentum's HP growth + split. This creates a feedback loop where trying to kill one cell strengthens its neighbors.
- **Momentum + Fracture synergy**: Fracture creates split debris in empty cells. Momentum splits also create cells in empty cells. If both target the same empty positions, Fracture runs first (on cell death) and Momentum runs after (on survival). No conflict -- they use different triggers.
- **Split cell inherits hazard components**: New cells from splits need `CellStartingHp` (and `VolatilityTimer` if Volatility is active). The spawn system must ensure hazard components are attached to dynamically spawned cells.
- **Cascade chain**: If a split cell is immediately adjacent to a cell that then dies, Cascade heals it. Combined with Momentum's HP growth, cells become very hard to kill. This is the intended trap synergy.
- **Overflow prevention**: If a cell accumulates massive HP (e.g., 100x starting), the split mechanic still only resets to 1x. The system is self-regulating -- splits produce more cells but each at base HP.
- **Cleanup**: `CellStartingHp` is on cell entities -- cleaned up on despawn. `MomentumConfig` removed at run end.
