# Renewal: align design doc with cleaner impl

## Problems addressed

- `audit/hazards/renewal.md` Issues 1, 2, 3, 4 \u2014 Design doc diverges from the cleaner impl in several places: `duration` field unused, 3-system split collapsed to 2, field names verbose in design / descriptive in impl, death-pipeline ordering unpinned.
- Issue 5 (cleanup) \u2014 Subsumed by `run-end-config-cleanup.md` (Renewal in "no state resources" list; per-cell RenewalTimer despawns with cells at node exit).

## Remediation

Open `docs/todos/detail/mod-system-design/hazards/renewal.md`.

\u00a7Components \u2014 replace:

```rust
pub(crate) struct RenewalTimer {
    pub remaining: f32,
    pub duration: f32,
}
```

with:

```rust
pub(crate) struct RenewalTimer {
    /// Seconds until the next heal fires. On expiry, heals the cell to starting
    /// HP and resets to the duration computed from the current active stacks.
    pub remaining: f32,
}
```

Note: duration is recomputed each reset from live `HazardActive.stacks` + `RenewalConfig`. Storing it on the component is redundant; the recompute guarantees the newest stack count applies to the next cycle.

\u00a7Config Resource \u2014 rename fields:

- `base_timer: f32` \u2192 `base_period_secs: f32`
- `per_level_reduction_percent: f32` (percent units) \u2192 `per_level_reduction_frac: f32` (fractional units)

Update the per-stack duration formula description: `duration = base_period_secs * (1.0 - per_level_reduction_frac)^(stacks - 1)`.

\u00a7Systems \u2014 replace the three-system description with two:

> **`renewal_attach_timers`** (runs every tick, gated on `hazard_active(Renewal) + in_state(Playing)`)
>
> Queries every `Cell` without a `RenewalTimer`. Attaches `RenewalTimer { remaining: duration_secs(current_stacks) }`. Idempotent: cells already with a timer are skipped. This subsumes both the "attach on activation" and "attach to newly spawned cells" concerns (Echo Cells ghosts, Fracture debris, Momentum splits).
>
> **`renewal_tick`** (runs every tick, ordered `.after(DeathPipelineSystems::HandleKill).before(DeathPipelineSystems::ApplyHeal)`)
>
> Decrements `RenewalTimer.remaining` by `dt`. On expiry: if `hp.current < hp.starting`, emits `HealDealt<Cell> { target, amount: starting - current, cap: HealCap::Starting, source: "hazard:renewal" }`. Resets `remaining` to `duration_secs(current_stacks)`. If `hp.current >= hp.starting`, skips the heal emit but still resets the timer.

Remove the `renewal_init_timers` and `renewal_reset_on_stack_change` entries. Both are subsumed by the two-system approach.

\u00a7Ordering \u2014 pin the death-pipeline requirement:

> `renewal_tick` MUST run `.after(DeathPipelineSystems::HandleKill).before(DeathPipelineSystems::ApplyHeal)`. After HandleKill: dead cells are removed from the query so Renewal doesn't heal them. Before ApplyHeal: this tick's heals feed the same tick's apply system.

### Code

No code change. The impl is already correct per the new design doc.

### Tests

No test change. Existing tests pin the impl's behavior; the design doc update aligns docs with already-correct code.
