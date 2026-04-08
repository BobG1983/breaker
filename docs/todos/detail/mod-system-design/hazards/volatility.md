# Hazard: Volatility

## Game Design

Cells gain HP when not being hit. HP caps at 2x starting HP. This is a "neglect tax" -- the player must keep touching cells to suppress growth. If you focus on one area, unattended cells silently grow tougher.

**Stacking formula**: Base growth is +1 HP per 5 seconds of not being hit. Stacking reduces the interval (cells grow faster). The interval diminishes per level with a floor to prevent instant growth.

**Design note**: "Per level" means per level after the first. Stack 1 = base amount.

## Config Resource

```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct VolatilityConfig {
    pub hp_per_interval: f32, // 1.0 HP gained per tick
    pub interval_secs: f32,   // 5.0 seconds between growth ticks
    pub max_multiplier: f32,  // 2.0 (cap at 2x starting HP)
}
```

**Stacking effect on interval**: The interval shrinks per stack. Formula: `effective_interval = interval_secs / (1.0 + 0.25 * (stack - 1))`. This is diminishing returns -- each stack adds less shrinkage.

| Stack | Effective interval |
|-------|--------------------|
| 1     | 5.0s               |
| 2     | 4.0s               |
| 3     | 3.33s              |
| 5     | 2.5s               |

Floor: interval never drops below 1.0s.

## Components

```rust
/// Tracks time since this cell was last hit (damaged).
/// Resets to 0.0 on any incoming damage.
#[derive(Component, Debug)]
pub(crate) struct VolatilityTimer {
    pub elapsed: f32,
}

/// Records the cell's HP at spawn time, used to compute the 2x cap.
#[derive(Component, Debug)]
pub(crate) struct CellStartingHp(pub f32);
```

`VolatilityTimer` is added to all cell entities when the hazard is active. `CellStartingHp` may already exist in the cells domain (needed for Momentum too) -- if so, reuse it rather than duplicating.

## Messages

**Reads**: `DamageDealt<Cell>` (to detect when a cell was hit and reset its timer)
**Sends**: `HealCell { cell: Entity, amount: f32 }`

## Systems

1. **`attach_volatility_timers`**
   - Schedule: Runs when cells are spawned (after cell builder, per node start)
   - Run if: `hazard_active(HazardKind::Volatility)` AND cells exist without `VolatilityTimer`
   - Behavior: Insert `VolatilityTimer { elapsed: 0.0 }` and `CellStartingHp(current_hp)` on all cell entities

2. **`reset_volatility_on_damage`**
   - Schedule: `FixedUpdate`
   - Run if: `hazard_active(HazardKind::Volatility)` AND `in_state(NodeState::Playing)`
   - Ordering: After `apply_damage`
   - Behavior:
     1. Read `DamageDealt<Cell>` entries
     2. For each damaged cell entity, reset its `VolatilityTimer.elapsed` to 0.0

3. **`volatility_grow_cells`**
   - Schedule: `FixedUpdate`
   - Run if: `hazard_active(HazardKind::Volatility)` AND `in_state(NodeState::Playing)`
   - Ordering: After `reset_volatility_on_damage`
   - Behavior:
     1. Compute `effective_interval` from config + stack count
     2. For each cell with `VolatilityTimer`:
        a. Advance `elapsed` by `delta_secs`
        b. While `elapsed >= effective_interval`:
           - Check if cell HP < `starting_hp * max_multiplier`
           - If under cap: send `HealCell { cell, amount: hp_per_interval }`
           - Subtract `effective_interval` from `elapsed`

## Stacking Behavior

| Stack | Interval | HP/interval | Growth rate (HP/s) | Time to 2x (from base 10 HP) |
|-------|----------|-------------|-------------------|------------------------------|
| 1     | 5.0s     | 1.0         | 0.2               | 50s                          |
| 2     | 4.0s     | 1.0         | 0.25              | 40s                          |
| 3     | 3.33s    | 1.0         | 0.3               | 33.3s                        |

The growth rate is modest -- the threat is cumulative. In a node with 30+ cells, even 0.2 HP/s per neglected cell adds up fast.

## Cross-Domain Dependencies

| Domain | Interaction | Message |
|--------|------------|---------|
| `cells` | Reads cell HP to check cap | Direct query (or via `CellStartingHp` component) |
| `cells` | Heals cells | `HealCell` message |
| `cells` | Reads damage events to reset timer | `DamageDealt<Cell>` (read only) |

## Expected Behaviors (for test specs)

1. **Cell gains HP after not being hit at stack=1**
   - Given: Cell with 10 HP, starting HP 10, `VolatilityTimer` at 0.0, stack=1
   - When: 5.0 seconds pass with no damage
   - Then: `HealCell { amount: 1.0 }` sent, cell now 11 HP

2. **Timer resets on damage**
   - Given: Cell with `VolatilityTimer.elapsed = 4.5` (0.5s from next tick)
   - When: Cell takes damage
   - Then: `VolatilityTimer.elapsed` resets to 0.0, next growth tick is 5.0s away

3. **HP caps at 2x starting HP**
   - Given: Cell with starting HP 10, current HP 19, stack=1
   - When: Volatility tick fires
   - Then: `HealCell { amount: 1.0 }` sent (cell goes to 20). Next tick: no `HealCell` sent (already at cap)

4. **Growth rate increases with stacking at stack=3**
   - Given: Cell with 10 HP, starting HP 10, stack=3 (interval=3.33s)
   - When: 3.33 seconds pass with no damage
   - Then: `HealCell { amount: 1.0 }` sent

5. **System does not run when hazard is inactive**
   - Given: Volatility not in `ActiveHazards`
   - When: time passes
   - Then: No `VolatilityTimer` components exist, no `HealCell` sent

## Edge Cases

- **Echo Cells + Volatility synergy**: Ghost cells (1 HP) grow rapidly if not cleared immediately. At stack=1, a ghost reaches 2 HP in 5 seconds. This is the intended trap -- ghosts look free but become real threats.
- **Fracture + Volatility synergy**: Split debris (1 HP) also grows if neglected. Combined with Fracture creating many small cells, Volatility turns "easy cleanup" into a race.
- **Cell spawned mid-node**: If a cell is created mid-node (by Fracture, Momentum split, or Echo Cells), it needs a `VolatilityTimer` and `CellStartingHp`. The `attach_volatility_timers` system should run as a reactive system (detect cells without the component) or the spawning system should add the components.
- **Damage of 0**: A 0-damage hit (from a chip or effect that applies 0 damage) should still reset the timer -- the cell was "touched."
- **Multiple heals per frame**: If `elapsed` accumulates past multiple intervals (e.g., game was paused then unpaused), the while loop fires multiple `HealCell` messages in one frame. This is correct.
- **Cleanup**: `VolatilityTimer` and `CellStartingHp` are on cell entities -- cleaned up when cells despawn at node end. `VolatilityConfig` removed at run end.
