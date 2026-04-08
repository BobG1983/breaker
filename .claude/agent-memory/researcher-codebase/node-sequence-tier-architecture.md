---
name: node sequence and tier architecture
description: NodeSequence resource, NodeOutcome progression tracker, tier_index on NodeAssignment, advance_node timing
type: project
---

## Key Resources

- **`NodeSequence`** (`state/run/resources/definitions.rs`): flat `Vec<NodeAssignment>` generated once per run `OnExit(MenuState::Main)`. Each `NodeAssignment` has `tier_index: u32`, `hp_mult`, `timer_mult`, `node_type`.
- **`NodeOutcome`** (`state/run/resources/definitions.rs`): runtime progression tracker with `node_index: u32` (incremented by `advance_node`), `result: NodeResult`, `transition_queued: bool`.
- **`DifficultyCurve`** (`state/run/resources/definitions.rs`): RON-loaded resource with `Vec<TierDefinition>` — source data for sequence generation.

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

## current_tier Does Not Exist Yet

`tier_index` is on `NodeAssignment` but is never surfaced to runtime systems. No `current_tier` resource or field exists. To add one, the recommended approach is adding `current_tier: u32` to `NodeOutcome` and updating `advance_node` to populate it from `NodeSequence.assignments[node_index].tier_index`.

**Why:** The protocol/hazard system (mod-system-design) needs to query the current tier for scaling decisions.
**How to apply:** When writing tier-stub spec or implementation, add to `NodeOutcome`, update `advance_node` with `Option<Res<NodeSequence>>`, and use the same fallback pattern.

## State Routing (condensed)

`NodeState::Playing → AnimateOut → Teardown` → condition route checks `NodeState == Teardown`
→ `RunState::Node → resolve_node_next_state(NodeOutcome.result)`:
- `InProgress` → `ChipSelect`
- `Quit` → `Teardown`
- other → `RunEnd`

After ChipSelect teardown → `RunState::Node` → `advance_node` fires again.
