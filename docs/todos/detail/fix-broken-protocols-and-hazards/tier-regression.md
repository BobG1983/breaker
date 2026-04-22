# Tier Regression

Assumes: #2 (`mutators/protocols/tier_regression/`). No other TODO dependencies.

## What's broken

1. **Cross-domain direct writes.** `tier_regression/system.rs` writes `NodeSequence.assignments` and `NodeOutcome.tier` / `position_in_tier` directly from the protocol domain, violating the message-driven architecture. `plugins.md:154-166` lists this as a documented exception — the exception should not exist.
2. **Unnecessary configuration.** `TierRegressionConfig` carries a `tiers_back` field. Design says the protocol always regresses by exactly 1. The config, its RON field, and its cleanup are all overhead for a zero-tuning mechanic.
3. **Dead snapshot state.** `TierRegressionPending` + `snapshot_pre_advance_state` exist to handle a Boss-boundary edge case that doesn't need them under the simplified handler.
4. **No tier-0 floor documentation or test.** Current behavior works (saturating decrement) but isn't pinned.
5. **No reject-reoffer test.** Issue 1 from the audit: design text suggests "once per run whether taken or not." Impl is correct (once per run AFTER selection, matching every other protocol) but nothing pins it.

## Fix

### Delete `TierRegressionConfig` entirely

`mutators/protocols/tier_regression/system.rs`:
- Delete `TierRegressionConfig` struct + all `Res<TierRegressionConfig>` params.
- Delete the `activate()` function's `insert_resource::<TierRegressionConfig>` call (activation becomes a no-op).

`protocol/definition.rs`:
- `ProtocolTuning::TierRegression { tiers_back }` → `ProtocolTuning::TierRegression` (unit variant).

`assets/protocols/tier_regression.protocol.ron`:
- Delete the `tiers_back` field. File reduces to the minimal shape the RON format requires for a no-tuning protocol.

### Delete `TierRegressionPending` and `snapshot_pre_advance_state`

Both are removed from the codebase. No replacement. The simplified handler operates directly on `NodeOutcome.tier` via saturating decrement — no pre-advance snapshot needed.

### Introduce `RegressTier` message

`state/run/messages.rs`:

```rust
#[derive(Message, Debug, Clone, Default)]
pub struct RegressTier;
```

Payload-free. The receiver hardcodes `saturating_sub(1)` — no field needed. Register via `app.add_message::<RegressTier>()` in the run-state plugin.

### Sender — protocol side

`mutators/protocols/tier_regression/system.rs`:

```rust
pub(crate) fn apply_tier_regression(
    mut writer: MessageWriter<RegressTier>,
    active: Option<Res<ActiveProtocols>>,
    // ...existing one-shot gating params (e.g., a TierRegressionFired marker
    //    resource to prevent re-fire on the same node entry)...
) {
    let Some(active) = active else { return };
    if !active.contains(ProtocolKind::TierRegression) { return }
    // ...existing one-shot gating — fire only on the canonical apply frame...
    writer.write(RegressTier);
}
```

Schedule: `OnEnter(RunState::Node)`, `.after(NodeSystems::AdvanceNode)`. Matches the original apply schedule. No resource mutation.

### Receiver — run-state side

`state/run/systems/apply_regress_tier/system.rs` (new):

```rust
pub(crate) fn apply_regress_tier(
    mut reader: MessageReader<RegressTier>,
    mut outcome: ResMut<NodeOutcome>,
) {
    for _msg in reader.read() {
        outcome.tier = outcome.tier.saturating_sub(1);
    }
}
```

Schedule: `OnEnter(RunState::Node)`, `.after(NodeSystems::AdvanceNode)`, `.after(apply_tier_regression)`. Registered in the run-state plugin.

`saturating_sub(1)` is the tier-0 floor. No branch, no clamp resource, no pre-filter — underflow is impossible.

### Design-doc updates

