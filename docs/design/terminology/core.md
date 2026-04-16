# Core Entities & Mechanics

| Term | Meaning | Code Examples |
|------|---------|---------------|
| **Breaker** | The paddle | `Breaker`, `DashState`, `BreakerPlugin` |
| **Bolt** | The ball | `Bolt`, `BaseSpeed`, `BoltLost` |
| **Cell** | A brick | `Cell`, `CellGrid`, `CellDestroyed` |
| **GuardedCell** | A cell that has guardian children orbiting it in a 3×3 ring. The parent is fully damageable — guardians are independent cells that must be destroyed separately. | `GuardedCell`, `GuardianCell`, `CellBehavior::Guarded`, `GuardedBehavior` |
| **GuardianCell** | A child cell that slides clockwise around its guarded parent's ring. Square dimensions. Spawned via `ChildOf` relationship — auto-despawns when parent dies. | `GuardianCell`, `GuardianSlot`, `SlideTarget`, `GuardianSlideSpeed`, `GuardianGridStep` |
| **LockCell** | A cell that is immune to damage until its lock targets are destroyed. Lock targets are defined in the node layout RON (`locks` field), not the cell type definition. | `LockCell`, `Locked`, `Locks`, `Unlocked` |
| **VolatileCell** | A cell that detonates on death, dealing flat AoE damage to every live cell within a fixed radius. Damage and radius are carried on the cell's stamped `BoundEffects` tree — the explosion is dispatched by the existing effect pipeline on `Trigger::Died`, not by a dedicated volatile system. Chain reactions fall out naturally: an explosion that kills another volatile cell fires its tree too. | `VolatileCell`, `CellBehavior::Volatile`, `cells/behaviors/volatile/stamp.rs` |
| **Invulnerable** | A marker component that causes `apply_damage<T>` to skip an entity entirely (`Without<Invulnerable>` filter). Inserted/removed automatically by component hooks on `Locked` cells via `sync_lock_invulnerable`. Any system may insert `Invulnerable` to make an entity immune at all damage sources simultaneously. | `Invulnerable`, `shared/death_pipeline/invulnerable.rs`, `sync_lock_invulnerable` |
| **CellBehavior** | An enum variant attached to a `CellTypeDefinition` that activates runtime behavior at spawn. Current variants: `Regen { rate }`, `Guarded(GuardedBehavior)`, `Volatile { damage, radius }`, `Sequence { group, position }`, `Armored { value, facing }`, `Phantom { solid_secs, ghost_secs, starting_phase }`, `Magnetic { radius, strength }`, `Survival { pattern, timer_secs }`, `SurvivalPermanent { pattern }`, `Portal`. | `CellBehavior`, `CellTypeDefinition.behaviors` |
| **Node** | A level | `Node`, `NodeTimer`, `NodeLayout` |
| **ExtraBolt** | Additional bolt spawned by Prism breaker on a perfect bump; despawned on loss rather than respawned | `ExtraBolt` |
| **ChainBolt** | A bolt entity spawned tethered to an anchor bolt via `DistanceConstraint`. Spawned by the `ChainBolt` effect directly in `chain_bolt::fire()` via `&mut World` | `ChainBoltMarker`, `ChainBoltAnchor`, `ChainBoltConstraint`, `DistanceConstraint` |
| **Birthing** | The brief animation phase after a bolt spawns — scale lerps from zero to full size while collision is disabled. Prevents single-frame cascade explosions and provides visual spawn feedback. | `Birthing`, `begin_node_birthing`, `tick_birthing`, `BIRTHING_DURATION` |
| **SurvivalTurret** | A cell that fires salvos at the breaker on a countdown. Has a survival timer that, once started (on first fire), counts down to self-destruct. Attack pattern determines how many salvos and in what spread. | `SurvivalTurret`, `SurvivalTimer`, `SurvivalPattern`, `AttackPattern`, `CellBehavior::Survival`, `cells/behaviors/survival/` |
| **Salvo** | A projectile entity fired by a SurvivalTurret. Travels in a straight line, damages any cell it overlaps, despawns on contact with the Breaker or on exiting the playfield. Not deflected by the Bolt — Bolt-Salvo contact despawns the salvo. | `Salvo`, `SalvoDamage`, `SalvoSource`, `SalvoFireTimer`, `cells/behaviors/survival/salvo/` |
| **PortalCell** | A cell that is destroyed when the Bolt hits it, triggering a future sub-node transition. Currently implemented as a mock: bolt contact immediately kills the portal cell. | `PortalCell`, `PortalEntered`, `PortalCompleted`, `cells/behaviors/portal/` |
| **Bump** | Breaker's upward hit | `BumpGrade`, `BumpPerformed` |
| **Rig** | The player's complete build (Breaker + Bolt + Chips + seed + score) | `Rig`, `RigSummary` |
