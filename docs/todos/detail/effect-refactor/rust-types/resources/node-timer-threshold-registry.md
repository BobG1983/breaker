# Name
NodeTimerThresholdRegistry

# Struct
```rust
#[derive(Resource, Default)]
struct NodeTimerThresholdRegistry {
    thresholds: Vec<OrderedFloat<f32>>,
    fired: HashSet<OrderedFloat<f32>>,
}
```

# Location
`src/effect/triggers/node/resources.rs`

# Description
Global resource storing registered node timer thresholds. `thresholds` is populated by a dedicated scan system that runs after tree installation — it scans all BoundEffects on all entities for `NodeTimerThresholdOccurred(ratio)` trigger variants and collects every unique ratio. `fired` tracks which thresholds have already fired this node to avoid re-firing. Reset on `OnEnter(NodeState::Playing)` by `reset_node_timer_thresholds`.

Checked each frame by `check_node_timer_thresholds`. When the node timer ratio crosses a threshold in `thresholds` that isn't in `fired`, the system sends `NodeTimerThresholdCrossed` and adds the ratio to `fired`.
