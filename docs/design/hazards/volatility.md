# Hazard: Volatility

## Game Design

Cells gain HP when not being hit. HP caps at 2× starting HP. "Neglect tax" — player must keep touching cells to suppress growth. Focus on one area and unattended cells silently grow tougher.

**Base growth**: +1 HP per 5 seconds not being hit. Stacking shrinks the interval (cells grow faster). Floor prevents instant growth.

## Config Resource

```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct VolatilityConfig {
    pub hp_per_interval: f32,   // 1.0
    pub interval_secs: f32,     // 5.0 — base interval between growth ticks
    pub max_multiplier: f32,    // 2.0 — cap at 2× starting HP
    pub min_interval_secs: f32, // 1.0 — floor
}
```

**Stacking effect on interval**: `effective_interval = (interval_secs / (1.0 + 0.25 * (stack - 1))).max(min_interval_secs)`. Diminishing returns; floor at `min_interval_secs`.

| Stack | Effective interval |
|-------|--------------------|
| 1 | 5.0s |
| 2 | 4.0s |
| 3 | 3.33s |
| 5 | 2.5s |

Populated from `HazardTuning::Volatility`.

## Components

```rust
#[derive(Component, Debug)]
pub(crate) struct VolatilityTimer {
    pub elapsed: f32,
}
```

Attached via `Added<Cell>` to every spawned cell while Volatility is active. Timer resets on incoming damage; accrues growth ticks otherwise.

**Dynamic `Hp.max` lift**: `attach_volatility_timer` also lifts `Hp.max` to `max(existing_max, hp.starting * max_multiplier)` on each cell — takes the HIGHER of the cell's current `Hp.max` (if any) and `2× starting`. Never lowers an existing cap (future buffs with higher max are preserved); always ensures ceiling is AT LEAST `2× starting` so Volatility's growth path is reachable through `HealCap::Max`.

## Messages
**Reads**: `DamageDealt<Cell>` (from `rantzsoft_dmg`) to detect hits and reset timers.
**Sends**: `HealDealt<Cell> { target, amount: hp_per_interval, cap: HealCap::Max, source: "hazard:volatility" }`. `HealCap::Max` clamps arriving heals at `Hp.max` (lifted to `starting * max_multiplier`). Volatility also performs its OWN pre-send gate — emits the heal only if `hp.current < hp.starting * max_multiplier` — belt-and-braces cap enforcement.

## Systems

