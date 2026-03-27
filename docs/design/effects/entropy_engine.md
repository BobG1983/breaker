# EntropyEngine

Counter-gated random effect — fires a random effect from a pool every Nth trigger.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `threshold` | `u32` | Number of triggers before firing |
| `pool` | `Vec<(f32, EffectNode)>` | Weighted pool of effects |

## Behavior

Maintains a hit counter on the entity. Each trigger increments the counter. When the counter reaches `threshold`, selects and fires a random effect from the pool (weighted), then resets the counter.

## Reversal

No-op. Inner effects handle their own reversal via Until/Reverse nodes.
