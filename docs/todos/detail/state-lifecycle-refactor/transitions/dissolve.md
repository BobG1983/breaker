# Dissolve Transition

Full-screen overlay with a fragment shader that uses procedural noise to dissolve pixels in/out.

## DissolveOut
Threshold eases from 0.0 to 1.0. Each pixel compares its noise value against the threshold — pixels with noise below threshold become the solid overlay color, rest remain transparent. At threshold 1.0 the screen is fully covered.

## DissolveIn
Threshold eases from 1.0 to 0.0. Reverse of DissolveOut — solid pixels dissolve away to reveal the screen beneath.

## DissolveOutIn
DissolveOut → state change → DissolveIn. Duration splits evenly.

## Noise
- Procedural in the fragment shader — no texture asset needed
- Simplex noise or value noise, 2D, at screen-space UV coordinates
- Scale tuned so the pattern is visible but not chunky (~8-12 cells across the screen width)
- Seed randomized per transition instance (pass as uniform) so consecutive dissolves look different
- Small edge glow/gradient at the dissolve boundary (1-2px soft edge where noise ≈ threshold) — smoothstep around the threshold value in the shader

## Implementation
- **start**: Spawn overlay entity with custom `DissolveMaterial` (shader material), pass color + seed as uniforms. `GlobalZIndex(i32::MAX - 1)`.
- **run**: Each frame, sample easing curve → set `threshold` uniform. Re-derive size from camera each frame (viewport resize).
- **end**: Despawn overlay entity
- Requires a fragment shader (`dissolve.wgsl`)
- All timing uses `Time<Real>` — virtual time is paused during transitions
- Overlay at `GlobalZIndex(i32::MAX - 1)` — above all game content
- OutIn splits `TransitionConfig.duration` across Out and In phases
