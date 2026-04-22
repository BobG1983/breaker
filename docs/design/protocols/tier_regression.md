# Protocol: Tier Regression

## Category
`custom-system`

## Game Design
You WANT to retreat instead of advance.

Drop back 1 tier of difficulty. Replay the easier tier's nodes for extra chip offerings. Can only appear **once per run after being selected** (matches every other protocol — not "whether taken or not"). In infinite mode: regresses difficulty but keeps the hazard stack.

## Config Resource
None. Tier Regression is a stateless, one-shot mutation — activation emits a payload-free message and the state/run domain owns the tier decrement + node-sequence rewind.

## Components
None.

## Messages
**Reads**: `ProtocolActivated { kind }` — handled by the dispatch pipeline; on `kind == TierRegression`, emits `RegressTier`.
**Sends**: `RegressTier` — payload-free message. State/run domain consumer handles the actual tier decrement, node-sequence rewind, and `NodeOutcome.tier` update. No `TierRegressionPending` snapshot resource, no `TierRegressionConfig` — all state lives in the consumer's domain.

## Systems

### `tier_regression_on_activate`
- **Schedule**: `FixedUpdate`.
- **run_if**: `protocol_active(ProtocolKind::TierRegression)` + `in_state(ChipSelectState::Selecting)` (or on `ProtocolSelected` message — one-shot at selection time).
- **Behavior**: Reads `ProtocolActivated { kind }`. If `kind == TierRegression`: emits payload-free `RegressTier`. That's it — the state/run consumer does the rest.

State/run consumer (owned by state/run, not TierRegression):

### `apply_regress_tier` (state/run domain)
- Reads `RegressTier` message.
- Decrements `NodeOutcome.tier` by 1 (clamped to a minimum tier floor — TBD design: tier 0 or tier 1).
- Resets `NodeOutcome.position_in_tier` to 0.
- Regenerates or rewinds the active tier's nodes (exact mechanism depends on the TODO #15 node-sequencing refactor — generation is deterministic from the seed, so regenerating the previous tier's nodes uses the same inputs).

Tier Regression does NOT write `NodeSequence` or `NodeOutcome` directly — state/run owns both.

## Pipeline position (dmg crate)

- **Not in the death pipeline.** Tier Regression is a protocol-selection / run-state mechanic.
- **Trigger**: `ProtocolActivated { kind: TierRegression }` (protocol-domain message on chip-select commit).
- **Emits**: payload-free `RegressTier` (state/run domain consumer).
- **No** `DamageDealt<T>` / `HealDealt<T>` / `Destroyed<T>` / `DamageBoostStack` / `VulnerableStack` involvement.

## Cross-Domain Dependencies
- **state/run**: Consumes `RegressTier`. Owns `NodeOutcome.tier` / `position_in_tier` decrement + node-sequence rewind.
- **hazard**: In infinite mode, `ActiveHazards` is read-only — hazards persist through regression.
- **protocol**: `ActiveProtocols` gates offering (once-per-run-after-selection).

## Expected Behaviors (for test specs)

1. **Tier index decreases by 1 on activation** — player at tier 5, Tier Regression selected: `ProtocolActivated { kind: TierRegression }` → `RegressTier` emitted → state/run consumer sets `NodeOutcome.tier = 4` and `position_in_tier = 0`.
2. **Can only appear once per run after being selected** — matches every other protocol. If offered but rejected, still eligible for future offerings. Once selected (in `ActiveProtocols`), not offered again.
3. **Regressed nodes still offer chip selections** — standard chip-select UI runs after each regressed node.
4. **Hazard stack persists through regression in infinite mode** — tier 12 with 4 hazards active, regress to tier 11: `ActiveHazards` unchanged.
5. **Cell difficulty matches regressed tier** — regressed nodes use the regressed tier's generation seed + difficulty parameters. (Depends on node-sequencing refactor providing tier-scoped regeneration.)
6. **Protocol activation is immediate** — selecting Tier Regression on chip-select: the next node uses the regressed tier.
7. **Minimum tier floor** — regression from the minimum tier clamps (state/run consumer enforces — either no-op or the protocol was not offered).

## Edge Cases
- **Tier 0 / minimum tier regression**: state/run clamps. Ideally the protocol is not offered when regression has no effect; if it is and the player picks it, the regression is a no-op (wasted pick).
- **Infinite mode tier calculation**: regression decrements and the state/run consumer handles regeneration deterministically from the seed. Depends on the node-sequencing refactor (TODO #15) providing tier-scoped regeneration.
- **Hazard selection interaction**: the player still gets hazard offerings post-regression if tier 9+ — the hazard stack grows regardless of tier regression.
- **Protocol re-offering**: once in `ActiveProtocols`, not offered again. Rejection returns it to the eligible pool.
- **Node-sequence integrity**: regression must not corrupt the sequence. The state/run consumer is authoritative.
- **Return-to-normal after regression**: open design — does the player resume from the tier they were at, or advance from the regressed tier? TBD (tied to TODO #15).
- **Run-seed determinism**: regenerated nodes are deterministic from the seed.
