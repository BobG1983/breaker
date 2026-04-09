---
name: node sequence and tier architecture
description: NodeSequence resource, NodeOutcome progression tracker, tier_index on NodeAssignment, advance_node timing
type: project
---

## Key Resources

- **`NodeSequence`** (`state/run/resources/definitions.rs`): flat `Vec<NodeAssignment>` generated once per run `OnExit(MenuState::Main)`. Each `NodeAssignment` has `tier_index: u32`, `timer_mult: f32`, `node_type: NodeType`. NOTE: `hp_mult` was removed — cell HP is now computed from `ToughnessConfig`; `boss_hp_mult` moved to `ToughnessConfig::boss_multiplier`.
- **`NodeOutcome`** (`state/run/resources/definitions.rs`): runtime progression tracker with `node_index: u32`, `result: NodeResult`, `cleared_this_frame: bool`, `tier: u32`, `position_in_tier: u32`. NOTE: `transition_queued: bool` was removed and replaced by `cleared_this_frame: bool`.
- **`DifficultyCurve`** (`state/run/resources/definitions.rs`): RON-loaded resource with `Vec<TierDefinition>` — source data for sequence generation. NOTE: `boss_hp_mult` field removed from `DifficultyCurve`.

## Node Advancement

`advance_node` runs `OnEnter(RunState::Node)` — fires on EVERY entry including the first node.
- Increments `node_index` BEFORE `OnEnter(NodeState::Loading)` systems run.
- So at play time, `node_index` is 1-indexed: first node is index 1, not 0.
- `set_active_layout`, `init_node_timer`, `spawn_cells_from_layout` all read `node_index` to index into `NodeSequence`.

## Tier Lookup Pattern

Both `init_node_timer` and `spawn_cells_from_layout` use this pattern:
```rust
fn resolve_something(run_state: Option<&NodeOutcome>, node_sequence: Option<&NodeSequence>) -> T {
    if let (Some(state), Some(sequence)) = (run_state, node_sequence) {
        sequence.assignments.get(state.node_index as usize).map_or(default, |a| a.field)
    } else { default }
}
```
Falls back to default when NodeSequence is absent (tests/scenarios).

## current_tier / tier Field — NOW EXISTS

`NodeOutcome.tier: u32` and `NodeOutcome.position_in_tier: u32` were added in the Toughness + HP Scaling feature (2026-04-08, commit cd6fb019). `advance_node` now updates both fields:
- After a Boss node clear: `tier += 1`, `position_in_tier = 0`
- After any other node: `position_in_tier += 1`

`advance_node` takes `Option<Res<NodeSequence>>` (same fallback pattern as other systems). The protocol/hazard system can query `NodeOutcome.tier` directly — no further work needed.

## State Routing (condensed)

`NodeState::Playing → AnimateOut → Teardown` → condition route checks `NodeState == Teardown`
→ `RunState::Node → resolve_node_next_state(NodeOutcome.result)`:
- `InProgress` → `ChipSelect`
- `Quit` → `Teardown`
- other → `RunEnd`

After ChipSelect teardown → `RunState::Node` → `advance_node` fires again.
