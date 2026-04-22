# Erosion — `EffectStack<SizeBoostConfig>` reconciliation in the canonical design doc

## Target file

`docs/design/hazards/erosion.md` (promoted during this sweep).

## What the target doc must say

§ Messages:

> **Writes**: Source-tagged `EffectStack<SizeBoostConfig>` entry on every Breaker (source: `"hazard:erosion"`). The entry's `width_fraction` is reconciled each FixedUpdate to reflect the current eroded width. Idempotent: the same source is retained-and-replaced via `EffectStack::retain_by_source`, so re-running the reconciliation every frame produces stable state.
>
> Reconciliation-based updating is cleaner for continuous modulation than a delta message — the aggregated value is always the current truth.

§ Systems — for `erosion_apply_width`:

> `erosion_apply_width` runs `.after(erosion_shrink).after(erosion_restore)`. For every Breaker, it reads the current `ErosionState.width_fraction` and writes a single `EffectStack<SizeBoostConfig>` entry with source `"hazard:erosion"`, value `width_fraction`. The stack's aggregate drives visual scale and collision half-width (X-only — Erosion does not shrink Y).

## What the target doc must NOT say

- Do not reference an `ApplyBreakerShrink` message — that design was never built and the reconciliation pattern supersedes it.

## Pipeline position (dmg crate)

- **Not in the death pipeline.** Erosion does not participate in any `DeathPipelineSystems` set.
- **Trigger**: `BumpPerformed` grade (Perfect / non-whiff) drives the restore side; a FixedUpdate shrink timer drives the shrink side.
- **Writes**: `EffectStack<SizeBoostConfig>` reconciled per-tick on every Breaker (source `"hazard:erosion"`).
- **No** `DamageDealt<T>` / `HealDealt<T>` / `Destroyed<T>` / `DamageBoostStack` involvement.

## Why

`EffectStack` reconciliation is the canonical pattern for continuous modulation on Breaker components. `SizeBoostConfig` survives post-TODO #1.
