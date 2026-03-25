# Core Entities & Mechanics

| Term | Meaning | Code Examples |
|------|---------|---------------|
| **Breaker** | The paddle | `Breaker`, `BreakerState`, `BreakerPlugin` |
| **Bolt** | The ball | `Bolt`, `BoltSpeed`, `BoltLost` |
| **Cell** | A brick | `Cell`, `CellGrid`, `CellDestroyed` |
| **Node** | A level | `Node`, `NodeTimer`, `NodeLayout` |
| **ExtraBolt** | Additional bolt spawned by the Prism archetype on a perfect bump; despawned on loss rather than respawned | `ExtraBolt` |
| **ChainBolt** | A bolt entity spawned tethered to an anchor bolt via `DistanceConstraint`. Spawned by the `ChainHit` effect via `SpawnChainBolt` message | `SpawnChainBolt`, `spawn_chain_bolt`, `break_chain_on_bolt_lost` |
| **Bump** | Breaker's upward hit | `BumpGrade`, `BumpPerformed` |
| **Rig** | The player's complete build (Breaker + Bolt + Chips + seed + score) | `Rig`, `RigSummary` |
