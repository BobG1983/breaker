# Anchor

Plant mechanic — after the breaker remains stationary for a delay period, it becomes "planted" with boosted bump force and a wider perfect bump window. Moving or dashing cancels the planted state.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `bump_force_multiplier` | `f32` | Multiplier applied to bump force while planted |
| `perfect_window_multiplier` | `f32` | Multiplier applied to perfect bump timing window while planted |
| `plant_delay` | `f32` | Seconds the breaker must be stationary before becoming planted |

## Components

| Component | Description |
|-----------|-------------|
| `AnchorActive` | Config inserted by fire() — stores multipliers and plant_delay |
| `AnchorTimer(f32)` | Countdown timer — inserted when breaker stops/starts settling, removed on movement |
| `AnchorPlanted` | Marker inserted when timer reaches zero — bump system reads this for multipliers |

## Behavior

1. `fire()` inserts `AnchorActive { bump_force_multiplier, perfect_window_multiplier, plant_delay }` on the breaker
2. `tick_anchor` system (FixedUpdate) watches breaker movement state:
   - When breaker stops moving or starts settling from a dash → insert `AnchorTimer(plant_delay)`
   - While stationary: tick down `AnchorTimer` by dt
   - When `AnchorTimer` reaches zero → remove timer, insert `AnchorPlanted`
   - When breaker moves or dashes → remove `AnchorTimer` and `AnchorPlanted`
3. While `AnchorPlanted` is present:
   - Bump force is multiplied by `bump_force_multiplier`
   - Perfect bump timing window is multiplied by `perfect_window_multiplier`
4. Creates a "dash → stop → wait → plant → bump → dash" rhythm — the delay adds commitment tension

## Reversal

Removes `AnchorActive`, `AnchorTimer`, and `AnchorPlanted` from the breaker. All bonuses and timer state cleared.

## Ingredients

Quick Stop x2 + Bump Force x2.

## VFX

- While `AnchorTimer` is counting down: subtle charging glow beneath breaker (building anticipation)
- When `AnchorPlanted` activates: ground-anchor glow locks in with a brief flash
- On bump while planted: concentrated impact flash scaled by the boosted force
- On movement (cancelling plant): anchor glow dissipates
- The visual communicates three states: "charging", "planted and ready", "moving/not anchored"
