# Spatial & Physics (rantzsoft_*)

| Term | Meaning | Code Examples |
|------|---------|---------------|
| **Position2D** | Canonical 2D world-space position component. Game systems read/write `Position2D`; `Transform` is derived and must never be written directly. | `Position2D`, `GlobalPosition2D` |
| **Velocity2D** | 2D velocity component. Entities with `ApplyVelocity` marker have `Position2D` advanced by `Velocity2D` each fixed tick. | `Velocity2D`, `ApplyVelocity` |
| **Spatial2D** | Marker component that auto-inserts all required spatial components. | `Spatial2D`, `#[require(Spatial2D)]` |
| **DrawLayer** | Trait mapping a game-defined enum to a Z value for sprite sorting. | `DrawLayer`, `GameDrawLayer` |
| **GlobalPosition2D** | Resolved world-space position computed from parent/child hierarchy. Read by physics and collision systems. | `GlobalPosition2D`, `GlobalRotation2D`, `GlobalScale2D` |
| **CollisionLayers** | Bitmask pair (`membership`, `mask`) controlling spatial query interaction filtering. | `CollisionLayers`, `rantzsoft_physics2d` |
| **Aabb2D** | Axis-aligned bounding box for collision detection and quadtree indexing. | `Aabb2D`, `CollisionQuadtree` |
| **DistanceConstraint** | Tethered pair of entities with a maximum separation distance. Used by chain bolts. | `DistanceConstraint`, `enforce_distance_constraints` |
| **SpawnChainBolt** | Message requesting spawning a tethered chain bolt. | `SpawnChainBolt`, `bolt/messages.rs` |