`docs/design/protocols/tier_regression.md`:
- §Config Resource — DELETE.
- §Activation — "parameterless; just flags the protocol in `ActiveProtocols`. No resource insert."
- §Game Design — "Drop back EXACTLY 1 tier of difficulty. This is not tunable."
- §Cross-Domain Dependencies — "Sends `RegressTier` message to the run-state domain; run-state owns `NodeOutcome.tier` mutation. No direct cross-domain writes."
- §Cleanup — "Tier Regression has no state or config resources to clean."
- §Expected Behaviors 2 — rewrite to "Can only appear once per run AFTER being selected. Rejected offerings remain eligible for future offerings. Matches every other protocol's convention."
- §Expected Behaviors 7 — rewrite as "Minimum tier floor is 0 via `saturating_sub(1)`. Player at tier 0 who selects TR stays at tier 0 — 'wastes' the pick but state remains consistent."
- §Edge Cases — "Infinite-mode tier calculation: pure decrement works identically at any tier value, including procedurally-generated tiers. No special handling for infinite mode."
- §Systems — two-system split: `apply_tier_regression` (protocol, sender) + `apply_regress_tier` (state/run, receiver).

### Delete `plugins.md` exception

`docs/architecture/plugins.md` §Cross-domain writes: find the entry at lines 154-166 documenting the tier_regression direct write. DELETE entirely.

## Tests

`mutators/protocols/tier_regression/tests/sends_regress_tier_message.rs`:

1. **`apply_emits_one_regress_tier_message`** — activate TR; drive the canonical apply frame (`OnEnter(RunState::Node)` after `NodeSystems::AdvanceNode`); assert exactly one `RegressTier` message in the reader.
2. **`apply_without_active_is_noop`** — TR not in `ActiveProtocols`; drive the frame; assert zero messages.

`state/run/systems/apply_regress_tier/tests.rs`:

3. **`decrements_outcome_tier`** — `NodeOutcome { tier: 5 }`; emit one `RegressTier`; tick; assert `tier == 4`.
4. **`saturates_at_zero`** — `NodeOutcome { tier: 0 }`; emit `RegressTier`; tick; assert `tier == 0` (no underflow).
5. **`multiple_messages_compound`** — `NodeOutcome { tier: 3 }`; emit two `RegressTier`; tick; assert `tier == 1`.

`mutators/protocols/tier_regression/tests/reject_reoffers.rs`:

6. **`rejected_tr_stays_in_offer_pool`** — activate protocol-offering system with unlocked pool including TR; generate offerings; assert TR present; simulate REJECT (nothing written to `ActiveProtocols`); generate again; assert TR STILL present.
7. **`selected_tr_removed_from_offer_pool`** — generate offerings; SELECT TR (add to `ActiveProtocols`); generate again; assert TR NOT present (existing `!params.active.contains(*kind)` filter works).

### Tests to DELETE

- `tests/activate.rs` assertions that check `TierRegressionConfig` is inserted — the type is gone.
- Any test asserting on `tiers_back` values (already shouldn't have been there per `ron-asset-tests-not-value-pinned.md`, but if present, remove).
- Any splice-logic test — splice behavior is gone.

## Code changes summary

| File | Change |
|------|--------|
| `mutators/protocols/tier_regression/system.rs` | DELETE `TierRegressionConfig`, `TierRegressionPending`, `snapshot_pre_advance_state`; REWRITE `apply_tier_regression` as a thin `RegressTier` writer |
| `protocol/definition.rs` | `ProtocolTuning::TierRegression { tiers_back }` → unit variant |
| `state/run/messages.rs` | ADD `RegressTier` unit message |
| `state/run/plugin.rs` | Register `RegressTier` + `apply_regress_tier` |
| `state/run/systems/apply_regress_tier/system.rs` | NEW — receiver |
| `state/run/systems/apply_regress_tier/tests.rs` | NEW — tests 3-5 |
| `mutators/protocols/tier_regression/tests/sends_regress_tier_message.rs` | NEW — tests 1-2 |
| `mutators/protocols/tier_regression/tests/reject_reoffers.rs` | NEW — tests 6-7 |
| `assets/protocols/tier_regression.protocol.ron` | DELETE `tiers_back` field |
| `docs/architecture/plugins.md` | DELETE lines 154-166 (TR cross-domain exception) |
| `docs/design/protocols/tier_regression.md` | Design-doc updates per Fix section |

## Out of scope

- Node-sequencing refactor interaction — TR's simple `saturating_sub(1)` works identically before and after #11 (node sequencing refactor). When node sequencing lands, any additional work TR needs to do (splice, regenerate) is added to `apply_regress_tier` in the run-state domain. NOT this TODO's problem.
- Tier-0 gameplay polish (UI indication that TR is "wasted" at tier 0) — Phase 5 HUD work.
