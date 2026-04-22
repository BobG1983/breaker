# Hazard: Erosion

## Game Design

Breaker shrinks over time. Non-whiff bumps restore 25% of lost width; Perfect bumps restore 50%. Minimum width 35%. Bump window height scales with width. Idle play melts the breaker — creates "stay active" pressure that compounds with Haste (faster bolt + smaller target).

## Config Resource

```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct ErosionConfig {
    pub shrink_rate: f32,           // fraction of full width per second (e.g., 0.05 = 5%/s)
    pub min_width_fraction: f32,    // 0.35
    pub non_whiff_restore: f32,     // 0.25
    pub perfect_restore: f32,       // 0.50
}
```

Populated from `HazardTuning::Erosion`.

### `ErosionState`

```rust
#[derive(Resource, Debug)]
pub(crate) struct ErosionState {
    pub width_fraction: f32,    // 1.0 at node start, clamped to [min, 1.0]
}
```

Inserted alongside `ErosionConfig` at activation. One resource — applies globally to every Breaker.

## Components
None.

## Messages
**Reads**: `BumpPerformed { grade, .. }` — existing from the bolt/breaker domain.
**Writes**: Source-tagged `EffectStack<SizeBoostConfig>` entry on every Breaker (source: `"hazard:erosion"`). The entry's `width_fraction` is reconciled each FixedUpdate. Idempotent via `EffectStack::retain_by_source` — rerunning each frame is stable. Reconciliation-based updating is cleaner for continuous modulation than a delta message — the aggregated value is always current truth.

## Systems

### `erosion_shrink`
- **Schedule**: `FixedUpdate`.
- **run_if**: `hazard_active(HazardKind::Erosion)` + `in_state(NodeState::Playing)`.
- **Behavior**: `shrink_per_tick = shrink_rate * stack * delta_secs`. Decrements `ErosionState.width_fraction`, clamped to `min_width_fraction`.

### `erosion_restore`
- **Schedule**: `FixedUpdate`, `.after(erosion_shrink)`.
- **run_if**: `hazard_active(HazardKind::Erosion)` + `in_state(NodeState::Playing)`.
- **Behavior**: Reads `BumpPerformed`. Per bump:
  - `Perfect`: restore `perfect_restore * (1.0 - width_fraction)`.
  - `Early`/`Late`: restore `non_whiff_restore * (1.0 - width_fraction)`.
  - `Whiff`: no restore.
  - Clamp `width_fraction` to 1.0.

### `erosion_apply_width`
- **Schedule**: `FixedUpdate`, `.after(erosion_shrink).after(erosion_restore)`.
- **run_if**: `hazard_active(HazardKind::Erosion)` + `in_state(NodeState::Playing)`.
- **Behavior**: For every Breaker, writes a single `EffectStack<SizeBoostConfig>` entry with source `"hazard:erosion"`, value `width_fraction`. The stack's aggregate drives visual scale and collision half-width (X-only — Erosion does not shrink Y). The breaker domain's bump-window height system reads the same aggregate to scale the window proportionally.

## Pipeline position (dmg crate)

- **Not in the death pipeline.** Erosion does not participate in any `DeathPipelineSystems` set.
- **Trigger**: `BumpPerformed` grade (Perfect / non-whiff) drives the restore side; a FixedUpdate shrink tick drives the shrink side.
- **Writes**: `EffectStack<SizeBoostConfig>` reconciled per-tick on every Breaker (source `"hazard:erosion"`).
- **No** `DamageDealt<T>` / `HealDealt<T>` / `Destroyed<T>` / `DamageBoostStack` involvement.

## Stacking Behavior

Linear shrink-rate scaling: `effective_shrink_rate = shrink_rate * stack`.

| Stack | Shrink multiplier | Notes |
|-------|-------------------|-------|
| 1 | 1× | Gradual, easy to maintain |
| 2 | 2× | Must bump frequently |
| 3 | 3× | Aggressive |

Restoration percentages (25%/50%) are stack-independent. At high stacks, restoration can't keep pace — the breaker trends toward `min_width_fraction`. Floor is fixed at 35% regardless of stack.

## Cross-Domain Dependencies
- **breaker**: Consumes the aggregated `EffectStack<SizeBoostConfig>` — owns width and bump-window-height scaling.
- **bolt**: Emits `BumpPerformed` (read by `erosion_restore`).

## Expected Behaviors (for test specs)

1. **Breaker shrinks over time at stack 1** — `shrink_rate: 0.05`, `width_fraction: 1.0`, stack 1, `delta_secs: 1.0`: `width_fraction -> 0.95`; reconciled `EffectStack` entry reflects 0.95.
2. **Breaker shrinks faster at stack 3** — stack 3, same config: `width_fraction -> 0.85`.
3. **Breaker does not shrink below minimum** — `min_width_fraction: 0.35`, `width_fraction: 0.36`: clamped to 0.35; no over-shrink.
4. **Perfect bump restores 50% of lost width** — `width_fraction: 0.60`, `perfect_restore: 0.50`: lost 0.40, restore 0.20, new `width_fraction: 0.80`.
5. **Non-whiff bump restores 25% of lost width** — `width_fraction: 0.60`, `non_whiff_restore: 0.25`, `Early` grade: new `width_fraction: 0.70`.
6. **Whiff bump restores nothing** — `Whiff` grade: `width_fraction` unchanged.
7. **EffectStack entry reconciled each tick** — same source `"hazard:erosion"` replaced-in-place via `retain_by_source`; running the system twice leaves exactly one entry.

## Edge Cases
- **Erosion + Haste**: faster bolt + shrinking breaker — precision pressure.
- **Erosion + Overcharge**: accelerating bolt must be caught on a shrinking target.
- **Cleanup**: `ErosionConfig` + `ErosionState` removed at run end. `EffectStack<SizeBoostConfig>` entries cleared by source when the hazard deactivates.
- **Restoration near full width**: lost width is small, so restoration yields diminishing absolute gains.
- **Multiple bumps per frame**: each bump re-reads `width_fraction` as-of-its-processing — the second bump sees less lost width.
- **At minimum width**: shrink stops; restoration still works, but immediately begins eroding again.
