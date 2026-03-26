---
name: spatial2d-propagation-flow
description: End-to-end spatial2d propagation pipeline -- save_previous, propagate_{position,rotation,scale}, interpolation, Absolute/Relative hierarchy, orbit cell sync
type: reference
---

## Schedule Order (Bevy 0.18)

1. **FixedFirst** -- `save_previous` snapshots Position2D/Rotation2D/Scale2D into Previous* components (only for entities with InterpolateTransform2D marker)
2. **FixedUpdate** -- game logic (physics, collisions, orbit rotation, orbit sync) mutates Position2D/Rotation2D/Scale2D
3. **RunFixedMainLoop / AfterFixedMainLoop** -- `propagate_position`, `propagate_rotation`, `propagate_scale` (chained) write to Transform from the 2D components, using Time<Fixed>.overstep_fraction() for interpolation

## Propagation Pipeline

Each propagate system:
1. Reads the current 2D component (Position2D, Rotation2D, Scale2D)
2. If InterpolateTransform2D marker present AND Previous* exists: lerp(previous, current, alpha)
3. If entity is a child with *Propagation::Absolute: counteract parent's value (subtract position, subtract rotation angle, divide scale)
4. Write to Transform (position also uses DrawLayer::z() and optional VisualOffset)

## Absolute Propagation Hack

For children with Absolute propagation, the system pre-counteracts the parent's value so that when Bevy's built-in TransformPropagate runs (which always adds parent transform to child), the net result is the child's world-space value equals its 2D component directly.

Example: parent Position2D(10,0), child Position2D(5,0) with Absolute:
- propagate_position writes child Transform.translation.x = 5.0 - 10.0 = -5.0
- Bevy TransformPropagate: GlobalTransform.x = parent(10) + child(-5) = 5.0

## Orbit Cells (game-specific, NOT in spatial2d crate)

- `rotate_shield_cells` (FixedUpdate): increments OrbitAngle by speed * dt
- `sync_orbit_cell_positions` (FixedUpdate): writes world-space Position2D = parent_pos + radius * (cos(angle), sin(angle))
- Orbit cells use PositionPropagation::Absolute so the quadtree sees correct world coords
- **Neither system is currently registered in CellsPlugin** -- they exist only in their test modules

## Who Uses InterpolateTransform2D

- Bolt: yes (#[require] in Bolt component)
- Breaker: yes (#[require] in Breaker component)
- Cells: NO (static entities, no interpolation)
- Walls: NO (static entities)

## Key Limitation

Position2D serves dual purpose: it's both the "local" position that gameplay writes and the value propagated to Transform. For orbit cells, sync_orbit_cell_positions manually computes world-space Position2D, bypassing the hierarchy. This works only because orbit cells have PositionPropagation::Absolute which tells propagate_position to counteract the parent.
