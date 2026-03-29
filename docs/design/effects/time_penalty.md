# TimePenalty

Subtracts time from the node timer.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `seconds` | `f32` | Penalty duration in seconds |

## Behavior

Sends `ApplyTimePenalty { seconds }` message. The `apply_time_penalty` system in the node subdomain reads the message and subtracts `seconds` from `NodeTimer::remaining`, clamping to 0.0 minimum.

## Reversal

Sends `ReverseTimePenalty { seconds }` message. The `reverse_time_penalty` system in the node subdomain reads the message and adds `seconds` back to `NodeTimer::remaining`, clamping to `NodeTimer::total`.
