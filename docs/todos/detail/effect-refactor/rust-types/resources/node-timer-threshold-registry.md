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
`src/effect/resources/`

# Description
Global resource storing registered node timer thresholds. `thresholds` is populated during tree installation — every unique ratio that appears in any effect tree's `NodeTimerThresholdOccurred(ratio)` is added here. `fired` tracks which thresholds have already fired this node to avoid re-firing. Reset on node start by the bridge.

Checked each frame by `check_node_timer_thresholds`. When the node timer ratio crosses a threshold in `thresholds` that isn't in `fired`, the system sends `NodeTimerThresholdCrossed` and adds the ratio to `fired`.
