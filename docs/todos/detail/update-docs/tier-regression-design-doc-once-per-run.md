# Tier Regression — "once per run after selection" in the canonical design doc

## Target file

`docs/design/protocols/tier_regression.md` (promoted during this sweep).

## What the target doc must say

§ Expected Behaviors (behaviour 2):

> 2. **Can only appear once per run after being selected** — matches the convention for every other protocol. If Tier Regression is OFFERED but not SELECTED, it remains eligible for future offerings. Once selected (added to `ActiveProtocols`), it will not appear in subsequent offerings.

§ Edge Cases — re-offering entry:

> If the player rejects Tier Regression at an offering, it returns to the pool of eligible offerings. It does NOT get permanently removed after a single appearance.

## What the target doc must NOT say

Do not describe Tier Regression as "once per run whether taken or not" — that contradicts every other protocol's behaviour.

## Pipeline position (dmg crate)

- **Not in the death pipeline.** Tier Regression is a protocol-selection / run-state mechanic — not a damage, heal, or kill mechanic.
- **Trigger**: `ProtocolActivated { kind: TierRegression }` (protocol-domain message) on chip-select commit.
- **Emits**: payload-free `RegressTier` message (per TODO #9 — state/run domain owns the tier decrement + node-sequence rewind).
- **No** `DamageDealt<T>` / `HealDealt<T>` / `Destroyed<T>` / `DamageBoostStack` / `VulnerableStack` involvement.

## Why

Once-per-run-after-selection is the canonical convention. Tier Regression follows it; the design doc must say so.
