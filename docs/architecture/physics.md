# Physics — FixedUpdate + Quadtree

## Timestep

All physics runs in `FixedUpdate` for deterministic behavior. This is required for seeded run reproducibility — the same seed must produce identical physics across hardware. Visual interpolation smooths rendering between fixed ticks.

## Collision — Quadtree

- Persistent quadtree `Resource` that entities insert into on spawn, update on move, and remove from on despawn
- Handles both static cell grids and moving cells (active nodes, Phase 6+)
- Bolt-vs-cell, bolt-vs-breaker, bolt-vs-wall queries through the quadtree

## Bolt Reflection

- Direction entirely overwritten on breaker contact — no incoming angle carryover
- Reflection angle determined by: hit position on breaker, breaker tilt state, bump grade
- No perfectly vertical or horizontal reflections — always enforce minimum angle
