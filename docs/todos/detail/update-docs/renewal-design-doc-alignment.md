# Renewal — canonical design doc shape

## Target file

`docs/design/hazards/renewal.md` (promoted during this sweep).

## What the target doc must say

§ Components:

```rust
pub(crate) struct RenewalTimer {
    /// Seconds until the next heal fires. On expiry, heals the cell to starting
    /// HP and resets to the duration computed from the current active stacks.
    pub remaining: f32,
}
```

> Duration is recomputed each reset from live `HazardActive.stacks` + `RenewalConfig`. Storing it on the component is redundant; the recompute guarantees the newest stack count applies to the next cycle.

§ Config Resource fields:
- `base_period_secs: f32` (not `base_timer`)
- `per_level_reduction_frac: f32` (fractional units, not percent)

Per-stack duration formula: `duration = base_period_secs * (1.0 - per_level_reduction_frac)^(stacks - 1)`.

§ Systems — two systems:

> **`attach_renewal_timer`** (schedule: `FixedUpdate`, `run_if = hazard_active(Renewal) + in_state(NodeState::Playing)`)
>
> ```rust
> Query<Entity, Added<Cell>>
> ```
>
> On each newly spawned cell, inserts `RenewalTimer { remaining: duration_secs(current_stacks) }`. `Added<Cell>` fires exactly once per entity on spawn — no idempotency checks needed, no every-tick scan. Since hazards activate at node boundaries (not mid-node), every cell spawned during a Renewal-active node hits `Added<Cell>` and gets the timer.
>
> **`renewal_tick`** (schedule: `FixedUpdate`, ordered `.after(DeathPipelineSystems::ApplyKill).before(DeathPipelineSystems::ApplyHeal)`)
>
> Decrements `RenewalTimer.remaining` by `dt`. On expiry: if `hp.current < hp.starting`, emits `HealDealt<Cell> { target, amount: starting - current, cap: HealCap::Starting, source: "hazard:renewal" }`. Resets `remaining` to `duration_secs(current_stacks)`. If `hp.current >= hp.starting`, skips the heal emit but still resets the timer.

§ Ordering rationale:

> `renewal_tick` must run after `ApplyKill` (so dead cells are removed from the query) and before `ApplyHeal` (so this tick's heals feed the same-tick apply).

## What the target doc must NOT say

- Do not include a `duration` field on `RenewalTimer`.
- Do not reference `base_timer` or percent-valued `per_level_reduction_percent`.
- Do not describe a 3-system split (`attach` + `init` + `reset_on_stack_change`) — the 2-system shape supersedes it.
- Do not describe `attach_renewal_timer` as an every-tick `Query<Entity, (With<Cell>, Without<RenewalTimer>)>` scan — `Added<Cell>` is the canonical trigger (per TODO #9 attach-system-migration).

## Pipeline position (dmg crate)

- **Trigger**: FixedUpdate tick (`renewal_tick` decrement), gated by `hazard_active(Renewal) + in_state(NodeState::Playing)`.
- **Emits**: `HealDealt<Cell> { target, amount, cap: HealCap::Starting, source: "hazard:renewal" }` from `rantzsoft_dmg`.
- **Ordering**: `renewal_tick` runs in `DeathPipelineSystems::EmitHeal`, `.after(DeathPipelineSystems::ApplyKill).before(DeathPipelineSystems::ApplyHeal)`.
- **Attach**: `attach_renewal_timer` reads `Added<Cell>` — distinct system, runs in `FixedUpdate` outside the death-pipeline sets.
- **No** `DamageDealt<T>` / `Destroyed<T>` / `DamageBoostStack` involvement.

## Why

`Added<Cell>` is the canonical attach trigger. The 2-system shape is cleaner and the ordering pin prevents both heal-dead-cells and cross-tick ordering drift. Field renames improve intuition (fraction vs percent units).
