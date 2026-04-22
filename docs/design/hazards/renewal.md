# Hazard: Renewal

## Game Design

Cells have a countdown timer and regenerate to starting HP on expiry, then the timer resets. 10s base period, `-20%` per level (diminishing returns). Creates a "beat the clock per cell" mechanic — player must finish off damaged cells before they heal. Partially damaged cells that survive long enough snap back to full.

## Config Resource

```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct RenewalConfig {
    pub base_period_secs: f32,          // 10.0
    pub per_level_reduction_frac: f32,  // 0.20 (fractional — 20% reduction per stack beyond 1)
}
```

Per-stack duration formula: `duration = base_period_secs * (1.0 - per_level_reduction_frac)^(stacks - 1)`. Diminishing returns — the timer asymptotes toward 0, never reaches it.

| Stack | Duration |
|-------|----------|
| 1 | 10.0s |
| 2 | 8.0s |
| 3 | 6.4s |
| 5 | 4.096s |
| 10 | 1.342s |

Populated from `HazardTuning::Renewal`.

## Components

```rust
#[derive(Component, Debug)]
pub(crate) struct RenewalTimer {
    /// Seconds until the next heal fires. On expiry, heals the cell to starting
    /// HP and resets to the duration computed from the current active stacks.
    pub remaining: f32,
}
```

Duration is recomputed each reset from live `ActiveHazards.stacks(Renewal)` + `RenewalConfig`. Storing it on the component would be redundant — the recompute guarantees the newest stack count applies to the next cycle.

## Messages
**Reads**: `Option<Res<ActiveHazards>>` for stack count (harness-safety).
**Sends**: `HealDealt<Cell> { target, amount, cap: HealCap::Starting, source: "hazard:renewal" }` from `rantzsoft_dmg`. `HealCap::Starting` enforces heal-to-pristine regardless of any buffed `Hp.max`.

## Systems

### `attach_renewal_timer`
- **Schedule**: `FixedUpdate`.
- **run_if**: `hazard_active(HazardKind::Renewal)` + `in_state(NodeState::Playing)`.
- **Query**: `Query<Entity, Added<Cell>>` — `Added<Cell>` fires exactly once per entity on spawn. No idempotency checks needed; no every-tick scan. Since hazards activate at node boundaries (not mid-node), every cell spawned during a Renewal-active node hits `Added<Cell>` and gets the timer.
- **Behavior**: On each newly-spawned cell, inserts `RenewalTimer { remaining: duration_secs(current_stacks) }`.

### `renewal_tick`
- **Schedule**: `FixedUpdate`, `.after(DeathPipelineSystems::ApplyKill).before(DeathPipelineSystems::ApplyHeal)`.
- **run_if**: `hazard_active(HazardKind::Renewal)` + `in_state(NodeState::Playing)`.
- **Behavior**: Decrements `RenewalTimer.remaining -= delta_secs`. On expiry:
  1. If `hp.current < hp.starting`: emits `HealDealt<Cell> { target, amount: starting - current, cap: HealCap::Starting, source: "hazard:renewal" }`.
  2. Resets `remaining = duration_secs(current_stacks)`.
  3. If `hp.current >= hp.starting`: skips the heal emit but still resets the timer.

**Ordering rationale**: `renewal_tick` runs after `ApplyKill` (so dead cells are removed from the query) and before `ApplyHeal` (so this tick's heals feed the same-tick apply).

## Pipeline position (dmg crate)

- **Trigger**: FixedUpdate tick (`renewal_tick` decrement), gated by `hazard_active(Renewal) + in_state(NodeState::Playing)`.
- **Emits**: `HealDealt<Cell> { target, amount, cap: HealCap::Starting, source: "hazard:renewal" }` from `rantzsoft_dmg`.
- **Ordering**: `renewal_tick` runs in `DeathPipelineSystems::EmitHeal`, `.after(DeathPipelineSystems::ApplyKill).before(DeathPipelineSystems::ApplyHeal)`.
- **Attach**: `attach_renewal_timer` reads `Added<Cell>` — distinct system, runs in `FixedUpdate` outside the death-pipeline sets.
- **No** `DamageDealt<T>` / `Destroyed<T>` / `DamageBoostStack` involvement.

## Stacking Behavior

Diminishing returns on duration: `duration = base_period_secs * (1.0 - per_level_reduction_frac)^(stacks - 1)`.

Early stacks have outsized impact (10 → 8 → 6.4s); later stacks contribute less. Cells regen almost instantly at extreme stacks, but the player is overwhelmed long before that.

## Cross-Domain Dependencies
- **cells**: Reads `Hp` (for heal-amount computation). `Added<Cell>` drives attach.
- **damage crate (`rantzsoft_dmg`)**: Emits `HealDealt<Cell>`.

## Expected Behaviors (for test specs)

1. **Cell regens to full after timer expires at stack 1** — cell 30/100 HP, `RenewalTimer { remaining: 0.05 }`, `delta_secs: 0.1`: `HealDealt<Cell> { amount: 70.0, cap: HealCap::Starting }` emitted; `remaining` resets to 10.0.
2. **Timer is shorter at stack 3** — stack 3, `base_period_secs: 10.0`, `per_level_reduction_frac: 0.20`: timer resets to `10.0 * 0.8^2 = 6.4s`.
3. **Full-HP cell resets timer without heal** — cell at starting HP, timer expires: no `HealDealt<Cell>` emitted; timer still resets.
4. **Timer ticks between regens** — `remaining: 5.0`, `delta_secs: 0.1`: `remaining = 4.9`; no heal.
5. **New cells get timer on spawn via `Added<Cell>`** — Renewal active at stack 2, new cell spawned: `RenewalTimer { remaining: 8.0 }` attached same tick.
6. **Ghost cells (Echo Cells) also get timers** — Echo-Cells ghost spawned while Renewal active: `Added<Cell>` fires, timer attached.

## Edge Cases
- **Renewal + Decay**: double time pressure. No special-case code — emergent.
- **Renewal + Cascade**: if Cascade heals above pristine HP (`Hp.max` buffed), Renewal's `HealCap::Starting` still clamps to starting. Independent.
- **Destroyed cells**: entity despawned → `RenewalTimer` goes with it. No stale cleanup.
- **Stack increase mid-run**: existing timers keep their `remaining`; next reset uses the new shorter duration via `duration_secs(current_stacks)`.
- **Diminishing-returns floor**: timer approaches 0 asymptotically. At stack 20 it's ~0.115s — in practice the player loses first.
- **Cleanup**: `RenewalConfig` removed at run end. `RenewalTimer` components cleaned up with cells at node end.
