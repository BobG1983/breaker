# TimePenalty

## Config
```rust
struct TimePenaltyConfig {
    /// Seconds to subtract from node timer
    seconds: f32,
}
```
**RON**: `TimePenalty(seconds: 5.0)`

## Reversible: YES (via reverse message)

## Target: Any (entity-independent — writes global message)

## Fire
1. Write `ApplyTimePenalty { seconds }` message
2. The `apply_time_penalty` system in the node subdomain reads the message and subtracts seconds from the node timer (with clamping and expiry detection)

## Reverse
1. Write `ReverseTimePenalty { seconds }` message
2. The `reverse_time_penalty` system adds time back, clamping to `NodeTimer::total`

## Messages Sent
- **Fire**: `ApplyTimePenalty { seconds: f32 }`
- **Reverse**: `ReverseTimePenalty { seconds: f32 }`

## Notes
- Entity parameter is ignored — the effect operates on the global node timer via messages
- Zero seconds still sends the message (node domain handles clamping)
