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

None. Momentum reads `DamageDealt<Cell>` (non-lethal) and responds with `HealDealt<Cell>` + potential split. The cell's current HP is queried directly from `&Hp`; the pristine HP is `Hp.starting` (unified death pipeline — no separate component needed).

## Messages

**Reads**: `DamageDealt<Cell>` (to detect non-lethal hits and compute heal amount).
**Sends**: `HealDealt<Cell>` with `HealCap::Max` — the whole point of Momentum is to push cells *past* their pristine HP, so the pipeline cap uses `Hp.max.unwrap_or(Hp.starting)`. The split mechanic, not the heal cap, is what bounds runaway growth (cell resets to `Hp.starting` when it hits `Hp.starting * split_threshold_multiplier`).

The split check is a hazard-domain system that reads cell HP and spawns new cells.

## Systems

1. **Heal-on-nonlethal** (hazard domain)
   - Schedule: `FixedUpdate`
   - Run if: `hazard_active(HazardKind::Momentum)` AND `in_state(NodeState::Playing)`
   - Ordering: After `apply_damage::<Cell>` (so the damage is in), before `DetectDeaths` (so the heal target isn't marked `Dead` before receiving the heal — actually the heal pipeline runs after `HandleKill` so lethal hits are filtered out by `Without<Dead>` naturally; the Momentum trigger still reads damage + cell HP and decides whether to emit a heal).
   - Behavior: Read `DamageDealt<Cell>` messages. For each entry, if the target's `hp.current > 0.0` after the damage was applied (non-lethal), compute `heal = base_hp_per_hit + hp_per_level * (stack - 1)` and emit `HealDealt::<Cell> { healer: None, target: cell, amount: heal, cap: HealCap::Max, source: Some("hazard:momentum".into()), _marker: PhantomData }`.

2. **`momentum_split_check`** (hazard domain)
   - Schedule: `FixedUpdate`
   - Run if: `hazard_active(HazardKind::Momentum)` AND `in_state(NodeState::Playing)`
   - Ordering: After `apply_heal::<Cell>` in the same tick, so the heal has already landed.
   - Behavior:
     1. For each cell with `Hp`, check `hp.current >= hp.starting * split_threshold_multiplier`
     2. If threshold reached:
        a. Find up to 2 adjacent empty grid positions
        b. Spawn new cells at those positions, each built with `Hp::new(hp.starting)` (no special components)
        c. Set the original cell's `hp.current` back to `hp.starting` (it "splits" -- doesn't keep the excess)
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
| `shared::death_pipeline` | Heals cells | `HealDealt<Cell>` with `HealCap::Max` (send) |
| `cells` | Spawns new cells on split | Cell spawn mechanism (message or direct, depends on cells domain API) |
| `cells` | Reads damage events | `DamageDealt<Cell>` (read) |

**Cell spawning**: Split cells are basic cells (no special rules, no locks, no special cell types). They inherit the parent's starting HP but not any special behaviors. The cells domain must provide a way to spawn a basic cell at a grid position mid-node.

## Expected Behaviors (for test specs)

1. **Non-lethal hit heals cell at stack=1**
   - Given: Cell with 10 HP (starting HP 10), bolt deals 5 damage (cell survives at 5 HP), stack=1
   - When: `momentum_heal_on_nonlethal` runs
   - Then: `HealDealt::<Cell> { target: cell, amount: 10.0, cap: HealCap::Max, .. }` sent. Cell goes from 5 HP to 15 HP.

2. **Cell splits at 2x starting HP**
   - Given: Cell with 20 HP (starting HP 10), 2 empty adjacent positions
   - When: `momentum_split_check` runs
   - Then: Original cell `hp.current` set to `hp.starting` (10.0). Two new cells spawned at adjacent positions, each built with `Hp::new(10.0)` (so `starting = 10.0`).

3. **Lethal hit does NOT trigger heal**
   - Given: Cell with 10 HP, bolt deals 15 damage (cell dies)
   - When: `momentum_heal_on_nonlethal` runs
   - Then: No `HealDealt<Cell>` sent (cell is dead)

4. **HP per hit scales with stack at stack=3**
   - Given: Cell with 50 HP (starting HP 50), bolt deals 10 damage (survives at 40 HP), stack=3
   - When: `momentum_heal_on_nonlethal` runs
   - Then: `HealDealt::<Cell> { target: cell, amount: 30.0, cap: HealCap::Max, .. }` sent. Cell goes from 40 to 70 HP.

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
- **Split cell inherits hazard components**: New cells from splits get `Hp::new(parent.hp.starting)` from the cell builder (so `starting` and `current` are both set correctly for the unified death pipeline). If Volatility is active, a `VolatilityTimer` must also be attached. The spawn system must ensure hazard components are attached to dynamically spawned cells.
- **Cascade chain**: If a split cell is immediately adjacent to a cell that then dies, Cascade heals it. Combined with Momentum's HP growth, cells become very hard to kill. This is the intended trap synergy.
- **Overflow prevention**: If a cell accumulates massive HP (e.g., 100x starting), the split mechanic still only resets to 1x. The system is self-regulating -- splits produce more cells but each at base HP.
- **Cleanup**: `Hp.starting` is part of the unified `Hp` component — cleaned up on cell despawn. `MomentumConfig` removed at run end.
