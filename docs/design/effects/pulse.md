# Pulse

Timed effect — bolt emits repeated small shockwave-like rings while active.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `base_range` | `f32` | Base radius per pulse ring |
| `range_per_level` | `f32` | Extra radius per stack |
| `stacks` | `u32` | Stack count |
| `speed` | `f32` | Expansion speed in world units/sec |
| `interval` | `f32` | Seconds between ring emissions (default 0.5) |

## Behavior

For a duration, the bolt emits small expanding rings at its current position at regular intervals controlled by `interval`. Each ring expands outward (faster expansion, lower damage than Shockwave) and damages cells it passes, each cell only once per ring. The bolt continues moving while pulsing. Uses its own component types — does NOT reuse Shockwave types.

## Reversal

No-op. Pulse ring entities self-despawn on their own lifecycle.
