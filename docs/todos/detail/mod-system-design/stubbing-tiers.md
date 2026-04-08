# Stubbing Tiers for Protocol & Hazard System

The protocol/hazard system needs to query "what tier is the player on?" The full tier system (per-tier batching, frame/block generation) comes later in the node sequencing refactor. This document specifies the minimal stub needed to unblock protocol/hazard implementation.

Based on [tier-stub research](research/tier-stub-trace.md).

## Key Finding: `tier_index` Already Exists

Every `NodeAssignment` in `NodeSequence` already carries `tier_index: u32`. It's populated correctly by `generate_node_sequence_system` at run start. **No system reads it after generation.** We just need to surface it.

## What to Change (3 files)

### 1. Add `current_tier` to `NodeOutcome`

**File**: `breaker-game/src/state/run/resources/definitions.rs`

```rust
pub struct NodeOutcome {
    pub node_index: u32,
    pub result: NodeResult,
    pub transition_queued: bool,
    pub current_tier: u32,   // ← new field
}
```

`NodeOutcome` derives `Default` — `current_tier` defaults to `0`. No change to `reset_run_state` needed (it does `*run_state = NodeOutcome::default()` which zeros everything).

### 2. Set `current_tier` in `advance_node`

**File**: `breaker-game/src/state/run/systems/advance_node.rs`

Add `Option<Res<NodeSequence>>` parameter. After incrementing `node_index`, look up the tier:

```rust
pub(crate) fn advance_node(
    mut run_state: ResMut<NodeOutcome>,
    node_sequence: Option<Res<NodeSequence>>,
) {
    run_state.node_index += 1;
    run_state.transition_queued = false;
    // Surface the tier from the existing NodeAssignment
    run_state.current_tier = node_sequence
        .and_then(|seq| seq.assignments.get(run_state.node_index as usize))
        .map_or(0, |a| a.tier_index);
}
```

This follows the exact pattern used by `init_node_timer` and `spawn_cells_from_layout` for reading `NodeSequence` assignments.

### 3. No change to `reset_run_state`

**File**: `breaker-game/src/state/run/loading/systems/reset_run_state.rs`

Already does `*run_state = NodeOutcome::default()` — zeroes `current_tier` automatically.

## How Protocol/Hazard Systems Use It

Any system that needs the current tier just reads `Res<NodeOutcome>`:

```rust
fn some_protocol_system(
    run_state: Res<NodeOutcome>,
) {
    let tier = run_state.current_tier;
    // Use tier for hazard selection, protocol eligibility, etc.
}
```

No new resources, no new systems, no new messages. The tier is always up-to-date because `advance_node` runs on `OnEnter(RunState::Node)` before any `NodeState::Loading` systems.

## Edge Cases

| Scenario | Behavior |
|----------|----------|
| `NodeSequence` absent (tests, scenarios without sequence) | `Option<Res<NodeSequence>>` → falls back to tier 0 |
| `node_index` out of bounds (scenario runner cycling) | `.get()` returns `None` → falls back to tier 0 |
| Boss node | `tier_index` stays at the boss's tier — correct for scaling |
| First node | `advance_node` increments to `node_index = 1`, so `current_tier = assignments[1].tier_index` — consistent with how `hp_mult`/`timer_mult` already work |

## Tests

- `advance_node` tests need `Option<Res<NodeSequence>>` — using `Option` means existing tests compile without inserting a sequence (they'll just get tier 0)
- Add a test: insert `NodeSequence` with known tier assignments, verify `current_tier` updates after `advance_node`
- Add a test: no `NodeSequence` → `current_tier` stays 0

## What This Unlocks

With `current_tier` on `NodeOutcome`, the protocol/hazard system can:
- Offer hazards at tier boundaries (detect `current_tier` change)
- Scale hazard intensity by tier
- Gate protocol eligibility by tier
- Implement tier regression (modify `current_tier` directly, or replay a lower tier's sequence)

The full node sequencing refactor later replaces the flat `NodeSequence` with per-tier generation, upgrades `NodeOutcome` to the full `RunProgress` resource, and adds `TierConfig`. This stub is forward-compatible — `current_tier` on `NodeOutcome` becomes `RunProgress.tier` in the refactor.
