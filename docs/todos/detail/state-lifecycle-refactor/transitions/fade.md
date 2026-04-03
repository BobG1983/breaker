# Fade Transition

Full-screen overlay sprite at `GlobalZIndex(i32::MAX - 1)`.

## FadeOut
Overlay starts at `Color::with_alpha(color, 0.0)`, eases to `Color::with_alpha(color, 1.0)` over duration. Screen is fully covered at the end.

## FadeIn
Overlay starts at `Color::with_alpha(color, 1.0)`, eases to `Color::with_alpha(color, 0.0)` over duration. Despawn overlay when done.

## FadeOutIn
FadeOut → state change → FadeIn. Duration splits evenly between out and in phases.

## Implementation
- **start**: Spawn overlay entity (sprite, full viewport size, `GlobalZIndex(i32::MAX - 1)`)
- **run**: Each frame, sample easing curve at `elapsed / duration`, set overlay alpha. Re-derive size from camera each frame (viewport resize).
- **end**: Despawn overlay entity
- No shader needed — just sprite color alpha
- All timing uses `Time<Real>` — virtual time is paused during transitions
- Overlay at `GlobalZIndex(i32::MAX - 1)` — above all game content
- OutIn splits `TransitionConfig.duration` across Out and In phases
