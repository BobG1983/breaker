# Hazard: Decay

## Game Design

Node timer ticks faster. 15%+5%/level. The core pressure is time — the node's countdown accelerates, forcing the player to clear faster or lose. Stacking makes the timer visibly melt away. This creates urgency that interacts with every other hazard: less time to deal with ghosts, regen, fracture debris, etc.

## Config Resource

```rust
#[derive(Resource, Debug, Clone)]
pub(crate) struct DecayConfig {
    /// Base timer speedup percentage at stack 1.
    pub base_percent: f32,
    /// Additional speedup percentage per stack beyond the first.
    pub per_level_percent: f32,
}
```

Extracted from `HazardTuning::Decay { base_percent, per_level_percent }` at activation time.

## Components

None. Decay operates purely on the node timer via messages — no per-entity state needed.

## Messages

**Reads**: None (reads `Time` for delta, `ActiveHazards` for stack count)
**Sends**: `ApplyTimePenalty { seconds: f32 }` — owned by `state/run/node` domain. Each tick sends the extra drain amount.

Note: `ApplyTimePenalty` is listed as an existing message in the interface design. If it does not yet exist, it must be created in the node/timer domain.

## Systems

### `decay_tick`

- **Schedule**: `FixedUpdate`
- **Run condition**: `hazard_active(HazardKind::Decay).and(in_state(NodeState::Playing))`
- **Ordering**: `.after(NodeSystems::TickTimer)` — runs after the normal timer tick so the penalty is additive on top of the base drain
- **Behavior**: Each tick, compute the speedup percentage from config + stack count, multiply by `time.delta_secs()`, and send `ApplyTimePenalty` with the extra seconds to drain.
- **Formula**: `extra_drain = delta_secs * (base_percent + per_level_percent * (stack - 1)) / 100.0`

The receiving system in the node domain subtracts `seconds` from `NodeTimer.remaining`.

## Stacking Behavior

Linear scaling: `speedup = base_percent + per_level_percent * (stack - 1)`

| Stack | Speedup | Timer drain rate |
|-------|---------|-----------------|
| 1 | 15% | 1.15x normal |
| 2 | 20% | 1.20x normal |
| 3 | 25% | 1.25x normal |

Each additional stack adds 5% speedup. At stack 10, the timer runs at 1.60x speed.

## Cross-Domain Dependencies

| Domain | Direction | Message |
|--------|-----------|---------|
| `state/run/node` | sends to | `ApplyTimePenalty` — drains node timer faster |

Decay never reads or writes `NodeTimer` directly. The node domain owns the timer and applies the penalty.

## Expected Behaviors (for test specs)

1. **Timer drains faster at stack 1**
   - Given: Decay active at stack 1, `base_percent=15.0`, `per_level_percent=5.0`, `delta_secs=0.1`
   - When: `decay_tick` runs
   - Then: `ApplyTimePenalty { seconds: 0.015 }` is sent (0.1 * 15.0 / 100.0)

2. **Timer drains faster at stack 3**
   - Given: Decay active at stack 3, same config, `delta_secs=0.1`
   - When: `decay_tick` runs
   - Then: `ApplyTimePenalty { seconds: 0.025 }` is sent (0.1 * 25.0 / 100.0)

3. **System does not run when Decay is inactive**
   - Given: Decay not in `ActiveHazards`
   - When: `FixedUpdate` runs
   - Then: `decay_tick` does not execute (gated by `hazard_active`)

4. **System does not run outside Playing state**
   - Given: Decay active, `NodeState::Paused`
   - When: `FixedUpdate` runs
   - Then: `decay_tick` does not execute (gated by `in_state(NodeState::Playing)`)

5. **Zero delta produces zero penalty**
   - Given: Decay active at stack 1, `delta_secs=0.0`
   - When: `decay_tick` runs
   - Then: `ApplyTimePenalty { seconds: 0.0 }` is sent (or no message — implementation choice)

## Edge Cases

- **Decay + Renewal synergy**: Decay makes the timer tick faster while Renewal regens cells. The player is fighting two clocks simultaneously. No special interaction code needed — the pressure is emergent.
- **Node-end cleanup**: `DecayConfig` resource is removed at run end via `hazards::cleanup()`. No per-entity state to clean up.
- **Max intensity bounds**: Linear scaling has no cap in the formula. At stack 20, speedup is 110% (timer runs at 2.1x). This is intentional — infinite play is meant to become impossible eventually. If a cap is desired, add a `max_percent: f32` field to config.
- **Very short nodes**: If a node starts with a very short timer, Decay's percentage drain is proportional to real time, not remaining time. A 5-second node with stack 1 Decay loses 0.75 extra seconds total — noticeable but not catastrophic.
