---
name: spatial2d-propagation-flow
description: End-to-end spatial2d propagation pipeline -- save_previous, compute_globals, derive_transform, interpolation, Absolute/Relative hierarchy, orbit cell sync
type: reference
---

## Schedule Order (Bevy 0.18)

1. **FixedFirst** -- `save_previous` (SpatialSystems::SavePrevious) snapshots Position2D/Rotation2D/Scale2D into Previous* components (only for entities with InterpolateTransform2D marker)
2. **FixedUpdate** -- `apply_velocity` (SpatialSystems::ApplyVelocity) advances Position2D for ApplyVelocity entities. Game logic (physics, collisions, orbit rotation, orbit sync) also mutates Position2D/Rotation2D/Scale2D.
3. **RunFixedMainLoop / AfterFixedMainLoop** -- `compute_globals` (SpatialSystems::ComputeGlobals) then `derive_transform` (SpatialSystems::DeriveTransform) chained. Write Global* components then write Transform from Global* + interpolation.

NOTE: `propagate_position`, `propagate_rotation`, `propagate_scale` are NOT registered by `RantzSpatial2dPlugin` — they are pub(crate) internal utilities. The active pipeline is compute_globals + derive_transform.

## Propagation Pipeline

`compute_globals`:
1. Walks hierarchy from root to children
2. Accumulates Global* from parent Global* and child's local Position2D/Rotation2D/Scale2D
3. Respects *Propagation::Relative vs *Propagation::Absolute per axis

`derive_transform`:
1. Reads GlobalPosition2D/GlobalRotation2D/GlobalScale2D
2. If InterpolateTransform2D marker present: lerp(Previous*, Global*, overstep_fraction)
3. Writes Transform.translation (also adds DrawLayer::z() and optional VisualOffset.x/y), Transform.rotation, Transform.scale

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
