# Reckless Dash — `BumpPerformed` in the canonical design doc

## Target file

`docs/design/protocols/reckless_dash.md` (promoted during this sweep).

## What the target doc must say

§ Cross-Domain Dependencies:

> - **breaker domain**: Reads `BumpPerformed` messages. `BumpPerformed` is the graded-bump event (carries grade, bolt, breaker) — it fires only when the player executes a timed bump, not on passive rebounds. Reckless Dash only considers graded catches.
> - **bolt domain**: Reads `BoltImpactCell` and `BoltLost` messages. Reads bolt's `BoltBaseDamage` for amplified-damage computation.
> - **damage crate** (`rantzsoft_dmg`): Writes `DamageBoostStack` one-shot entries (Pattern B) from a risky catch — consumed on the next cell impact.

§ Expected Behaviors — for behaviour 1 ("Risky catch grants damage boost"), clarify under § Edge Cases:

> "Catch" means a graded `BumpPerformed` event (the player executed a timed bump during the risky portion of a dash). A bolt that passively rebounds off the breaker without an active bump window does not grant the boost, even if dash progress is in the risky zone.

## What the target doc must NOT say

- Do not reference `BoltImpactBreaker` as Reckless Dash's trigger.
- Do not mention a `RiskyDamageBoost` component or a two-level anti-feedback guard (both retired).
- Do not describe the damage boost as routed through `EffectStack<DamageBoostConfig>` — post-TODO #1, boosts live in `DamageBoostStack` from `rantzsoft_dmg`.

## Pipeline position (dmg crate)

- **Trigger**: reads `BumpPerformed` (breaker domain) — evaluates dash-risky state and Perfect grade.
- **Writes**: `DamageBoostStack::add_one_shot(multiplier)` on the bolt — Pattern B, consume-on-use.
- **Consumed in**: `DeathPipelineSystems::ApplyDamageBoosts` (crate-owned) — when the next `DamageDealt<Cell>` for that bolt aggregates, the one-shot multiplies in and is cleared.
- **No damage emission** — Reckless Dash does not emit `DamageDealt<T>` itself; it amplifies whatever damage the bolt already emits on its next cell impact.

## Why

`BumpPerformed` is the graded, player-initiated event. Rewarding passive rebounds would undermine the "risky catch" identity. `DamageBoostStack` is the canonical Pattern-B storage post-TODO #1.
