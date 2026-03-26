# Run Structure

| Term | Meaning | Code Examples |
|------|---------|---------------|
| **Tier** | A group of nodes in a run sequence sharing the same difficulty parameters (HP multiplier, timer multiplier, active ratio, introduced cell types) | `TierDefinition`, `tier_index` |
| **DifficultyCurve** | The ordered list of tier definitions loaded from `defaults.difficulty.ron`; drives procedural node sequence generation | `DifficultyCurve`, `DifficultyCurveDefaults` |
| **NodeType** | The category of a single node in the run sequence: Passive, Active, or Boss | `NodeType::Passive`, `NodeType::Active`, `NodeType::Boss` |
| **NodePool** | The pool a node layout belongs to — `Passive`, `Active`, or `Boss` — controls which layouts are eligible for each node type | `NodePool`, `NodeLayout.pool` |
| **NodeSequence** | The full ordered list of node assignments generated from the difficulty curve and run seed | `NodeSequence`, `NodeAssignment`, `generate_node_sequence` |
| **TransitionOut** | Game state representing an animated transition out of a completed node (clear animation) | `GameState::TransitionOut`, `TransitionDirection::Out` |
| **TransitionIn** | Game state representing an animated transition into the next node (load animation) | `GameState::TransitionIn`, `TransitionDirection::In` |
| **TransitionStyle** | Visual style of a node transition — `Flash` (full-screen overlay) or `Sweep` (rect sweeps across screen). Picked from seeded `GameRng`. | `TransitionStyle::Flash`, `TransitionStyle::Sweep` |
