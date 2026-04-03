# Slide Transition

Camera translation that slides between two coexisting screens. OneShot only.

## Slide
Camera position eases from the current screen's location to the next screen's location in the given `SlideDirection`. Both screens' content must be spawned and positioned side-by-side in world space before the transition starts.

E.g., `SlideDirection::Left` means the camera moves left — current screen exits to the right, next screen enters from the left.

## Constraints
- **OneShot only** — both screens coexist during the entire transition
- **Game's responsibility** to ensure the destination screen's content is already spawned and positioned at the correct world-space offset before the transition starts
- If content isn't ready, the transition will slide to an empty area — no guardrails
- No overlay, no color — purely a camera move

## Implementation
- **start**: Record current camera `Transform`. Calculate target position based on direction and viewport size.
- **run**: Each frame, sample easing curve → lerp camera position from start to target
- **end**: Camera is at target position. No cleanup needed (camera stays).
- No shader, no overlay entity — just `Transform` manipulation on the camera
- All timing uses `Time<Real>` — virtual time is paused during transitions
- No GlobalZIndex needed — no overlay entity (camera-based effect)
