# Overcharge: align design doc with lazy-insertion + fractional field names

## Problems addressed

- `audit/hazards/overcharge.md` Issue 2 — Design doc references `attach_overcharge_tracker` system that doesn't exist. Impl uses lazy insertion in `overcharge_count_kills`.
- `audit/hazards/overcharge.md` Issue 3 — Design fields use percent (`base_speed_per_kill: 5.0`); impl uses fractional (`base_frac: 0.05`). Mismatch.

## Context

Split from the original `overcharge-ron-and-design-alignment.md`. The RON portion (value fix `base_frac: 0.1 → 0.05`, `per_level_frac: 0.05 → 0.03`) plus the field rename in code (`base_speed_per_kill` → `base_frac`, `per_level_increase_per_kill` → `per_level_frac`) ride with `audit/remediations/ron-tuning-values.md`. This file retains only the design-doc alignment work.

## Remediation

Open `docs/todos/detail/mod-system-design/hazards/overcharge.md`.

§Config Resource — replace the `base_speed_per_kill: f32` / `per_level_increase_per_kill: f32` fields with:

```rust
pub(crate) struct OverchargeConfig {
    /// Base speed-multiplier increment per kill, as a fraction (0.05 = +5%).
    pub base_frac: f32,
    /// Per-stack additional increment per kill, as a fraction (0.03 = +3% per level beyond 1).
    pub per_level_frac: f32,
}
```

The per-kill multiplier is `1.0 + base_frac + per_level_frac * (stacks - 1)`. Raised to the power of `kills`. Example: stack 3, 1 kill → `1.0 + 0.05 + 0.03 * 2 = 1.11`; 10 kills → `1.11^10 ≈ 2.84`.

§Systems — delete the `attach_overcharge_tracker` entry. Replace with:

> `OverchargeKillCount` is inserted LAZILY on each bolt's first kill (via `overcharge_count_kills`). No separate attach system; the count starts at zero and is created the moment a bolt first matters. When a bolt despawns, its count despawns with it — no cleanup needed.

§Expected Behaviors worked examples — update to use the fractional values:

- Behavior 1: `base_frac: 0.05`, stack 1 (multiplier 1.05), 1 kill → `base_speed * 1.05`.
- Behavior 4: `base_frac: 0.05`, `per_level_frac: 0.03`, stack 3 (multiplier 1.11), 1 kill → `base_speed * 1.11`.

No code change in this remediation (code-side field rename is handled by the RON tuning remediation). No test change.

## Scope note

This remediation is part of the broader design-doc-alignment sweep.
