# Name
NodeTimerThresholdCrossed

# Struct
```rust
#[derive(Message, Clone, Debug)]
struct NodeTimerThresholdCrossed {
    ratio: OrderedFloat<f32>,
}
```

# Location
`src/effect_v3/triggers/node/messages.rs`

# Description
Sent by `check_node_timer_thresholds` when the node timer ratio crosses a registered threshold. Read by the `on_node_timer_threshold_occurred` bridge which dispatches `NodeTimerThresholdOccurred(ratio)` globally.
