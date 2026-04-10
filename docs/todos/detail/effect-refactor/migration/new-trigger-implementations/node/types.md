# Types

## NodeTimerThresholdRegistry (Resource)

Stores registered thresholds for node timer trigger dispatch.

```
NodeTimerThresholdRegistry {
    thresholds: Vec<OrderedFloat<f32>>,
    fired: HashSet<OrderedFloat<f32>>,
}
```

- `thresholds` -- every unique ratio that appears in any effect tree using `NodeTimerThresholdOccurred(ratio)`. Populated during tree installation.
- `fired` -- tracks which thresholds have already fired this node to avoid re-firing. Reset on node start.

## NodeTimerThresholdCrossed (Message)

Sent by the `check_node_timer_thresholds` game system when a threshold is crossed.

```
NodeTimerThresholdCrossed { ratio: f32 }
```
