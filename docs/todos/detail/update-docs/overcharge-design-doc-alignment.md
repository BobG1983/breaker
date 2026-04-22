# Overcharge — canonical design doc shape

## Target file

`docs/design/hazards/overcharge.md` (promoted during this sweep).

## What the target doc must say

§ Config Resource:

```rust
pub(crate) struct OverchargeConfig {
    /// Base speed-multiplier increment per kill, as a fraction (0.05 = +5%).
    pub base_frac: f32,
    /// Per-stack additional increment per kill, as a fraction (0.03 = +3% per level beyond 1).
    pub per_level_frac: f32,
}
```

> Per-kill multiplier: `1.0 + base_frac + per_level_frac * (stacks - 1)`. Raised to the power of `kills`. Example: stack 3, 1 kill → `1.0 + 0.05 + 0.03 * 2 = 1.11`; 10 kills → `1.11^10 ≈ 2.84`.

§ Systems:

> `OverchargeKillCount` is inserted LAZILY on each bolt's first kill (inside `overcharge_count_kills`). This is distinct from the `Added<Cell>` / `Added<Bolt>` attach pattern used for timers that must exist from spawn — Overcharge's count only matters once a bolt has made a kill, and most bolts never do.
>
> When a bolt despawns, its count despawns with it — no cleanup system needed.

§ Expected Behaviors — worked examples:

- Behaviour 1: `base_frac: 0.05`, stack 1 (multiplier 1.05), 1 kill → `base_speed * 1.05`.
- Behaviour 4: `base_frac: 0.05`, `per_level_frac: 0.03`, stack 3 (multiplier 1.11), 1 kill → `base_speed * 1.11`.

## What the target doc must NOT say

- Do not describe fields as `base_speed_per_kill` / `per_level_increase_per_kill`.
- Do not reference an `attach_overcharge_tracker` system — lazy insertion replaces it.
- Do not use percent-valued (`5.0`) examples — fractional (`0.05`) is the canonical form.

## Pipeline position (dmg crate)

- **Trigger**: reads `Destroyed<Cell>` from the `rantzsoft_dmg` crate inside `overcharge_count_kills` — on each kill attributed to a bolt (via `KilledBy`), the bolt's `OverchargeKillCount` is incremented (lazily inserted on first kill).
- **Not a damage emitter or mutator.** Overcharge does NOT participate in any `DeathPipelineSystems` set. After counting, it reconciles `EffectStack<SpeedBoostConfig>` on the bolt (source `"hazard:overcharge"`) — a speed multiplier, not a damage one. Reset on `BumpPerformed` clears the count.
- **Ordering**: `overcharge_count_kills` runs `.after(DeathPipelineSystems::ApplyKill)` so kill attribution is resolved.
- **No** `DamageDealt<T>` / `HealDealt<T>` / `DamageBoostStack` involvement.

## Why

Lazy insertion is the right trigger here: trackers only matter once a bolt has made a kill, and most bolts never do. This is distinct from cell-lifetime timers (Renewal / Volatility) which use `Added<Cell>` — those MUST exist from the moment the cell spawns, because their effect measures elapsed cell-time. Overcharge measures kills, not time, so spawn is the wrong trigger.
