# Haste: document EffectStack reconciliation as canonical (drop ApplyBoltSpeedMultiplier hedge)

## Problems addressed

- `audit/hazards/haste.md` Issue 3 — Design doc hedges between an `ApplyBoltSpeedMultiplier` message and EffectStack reconciliation. Impl chose EffectStack. Design doc should commit.

## Context

Split from the original `haste-ron-and-design-fix.md`. The RON portion (base_percent / per_level_percent value fixes + description copy-paste) is covered by `audit/remediations/ron-tuning-values.md`. This file retains only the design-doc alignment work.

## Remediation

Open `docs/todos/detail/mod-system-design/hazards/haste.md` §Messages and §Systems. Delete the `ApplyBoltSpeedMultiplier` fallback language. Document the EffectStack pattern as canonical:

> **Writes**: Source-tagged `EffectStack<SpeedBoostConfig>` entry on every Bolt (source: `"hazard:haste"`). The entry's multiplier is reconciled each FixedUpdate. Idempotent via `EffectStack::retain_by_source`.
>
> Haste's reconciliation mirrors Erosion's SizeBoost pattern. This is the canonical way for hazards to apply continuous modulation to Bolt components.

No code change. No test change.

## Scope note

This remediation is part of the broader design-doc-alignment sweep.
