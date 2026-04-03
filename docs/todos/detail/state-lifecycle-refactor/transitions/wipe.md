# Wipe Transition

Full-screen overlay where a solid-color edge sweeps across the screen in a direction.

## WipeOut
Leading edge starts off-screen on the `direction` side, sweeps across until the entire screen is covered by the solid color. E.g., `WipeDirection::Left` means the bar enters from the right and sweeps left.

## WipeIn
Leading edge starts covering the full screen, sweeps off-screen in the `direction`. E.g., `WipeDirection::Left` means the bar exits to the left, revealing the screen from right to left.

## WipeOutIn
WipeOut → state change → WipeIn. Duration splits evenly.

## Edge
- Hard edge (no gradient/feathering) — clean geometric wipe
- Edge position eases from 0.0 (fully off-screen) to 1.0 (fully covered) mapped to the easing curve

## Implementation
- **start**: Spawn overlay entity with solid color, sized to full viewport, positioned off-screen in the wipe direction. `GlobalZIndex(i32::MAX - 1)`.
- **run**: Each frame, sample easing curve → lerp overlay position from off-screen to fully covering (or vice versa for In). Re-derive size from camera each frame (viewport resize).
- **end**: Despawn overlay entity
- No shader needed — just sprite position animation using `Transform` translation
- All timing uses `Time<Real>` — virtual time is paused during transitions
- Overlay at `GlobalZIndex(i32::MAX - 1)` — above all game content
- OutIn splits `TransitionConfig.duration` across Out and In phases
