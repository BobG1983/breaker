# Iris Transition

Circular mask that expands or shrinks from a point, revealing or covering the screen.

## IrisOut
Circle starts at full-screen radius (everything visible through the hole), shrinks to zero radius at the origin point. Area outside the circle is the solid overlay color. At zero radius the screen is fully covered.

## IrisIn
Circle starts at zero radius (fully covered by overlay color), expands to full-screen radius. Screen is revealed through the growing circle.

## IrisOutIn
IrisOut → state change → IrisIn. Duration splits evenly.

## Origin
- `IrisOrigin::Center` — circle centered on viewport center
- `IrisOrigin::Position(Vec2)` — circle centered on a world position (e.g., bolt position at time of transition trigger). Converted to screen-space UV in the shader.

## Radius
- Maximum radius: distance from origin to the farthest viewport corner (ensures full coverage)
- Easing curve maps to radius interpolation between 0 and max

## Implementation
- **start**: Spawn overlay entity with custom `IrisMaterial` (shader material), pass color + origin (screen-space UV) as uniforms. `GlobalZIndex(i32::MAX - 1)`.
- **run**: Each frame, sample easing curve → set `radius` uniform (normalized 0.0–1.0, shader scales to actual pixel distance). Re-derive size from camera each frame (viewport resize).
- **end**: Despawn overlay entity
- Requires a fragment shader (`iris.wgsl`) — discard/alpha pixels inside the circle, solid color outside
- All timing uses `Time<Real>` — virtual time is paused during transitions
- Overlay at `GlobalZIndex(i32::MAX - 1)` — above all game content
- OutIn splits `TransitionConfig.duration` across Out and In phases
