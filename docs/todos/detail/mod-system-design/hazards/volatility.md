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
```

`VolatilityTimer` is added to all cell entities when the hazard is active. The cell's pristine HP is read from `Hp.starting` (unified death pipeline — no separate component).

**Dynamic `Hp.max` lift**: When Volatility activates (or when a new cell spawns while Volatility is active), `attach_volatility_timers` also sets `Hp.max = Some(hp.starting * max_multiplier)` on each cell that does not already have a higher max. This allows `HealCap::Max` in the generic pipeline to permit growth up to `2× starting` without needing a bespoke cap variant. When Volatility is cleaned up at run-end, cells are despawned with the node, so no `Hp.max` restoration is needed.

## Messages

**Reads**: `DamageDealt<Cell>` (to detect when a cell was hit and reset its timer)
**Sends**: `HealDealt<Cell>` — generic heal pipeline message in `shared::death_pipeline`. Cap variant: `HealCap::Max` with `amount = hp_per_interval`. The 2× cap is expressed through the cell's `Hp.max` (lifted to `hp.starting * max_multiplier` at `attach_volatility_timers` time); `HealCap::Max` then clamps arriving heals at that value. Volatility also performs its OWN pre-send gate — it only emits a heal if `hp.current < hp.starting * max_multiplier` — so the cap is enforced twice: once at send-time (cheap), once by the pipeline (authoritative).

## Systems

1. **`attach_volatility_timers`**
   - Schedule: Runs when cells are spawned (after cell builder, per node start)
   - Run if: `hazard_active(HazardKind::Volatility)` AND cells exist without `VolatilityTimer`
   - Behavior:
     1. Insert `VolatilityTimer { elapsed: 0.0 }` on all cell entities missing it.
     2. Lift `Hp.max` to `max(existing_max, hp.starting * max_multiplier)` on each cell — i.e., take the HIGHER of the cell's current `Hp.max` (if any) and `2× starting`. Formula:
        ```rust
        let target = hp.starting * config.max_multiplier;
        hp.max = Some(hp.max.map_or(target, |m| m.max(target)));
        ```
        This never lowers an existing cap (future buffs with higher max are preserved) and always ensures the ceiling is AT LEAST `2× starting` so Volatility's 2× growth path is reachable through `HealCap::Max` in the pipeline.

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
     2. For each cell with `VolatilityTimer` + `Hp`:
        a. Advance `elapsed` by `delta_secs`
        b. While `elapsed >= effective_interval`:
           - Check if `hp.current < hp.starting * max_multiplier`
           - If under cap: send `HealDealt::<Cell> { healer: None, target: cell, amount: hp_per_interval, cap: HealCap::Max, source: Some("hazard:volatility".into()), _marker: PhantomData }`. `HealCap::Max` clamps at `hp.max.unwrap_or(hp.starting)`, which (because `attach_volatility_timers` lifted `hp.max` to `starting * max_multiplier`) allows growth up to the 2× cap. The pre-send gate above is belt-and-braces.
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
| `shared::death_pipeline` | Reads cell HP to check cap | Direct query of `&Hp` |
| `shared::death_pipeline` | Heals cells | `HealDealt<Cell>` with `HealCap::Max` (relies on lifted `Hp.max`) |
| `shared::death_pipeline` | Reads damage events to reset timer | `DamageDealt<Cell>` (read only) |

## Expected Behaviors (for test specs)

1. **Cell gains HP after not being hit at stack=1**
   - Given: Cell with 10 HP, starting HP 10, `Hp.max` lifted to `Some(20.0)` by `attach_volatility_timers`, `VolatilityTimer` at 0.0, stack=1
   - When: 5.0 seconds pass with no damage
   - Then: `HealDealt::<Cell> { amount: 1.0, cap: HealCap::Max, .. }` sent, cell now 11 HP

2. **Timer resets on damage**
   - Given: Cell with `VolatilityTimer.elapsed = 4.5` (0.5s from next tick)
   - When: Cell takes damage
   - Then: `VolatilityTimer.elapsed` resets to 0.0, next growth tick is 5.0s away

3. **HP caps at 2x starting HP**
   - Given: Cell with starting HP 10, `Hp.max = Some(20.0)`, current HP 19, stack=1
   - When: Volatility tick fires
   - Then: `HealDealt::<Cell> { amount: 1.0, cap: HealCap::Max, .. }` sent; `apply_heal` clamps at 20 so cell reaches exactly 20. Next tick: Volatility's pre-send gate suppresses emission because `hp.current >= hp.starting * max_multiplier`; no `HealDealt<Cell>` sent.

4. **Growth rate increases with stacking at stack=3**
   - Given: Cell with 10 HP, starting HP 10, `Hp.max = Some(20.0)`, stack=3 (interval=3.33s)
   - When: 3.33 seconds pass with no damage
   - Then: `HealDealt::<Cell> { amount: 1.0, cap: HealCap::Max, .. }` sent

5. **System does not run when hazard is inactive**
   - Given: Volatility not in `ActiveHazards`
   - When: time passes
   - Then: No `VolatilityTimer` components exist, no `HealDealt<Cell>` sent

## Edge Cases

- **Echo Cells + Volatility synergy**: Ghost cells (1 HP) grow rapidly if not cleared immediately. At stack=1, a ghost reaches 2 HP in 5 seconds. This is the intended trap -- ghosts look free but become real threats.
- **Fracture + Volatility synergy**: Split debris (1 HP) also grows if neglected. Combined with Fracture creating many small cells, Volatility turns "easy cleanup" into a race.
- **Cell spawned mid-node**: If a cell is created mid-node (by Fracture, Momentum split, or Echo Cells), it needs a `VolatilityTimer`. The `attach_volatility_timers` system should run as a reactive system (detect cells without the component) or the spawning system should add it. `Hp.starting` is set by the builder so no additional component is needed for the cap math.
- **Damage of 0**: A 0-damage hit (from a chip or effect that applies 0 damage) should still reset the timer -- the cell was "touched."
- **Multiple heals per frame**: If `elapsed` accumulates past multiple intervals (e.g., game was paused then unpaused), the while loop fires multiple `HealDealt<Cell>` messages in one frame. This is correct.
- **Cleanup**: `VolatilityTimer` is on cell entities -- cleaned up when cells despawn at node end. `VolatilityConfig` removed at run end.
