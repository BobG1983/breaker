---
name: Toughness + HP Scaling patterns
description: HpScale precomputation in spawn_cells_from_layout; advance_node tier/position tracking — scheduling and access patterns confirmed
type: project
---

## spawn_cells_from_layout (OnEnter(NodeState::Loading))

- `ToughnessConfig` is accessed as `Option<Res<'w, ToughnessConfig>>` inside `CellSpawnContext` SystemParam.
- `HpScale::from_context()` calls `tier_scale()` once — one `powi()` + one `mul_add()` — before iterating the grid. The result is stored as a `Copy` struct on `GridCellContext`. Per-cell cost is two f32 multiplies only.
- All allocations in this system (HashMap, HashSet, VecDeque for lock toposort) are spawn-time, not per-frame.
- Runs exactly once per node load — never on a hot path.

## advance_node (OnEnter(RunState::Node))

- Takes `ResMut<NodeOutcome>` + `Option<Res<NodeSequence>>`. One `.get()` on a `Vec<NodeAssignment>` (array index). No allocations.
- Runs on state transition only — not per-frame, not per-update. Cost is negligible.

## CellSpawnContext SystemParam access

- Six resources: CellConfig, PlayfieldConfig, CellTypeRegistry, Option<NodeOutcome>, Option<NodeSequence>, Option<ToughnessConfig>.
- All immutable except NodeOutcome/NodeSequence (both Option<Res>, not ResMut). No write conflicts.
- Runs in OnEnter — not in a hot path, no parallelism concern.

## Verdict

All patterns are correct and efficient. Precomputing HpScale once per spawn batch (rather than per-cell) is the right call. No issues at current or projected scale.
