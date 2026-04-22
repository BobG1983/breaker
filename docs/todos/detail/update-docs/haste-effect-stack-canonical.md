# Haste — `EffectStack<SpeedBoostConfig>` reconciliation in the canonical design doc

## Target file

`docs/design/hazards/haste.md` (promoted during this sweep).

## What the target doc must say

§ Messages:

> **Writes**: Source-tagged `EffectStack<SpeedBoostConfig>` entry on every Bolt (source: `"hazard:haste"`). The entry's multiplier is reconciled each FixedUpdate. Idempotent via `EffectStack::retain_by_source`.
>
> Haste's reconciliation mirrors Erosion's `SizeBoost` pattern. This is the canonical way for hazards to apply continuous modulation to Bolt components.

§ Systems — the reconciliation system pushes a single source-tagged entry per bolt per tick; the bolt's movement system aggregates the stack.

## What the target doc must NOT say

- Do not reference an `ApplyBoltSpeedMultiplier` message — that hedge is dropped.
- Do not describe Haste as emitting one-off speed-change messages.

## Pipeline position (dmg crate)

- **Not in the death pipeline.** Haste does not participate in any `DeathPipelineSystems` set.
- **Trigger**: FixedUpdate tick (`run_if = hazard_active(Haste) + in_state(NodeState::Playing)`) — reconciliation runs every frame regardless of damage events.
- **Writes**: `EffectStack<SpeedBoostConfig>` reconciled per-tick on every Bolt (source `"hazard:haste"`).
- **No** `DamageDealt<T>` / `HealDealt<T>` / `Destroyed<T>` / `DamageBoostStack` involvement.

## Why

`EffectStack` reconciliation is the canonical pattern for continuous modulation. `SpeedBoostConfig` survives post-TODO #1 (only `DamageBoostConfig` and `VulnerableConfig` retired in favour of `DamageBoostStack` / `VulnerableStack` from `rantzsoft_dmg`).
