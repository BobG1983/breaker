# Phase 1d: Bump & Perfect Bump System

**Goal**: Timing-based bump mechanic as the core skill expression.

- **Bump**: Player presses bump button → breaker pops upward briefly → any bolt contacting the breaker during the bump receives upward velocity.
- **Bump timing grades**:
  - **Early bump**: Bump pressed too early relative to bolt contact. Reduced velocity transfer.
  - **Perfect bump**: Bump timed within a tight window around bolt impact. Amplified velocity transfer, enhanced trajectory control.
  - **Late bump**: Bump pressed too late. Reduced velocity transfer.
  - **No bump**: Bolt bounces off breaker passively with default reflection.
- Timing window parameters tunable in data (perfect window size, early/late windows, velocity multipliers per grade)

## What actually shipped

Beyond the plan:
- Grade-dependent bump cooldown (perfect = 0, weak = 0.15s)
- Whiff detection with BumpWhiffed message
- Bump visual pop animation with configurable easing (rise/fall phases)
- Base speed floor clamp (weak bumps never drop below base_speed)
- Perfect window widened to 150ms per side
- Bump input buffering to prevent loss when FixedUpdate skips a frame
- Debug UI showing bump state, grade, and timing windows