### `attach_volatility_timer`
- **Schedule**: `FixedUpdate`.
- **run_if**: `hazard_active(HazardKind::Volatility)` + `in_state(NodeState::Playing)`.
- **Query**: `Query<(Entity, &Hp), Added<Cell>>` — `Added<Cell>` fires once per entity on spawn. No every-tick `Without<VolatilityTimer>` scan (per TODO #8 attach-system-migration).
- **Behavior**: For each newly spawned cell:
  1. Insert `VolatilityTimer { elapsed: 0.0 }`.
  2. Lift `Hp.max = Some(hp.max.map_or(target, |m| m.max(target)))` where `target = hp.starting * max_multiplier`.

### `reset_volatility_on_damage`
- **Schedule**: `FixedUpdate`, `.after(DeathPipelineSystems::ApplyDamage)`.
- **run_if**: `hazard_active(HazardKind::Volatility)` + `in_state(NodeState::Playing)`.
- **Behavior**: Reads `DamageDealt<Cell>`. For each damaged cell with a `VolatilityTimer`, reset `elapsed = 0.0`. Zero-damage events also reset (the cell was "touched").

### `volatility_grow_cells`
- **Schedule**: `FixedUpdate`, `.after(reset_volatility_on_damage)`, in `DeathPipelineSystems::EmitHeal`.
- **run_if**: `hazard_active(HazardKind::Volatility)` + `in_state(NodeState::Playing)`.
- **Behavior**: For each cell with `VolatilityTimer` + `Hp`:
  1. `elapsed += delta_secs`.
  2. While `elapsed >= effective_interval`:
     - Pre-send gate: if `hp.current < hp.starting * max_multiplier`: emit `HealDealt<Cell> { target, amount: hp_per_interval, cap: HealCap::Max, source: "hazard:volatility" }`. `HealCap::Max` clamps at `hp.max` (= `starting * max_multiplier`); the pre-send gate is belt-and-braces.
     - Subtract `effective_interval` from `elapsed`.

## Pipeline position (dmg crate)

- **Trigger**: FixedUpdate tick (`volatility_grow_cells` accrual), gated by `hazard_active(Volatility) + in_state(NodeState::Playing)`.
- **Emits**: `HealDealt<Cell> { target, amount, cap: HealCap::Max, source: "hazard:volatility" }` from `rantzsoft_dmg`. `HealCap::Max` relies on `attach_volatility_timer` lifting `Hp.max` to `starting * max_multiplier`.
- **Ordering**: `volatility_grow_cells` runs in `DeathPipelineSystems::EmitHeal`, `.after(reset_volatility_on_damage)`.
- **Attach**: `attach_volatility_timer` reads `Added<Cell>` — distinct system, runs in `FixedUpdate` outside the death-pipeline sets.
- **Reset**: `reset_volatility_on_damage` reads `DamageDealt<Cell>` but does not emit damage — `.after(ApplyDamage)` so the damaged cell's state is committed before the timer reset.
- **No** `DamageBoostStack` / `VulnerableStack` / `Destroyed<T>` involvement.

## Stacking Behavior

| Stack | Interval | HP/s (per cell) | Time to 2× from base 10 HP |
|-------|----------|-----------------|----------------------------|
| 1 | 5.0s | 0.2 | 50s |
| 2 | 4.0s | 0.25 | 40s |
| 3 | 3.33s | 0.3 | 33s |

Per-cell growth rate is modest — threat is cumulative across 30+ cells.

## Cross-Domain Dependencies
- **cells**: Reads `Hp`; writes `Hp.max` once at attach time. Emits heals via `rantzsoft_dmg`.
- **damage crate (`rantzsoft_dmg`)**: Reads `DamageDealt<Cell>`; emits `HealDealt<Cell>`.

## Expected Behaviors (for test specs)

1. **Cell gains HP after no hits at stack 1** — cell 10/10 HP, `Hp.max = Some(20.0)` lifted, `VolatilityTimer { elapsed: 0.0 }`, stack 1; 5.0s pass: `HealDealt<Cell> { amount: 1.0, cap: HealCap::Max }` emitted; cell at 11 HP.
2. **Timer resets on damage** — `elapsed: 4.5` (0.5s from tick); cell takes damage: `elapsed = 0.0`; next tick 5.0s away.
3. **HP caps at 2× starting** — cell at 19/20 HP, stack 1: heal emitted; `apply_heal` clamps to 20. Next tick: pre-send gate suppresses (current >= max_multiplier * starting); no heal emitted.
4. **Growth rate scales with stack 3** — stack 3, interval 3.33s; 3.33s with no damage: `HealDealt<Cell> { amount: 1.0 }`.
5. **Inactive hazard = no growth** — no timers attached, no heals emitted.
6. **Added<Cell> attach** — new cell spawned mid-run (e.g., Fracture debris): `attach_volatility_timer` fires via `Added<Cell>`; timer + `Hp.max` lift applied same tick.
7. **Zero-damage hit resets timer** — a 0-damage chip effect still counts as a touch; `elapsed = 0.0`.
8. **Multi-interval elapsed**: `elapsed = 12.0` (> 2 × 5.0s): while-loop emits 2 heals same frame.

## Edge Cases
- **Echo Cells + Volatility**: 1-HP ghosts grow rapidly if not cleared. Stack 1 ghost reaches 2 HP in 5s. Intended trap.
- **Fracture + Volatility**: split debris (1 HP) grows if neglected — "easy cleanup" becomes a race.
- **Mid-node cell spawns**: Fracture debris / Echo-Cells ghosts / Momentum splits all go through the builder and hit `Added<Cell>` — `attach_volatility_timer` fires. `Hp.starting` is builder-set so the cap math is correct.
- **Zero damage**: a 0-damage hit resets the timer (cell was touched).
- **Cleanup**: `VolatilityTimer` on cells — cleaned up on despawn at node end. `VolatilityConfig` removed at run end. `Hp.max` lift is per-cell; no restoration needed since cells despawn.
