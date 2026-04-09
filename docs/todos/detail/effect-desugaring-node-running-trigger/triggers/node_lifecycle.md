# Node Lifecycle Triggers

## Triggers
- `NodeStartOccurred` — a new node started
- `NodeEndOccurred` — the current node ended
- `NodeTimerThresholdOccurred(f32)` — node timer crossed a ratio threshold

## Locality: GLOBAL
All three fire on **ALL entities** with BoundEffects/StagedEffects.

## Participant Enum
None — these triggers have no named participants. `On(...)` is not available.

## Source
- `NodeStartOccurred`: `NodeState` transition to `Playing`
- `NodeEndOccurred`: `NodeState` exit from `Playing`/`Paused`
- `NodeTimerThresholdOccurred(f32)`: node timer system checks ratio thresholds each tick

## Bridge Systems
```
fn bridge_node_start(node_state: Res<State<NodeState>>, ...) {
    // Detect transition to Playing from non-Playing non-Paused
    // Fire NodeStartOccurred on all entities with effects
}

fn bridge_node_end(node_state: Res<State<NodeState>>, ...) {
    // Detect exit from Playing/Paused to non-playing state
    // Fire NodeEndOccurred on all entities with effects
}

fn bridge_node_timer(timer: Res<NodeTimer>, ...) {
    // Track ratio thresholds (0.0 to 1.0)
    // When timer ratio crosses a registered threshold:
    // Fire NodeTimerThresholdOccurred(threshold) on all entities with effects
}
```

## TriggerContext
```rust
TriggerContext::None { depth: 0 }
```
No participant entities — context is empty.

## Notes
- `NodeTimerThresholdOccurred(f32)` uses a ratio (0.0 = start, 1.0 = expired), not seconds
- Bridge systems track state transitions to fire exactly once per transition
- These triggers have no participants, so `On(...)` redirect is not available inside their tree
- Key use case: `Until(NodeEndOccurred, Fire(SpeedBoost(1.5)))` — boost until node ends
