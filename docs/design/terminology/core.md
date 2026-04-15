# Core Entities & Mechanics

| Term | Meaning | Code Examples |
|------|---------|---------------|
| **Breaker** | The paddle | `Breaker`, `DashState`, `BreakerPlugin` |
| **Bolt** | The ball | `Bolt`, `BaseSpeed`, `BoltLost` |
| **Cell** | A brick | `Cell`, `CellGrid`, `CellDestroyed` |
| **GuardedCell** | A cell that has guardian children orbiting it in a 3×3 ring. The parent is fully damageable — guardians are independent cells that must be destroyed separately. | `GuardedCell`, `GuardianCell`, `CellBehavior::Guarded`, `GuardedBehavior` |
| **GuardianCell** | A child cell that slides clockwise around its guarded parent's ring. Square dimensions. Spawned via `ChildOf` relationship — auto-despawns when parent dies. | `GuardianCell`, `GuardianSlot`, `SlideTarget`, `GuardianSlideSpeed`, `GuardianGridStep` |
| **LockCell** | A cell that is immune to damage until its lock targets are destroyed. Lock targets are defined in the node layout RON (`locks` field), not the cell type definition. | `LockCell`, `Locked`, `Locks`, `Unlocked` |
| **Invulnerable** | A marker component that causes `apply_damage<T>` to skip an entity entirely (`Without<Invulnerable>` filter). Inserted/removed automatically by component hooks on `Locked` cells via `sync_lock_invulnerable`. Any system may insert `Invulnerable` to make an entity immune at all damage sources simultaneously. | `Invulnerable`, `shared/death_pipeline/invulnerable.rs`, `sync_lock_invulnerable` |
| **CellBehavior** | An enum variant attached to a `CellTypeDefinition` that activates runtime behavior at spawn. Current variants: `Regen { rate }`, `Guarded(GuardedBehavior)`. | `CellBehavior`, `CellTypeDefinition.behaviors` |
| **Node** | A level | `Node`, `NodeTimer`, `NodeLayout` |
| **ExtraBolt** | Additional bolt spawned by Prism breaker on a perfect bump; despawned on loss rather than respawned | `ExtraBolt` |
| **ChainBolt** | A bolt entity spawned tethered to an anchor bolt via `DistanceConstraint`. Spawned by the `ChainBolt` effect directly in `chain_bolt::fire()` via `&mut World` | `ChainBoltMarker`, `ChainBoltAnchor`, `ChainBoltConstraint`, `DistanceConstraint` |
| **Birthing** | The brief animation phase after a bolt spawns — scale lerps from zero to full size while collision is disabled. Prevents single-frame cascade explosions and provides visual spawn feedback. | `Birthing`, `begin_node_birthing`, `tick_birthing`, `BIRTHING_DURATION` |
| **Bump** | Breaker's upward hit | `BumpGrade`, `BumpPerformed` |
| **Rig** | The player's complete build (Breaker + Bolt + Chips + seed + score) | `Rig`, `RigSummary` |
