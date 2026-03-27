# TimePenalty

Subtracts time from the node timer.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `seconds` | `f32` | Penalty duration in seconds |

## Behavior

Subtracts `seconds` from the current node timer.

## Reversal

No-op. Time penalty is fire-and-forget.
