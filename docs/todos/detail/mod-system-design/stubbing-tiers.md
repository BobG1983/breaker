# Tier Surface for Protocol & Hazard System

**Status**: Already done. No work required.

When the protocol/hazard system needs to query "what tier is the player on?", it reads `NodeOutcome.tier`. The field already exists and is maintained by `advance_node`.

## Current State

`NodeOutcome` (in `breaker-game/src/state/run/resources/definitions.rs`):

```rust
#[derive(Resource, Debug, Clone, Default)]
pub struct NodeOutcome {
    pub node_index:         u32,
    pub result:             NodeResult,
    pub cleared_this_frame: bool,
    /// Current tier in the run (increments after boss clear).
    pub tier:               u32,
    /// Position within the current tier (resets after boss clear).
    pub position_in_tier:   u32,
}
```

`advance_node` (in `breaker-game/src/state/run/systems/advance_node.rs`) increments `tier` when the previous node was a `Boss` and resets `position_in_tier` to 0; otherwise it increments `position_in_tier`. Past the end of the sequence, both fields hold their last values.

This was put in place during the Toughness + HP Scaling work and is already unit-tested (see `advance_node` tests: `advance_from_boss_increments_tier_by_one`, `advance_past_end_of_sequence_holds_tier_and_position`, etc.).

## How Protocol / Hazard Systems Use It

```rust
fn some_protocol_system(outcome: Res<NodeOutcome>) {
    let tier = outcome.tier;
    // Gate eligibility, scale intensity, etc.
}
```

No new resources, no new systems, no new messages. `NodeOutcome` is already read across the codebase.

## Hazard Tier Gating

`HAZARD_TIER_THRESHOLD: u32 = 9` (see `research/interface-design.md` §10 and master detail Wave 6). The `resolve_post_chip_state` dynamic route reads `outcome.tier` and routes to `HazardSelect` when `tier >= 9`. With the current 5-tier difficulty curve, the route never fires; when todo #7 extends the curve, hazards activate automatically with zero code changes here.

## Historical Note

An earlier draft of this document proposed adding a new `current_tier: u32` field to `NodeOutcome` and wiring `advance_node` to populate it from `NodeSequence.assignments[node_index].tier_index`. That work was superseded when `NodeOutcome.tier` + `position_in_tier` were added directly during the Toughness + HP Scaling todo. The two approaches differ in semantics — `tier` is event-driven (incremented on boss clear), while the obsolete `current_tier` would have been index-driven (read from the current assignment). Event-driven is the correct semantics: it accounts for past-end-of-sequence behavior and doesn't drift when a node is regressed or skipped.

## What This Means for the Protocol & Hazard Todo

- **Tier stub prerequisite**: none. Skip the "Wave 0 / tier stub" step in the branch plan. Proceed directly to Wave 1 (plugin infrastructure).
- **Tier Regression (Wave 8)**: when its real body lands (blocked on todo #7), it will mutate `NodeOutcome.tier` directly (or `NodeSequence`, depending on the regression semantics chosen in todo #7).
