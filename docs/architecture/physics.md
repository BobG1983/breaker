# Physics — FixedUpdate + CCD

## Timestep

All physics runs in `FixedUpdate` for deterministic behavior. This is required for seeded run reproducibility — the same seed must produce identical physics across hardware. Visual interpolation smooths rendering between fixed ticks.

## Collision — Swept CCD

Continuous collision detection via ray-vs-expanded-AABB intersection. The bolt's path is traced as a ray each frame; cell and wall AABBs are Minkowski-expanded by the bolt radius so a point-ray test is equivalent to a circle-vs-rectangle test.

- `ray_vs_aabb` in `shared/math.rs` — shared math used by both `bolt_cell_collision` and `bolt_breaker_collision`
- `MAX_BOUNCES` cap prevents infinite loops in degenerate geometries (e.g., bolt trapped between two cells)
- `CCD_EPSILON` separation gap placed after each collision to prevent floating-point re-contact
- On each hit, the bolt is placed just before the impact point, velocity is reflected, and tracing continues with remaining movement distance

## Bolt Reflection

- Direction entirely overwritten on breaker contact — no incoming angle carryover
- Reflection angle determined by: hit position on breaker, breaker tilt state, bump grade
- No perfectly vertical or horizontal reflections — always enforce minimum angle
