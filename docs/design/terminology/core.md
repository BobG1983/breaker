# Core Entities & Mechanics

| Term | Meaning | Code Examples |
|------|---------|---------------|
| **Breaker** | The paddle | `Breaker`, `DashState`, `BreakerPlugin` |
| **Bolt** | The ball | `Bolt`, `BaseSpeed`, `BoltLost` |
| **Cell** | A brick | `Cell`, `CellGrid`, `CellDestroyed` |
| **Node** | A level | `Node`, `NodeTimer`, `NodeLayout` |
| **ExtraBolt** | Additional bolt spawned by Prism breaker on a perfect bump; despawned on loss rather than respawned | `ExtraBolt` |
| **ChainBolt** | A bolt entity spawned tethered to an anchor bolt via `DistanceConstraint`. Spawned by the `ChainBolt` effect directly in `chain_bolt::fire()` via `&mut World` | `ChainBoltMarker`, `ChainBoltAnchor`, `ChainBoltConstraint`, `DistanceConstraint` |
| **Bump** | Breaker's upward hit | `BumpGrade`, `BumpPerformed` |
| **Rig** | The player's complete build (Breaker + Bolt + Chips + seed + score) | `Rig`, `RigSummary` |
