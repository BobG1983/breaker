# Erosion: design doc reflects EffectStack reconciliation (not ApplyBreakerShrink message)

## Problems addressed

- `audit/hazards/erosion.md` Issue 2 \u2014 Design \u00a7Messages specs an `ApplyBreakerShrink` message that was never built. Impl uses source-tagged `EffectStack<SizeBoostConfig>` reconciliation instead \u2014 cleaner, idempotent, matches Haste's convention for continuous modulation. Design doc needs updating to match reality.

## Remediation

Open `docs/todos/detail/mod-system-design/hazards/erosion.md`.

\u00a7Messages \u2014 delete the `ApplyBreakerShrink` reference. Replace with:

> **Writes**: Source-tagged `EffectStack<SizeBoostConfig>` entry on every Breaker (source: `"hazard:erosion"`). The entry's `width_fraction` is reconciled each FixedUpdate to reflect the current eroded width. Idempotent: the same source is retained-and-replaced via `EffectStack::retain_by_source`, so re-running the reconciliation every frame produces stable state.
>
> This pattern replaces the earlier design proposal for an `ApplyBreakerShrink` delta message. Reconciliation-based updating is cleaner for continuous modulation because it doesn't require the consumer to track deltas \u2014 the aggregated value is always the current truth.

\u00a7Systems \u2014 update the description of `erosion_apply_width`:

> `erosion_apply_width` runs `.after(erosion_shrink).after(erosion_restore)`. For every Breaker, it reads the current `ErosionState.width_fraction` and writes a single `EffectStack<SizeBoostConfig>` entry with source `"hazard:erosion"`, value `width_fraction`. The stack's aggregate drives visual scale and collision half-width (per `erosion-shrink-width-only.md`).

No code change. No test change. Doc-only fix.
