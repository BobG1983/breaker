# Hazard: Momentum

## Game Design

Non-lethal hits give the cell HP. When a cell reaches 2× its starting HP, it splits into 2 cells at 1× starting HP each, placed at offsets from the origin cell. Punishes chip effects that deal repeated small damage (DoT, low-damage multi-hits). Rewards one-shot kills. Player must build for burst damage or accept that every non-lethal hit feeds the problem.

**Stacking formula**: `+10 HP + 10 HP * (stack - 1)` per non-lethal hit. Stack 1 adds 10 HP per hit; stack 3 adds 30 HP per hit.

## Config Resource

```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct MomentumConfig {
    pub base_hp_per_hit: f32,            // 10.0
    pub hp_per_level: f32,               // 10.0
    pub split_threshold_multiplier: f32, // 2.0
    pub split_radius: f32,               // offset distance for split cells (tuned to cell spacing)
    pub cell_half_size: f32,             // occupancy-check radius
}
```

Populated from `HazardTuning::Momentum`.

## Components
None. Momentum reads `Hp` cross-domain; pristine HP comes from `Hp.starting` (unified death pipeline — no separate component).

## Messages
**Reads**: `DamageDealt<Cell>` (from `rantzsoft_dmg`) to detect non-lethal hits.
**Sends**: `HealDealt<Cell>` with `HealCap::Max` — the point is to push cells past pristine, so the pipeline cap uses `Hp.max.unwrap_or(Hp.starting)`. The split mechanic bounds growth: cell resets to `Hp.starting` when it hits `Hp.starting * split_threshold_multiplier`. No dedicated split message — `Cell::builder().at_position(...).hp(starting).spawn()` creates each new cell.

## Systems

### `momentum_heal_on_nonlethal`
- **Schedule**: `FixedUpdate`, in `DeathPipelineSystems::EmitHeal`.
- **run_if**: `hazard_active(HazardKind::Momentum)` + `in_state(NodeState::Playing)`.
- **Behavior**: Reads `DamageDealt<Cell>`. For each damaged target with `hp.current > 0.0` (survived): `heal = base_hp_per_hit + hp_per_level * (stack - 1)`, emits `HealDealt<Cell> { target, amount: heal, cap: HealCap::Max, source: Some("hazard:momentum".into()), .. }`.

### `momentum_split_check`
- **Schedule**: `FixedUpdate`, `.after(DeathPipelineSystems::ApplyHeal)`.
- **run_if**: `hazard_active(HazardKind::Momentum)` + `in_state(NodeState::Playing)`.
- **Behavior**: For each cell with `Hp`, if `hp.current >= hp.starting * split_threshold_multiplier`:
  1. Generate the 4 offset positions `[(+split_radius, 0), (-split_radius, 0), (0, +split_radius), (0, -split_radius)]` from the cell's world position.
  2. Check each via `Quadtree::query_circle(offset_pos, cell_half_size)` — keep the first 2 that are empty.
  3. Spawn at each kept offset via `Cell::builder().at_position(pos).hp(starting).spawn()`.
  4. Reset origin cell's `hp.current = hp.starting` (the cell "splits" — doesn't keep excess).
  5. If fewer than 2 empty offsets exist, spawn as many as possible (1 or 0). If 0, the origin keeps its current HP and re-checks next tick.

## Pipeline position (dmg crate)

- **Trigger**: reads `DamageDealt<Cell>` from `rantzsoft_dmg` (for the heal) and queries `Hp` directly (for the split check).
- **Emits**: `HealDealt<Cell>` with `HealCap::Max` in `DeathPipelineSystems::EmitHeal`. Spawns new cell entities on split via `Cell::builder()`.
- **Ordering**: heal in `EmitHeal`; split check `.after(ApplyHeal)` so the heal has already pushed the cell past threshold.
- **No** `DamageBoostStack` / `VulnerableStack` involvement.

## Stacking Behavior

| Stack | HP per non-lethal hit | Hits to split (10 HP cell) |
|-------|-----------------------|----------------------------|
| 1 | 10 | 1 (10 + 10 = 20 = 2×) |
| 2 | 20 | 1 (well past threshold) |
| 3 | 30 | 1 |

Low-HP cells split easily on any non-lethal hit. High-HP cells resist — stack 1 on a 50 HP cell takes 5 non-lethal hits. Stacking accelerates across the board.

## Cross-Domain Dependencies
- **cells**: Reads `Hp`, `Position2D`. Spawns new cells via `Cell::builder().at_position(pos).hp(starting).spawn()`.
- **physics (`rantzsoft_physics2d`)**: `Quadtree::query_circle` for empty-slot checks. Works uniformly for static grid cells and animated boss cells.
- **damage crate (`rantzsoft_dmg`)**: Reads `DamageDealt<Cell>`; emits `HealDealt<Cell>`.

## Expected Behaviors (for test specs)

1. **Non-lethal hit heals cell at stack 1** — cell 10/10 HP, 5 damage, stack 1: `HealDealt<Cell> { amount: 10.0, cap: HealCap::Max }` → cell at 15 HP.
2. **Cell splits at 2× starting HP** — cell at 20/10 HP (`starting` 10, `hp.max` 20), 2 empty offset slots: 2 new cells spawned at offsets, origin resets to 10 HP.
3. **Lethal hit does NOT trigger heal** — cell dies from the damage: no `HealDealt<Cell>` emitted.
4. **HP per hit scales with stack 3** — stack 3: `HealDealt<Cell> { amount: 30.0 }`.
5. **Split with limited empty slots** — only 1 empty offset: 1 new cell spawned, origin resets.
6. **Split blocked by no empty slots** — all 4 offsets occupied: no split; origin retains HP for next tick re-check.
7. **Works for off-grid boss cells** — boss cell at animated position `(203.5, 71.8)` crosses threshold: offsets computed from that world position; range-query occupancy check includes neighbouring boss cells.

## Edge Cases
- **Momentum + Diffusion**: Diffusion bleeds damage, preventing one-shot kills → non-lethal hits feed Momentum growth + splits. Feedback loop: trying to kill one cell strengthens neighbors.
- **Momentum + Fracture**: Fracture creates 1-HP debris on death; Momentum splits create cells on survival. Triggers are disjoint — no conflict.
- **Split cells inherit hazard attach**: new cells from splits go through the standard builder path and receive `Added<Cell>` attach hooks (Volatility timer, Renewal timer, etc. — per TODO #8 attach-system-migration).
- **Cascade chain**: a split-spawned cell adjacent to a dying cell gets Cascade-healed — combined with Momentum's HP growth, cells become very hard to kill. Intended trap.
- **Overflow prevention**: a cell that accumulates massive HP (e.g., 100× starting) still only resets to 1× on split — self-regulating (more cells, each at base HP).
- **Cleanup**: `Hp.starting` lives in the unified `Hp` component (cleaned up on despawn). `MomentumConfig` removed at run end.
