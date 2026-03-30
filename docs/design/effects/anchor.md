# Anchor

Plant mechanic — when active, braking/stopped breaker gains boosted bump force and a wider perfect bump window.

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `bump_force_multiplier` | `f32` | Multiplier applied to bump force while planted |
| `perfect_window_multiplier` | `f32` | Multiplier applied to perfect bump timing window while planted |

## Behavior

1. `fire()` inserts `AnchorActive { bump_force_multiplier, perfect_window_multiplier }` on the breaker
2. While the breaker is braking or stationary:
   - Bump force is multiplied by `bump_force_multiplier`
   - Perfect bump timing window is multiplied by `perfect_window_multiplier`
3. While the breaker is moving or dashing, the bonuses do not apply
4. Creates a "dash → plant → bump → dash" rhythm

## Reversal

Removes `AnchorActive` from the breaker. Bump force and perfect window return to normal.

## Ingredients

Quick Stop x2 + Bump Force x2.

## VFX

- While planted (braking/stopped) with Anchor active: breaker gains a subtle ground-anchor glow beneath it
- On bump while planted: concentrated impact flash scaled by the boosted force
- The visual communicates "I'm planted and ready" vs "I'm moving"
