# Phase 1a: Breaker System

**Goal**: Responsive breaker movement with dash/brake/settle state machine and tilt mechanics.

- **Horizontal movement**: Smooth, responsive left/right movement. This is the primary movement mode.
- **Bump**: Button press causes the breaker to jump upward slightly, transferring vertical velocity to the bolt. This is the core skill mechanic for controlling bolt trajectory.
- **Dash**: Burst of horizontal speed. Breaker tilts in the direction of movement during dash, which affects bolt bounce angles. Cannot initiate a new dash while one is ongoing — must brake and settle first.
- **Brake**: Rapid deceleration from dash. Breaker tilts back hard in the opposite direction, providing another angle-control window.
- **Settle**: Return to neutral state after braking. Breaker tilt returns to flat. Only after settling can you dash again.
- **Breaker state machine**: Idle → Dashing → Braking → Settling → Idle (with normal movement available in Idle and Settling states)
- Breaker-specific parameters (breaker width, speed, bump strength, tilt angles, reflection behavior) defined in data
- Breaker hitbox and visual representation (placeholder shader rectangle, breaker-tinted, tilt visible)

## What actually shipped

Beyond the plan:
- Eased tilt for all phases (QuadraticInOut dash, CubicInOut brake, CubicOut settle)
- Eased deceleration curve during braking (speed-dependent via configurable ease function)
- Config-driven entity components (not resource reads at runtime)
- Action-based input domain for reliable FixedUpdate input
- Bump race condition fixes (breaker overlap collision)
- Frame-rate independent tilt interpolation

## Deferred to later phases

- Breaker as composable identity (Guardian/Chrono archetypes with unique bolt-lost behavior)
- Breaker-specific unique abilities
