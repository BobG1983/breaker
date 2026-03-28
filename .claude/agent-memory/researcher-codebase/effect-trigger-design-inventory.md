---
name: effect-trigger-design-inventory
description: Complete design-doc-sourced inventory of all effects and triggers with parameters, behaviors, and reversals. Sourced from docs/design/effects/ and docs/design/triggers/ plus chip-catalog.md.
type: project
---

## Triggers (from docs/design/triggers/)

### Bump Triggers
- **PerfectBump** — Global. Perfect-timed bump happened. Sweeps all BoundEffects.
- **PerfectBumped** — Targeted (bolt). "I was perfect bumped."
- **EarlyBump** — Global. Early bump happened.
- **EarlyBumped** — Targeted (bolt). "I was early bumped."
- **LateBump** — Global. Late bump happened.
- **LateBumped** — Targeted (bolt). "I was late bumped."
- **Bump** — Global. Any non-whiff bump (early, late, or perfect).
- **Bumped** — Targeted (bolt). "I was bumped" (any non-whiff).
- **BumpWhiff** — Global. Forward bump window expired without contact.
- **NoBump** — Global. Bolt passed breaker without any bump attempt.

### Impact Triggers
- **Impact(X)** — Global. "There was an impact involving an X." One collision fires Impact(Cell)+Impact(Bolt) both globally.
- **Impacted(X)** — Targeted (both participants). "You were in an impact, X was the other." Bolt gets Impacted(Cell), Cell gets Impacted(Bolt).

### Death / Destruction
- **Death** — Global. Something died.
- **Died** — Targeted (dying entity). "I died."
- **BoltLost** — Global. Bolt fell off screen.
- **CellDestroyed** — Global. A cell was destroyed.

### Node Lifecycle
- **NodeStart** — Global. New node started.
- **NodeEnd** — Global. Current node ended.

### Timer
- **NodeTimerThreshold(f32)** — Global. Node timer ratio dropped below threshold.
- **TimeExpires(f32)** — Special. Timer system ticks Until nodes in StagedEffects; fires Reverse when reaches zero.

---

## Effects (from docs/design/effects/)

### Combat
- **Shockwave** — Expanding ring of area damage. Params: base_range, range_per_level, stacks, speed. Effective range = base_range + (stacks-1)*range_per_level. Reverse: no-op.
- **ChainLightning** — Arc damage jumping between random cells. Params: arcs, range, damage_mult. Reverse: no-op.
- **PiercingBeam** — Fast beam rectangle in velocity direction. Params: damage_mult, width. Reverse: no-op.
- **Pulse** — Bolt emits repeated small rings while active. Params: base_range, range_per_level, stacks, speed. Uses own component types, does NOT reuse Shockwave. Reverse: no-op.
- **Explode** — Instant area damage burst. Params: range, damage_mult. Reverse: no-op (instant, can't undo).
- **TetherBeam** — Two free bolts connected by a beam that damages cells it intersects. Params: damage_mult. Evolution-tier. Reverse: no-op.

### Bolt Spawning
- **SpawnBolts** — Spawn N bolts with optional lifespan and effect inheritance. Params: count (default 1), lifespan (Option), inherit (bool). Reverse: no-op.
- **ChainBolt** — Spawn two bolts with DistanceConstraint tether. Params: tether_distance. Reverse: despawns chain bolts.
- **SpawnPhantom** — Temporary bolt with infinite piercing. Params: duration, max_active. Enforces max cap. Reverse: no-op (self-despawn).

### Stat Modifiers
- **SpeedBoost** — Multiplicative. Pushes to ActiveSpeedBoosts; product applied. Reverse: removes matching entry.
- **DamageBoost** — Multiplicative. Pushes to ActiveDamageBoosts; product applied. Reverse: removes matching entry.
- **Piercing** — Additive. Pushes to ActivePiercings; sum = total. Counted down on cell destroy. Reverse: removes matching count.
- **SizeBoost** — Multiplicative. Pushes to ActiveSizeBoosts; varies by entity type (breaker=width only, bolt=radius, cell=full scale, wall=no-op). Reverse: removes matching entry.
- **BumpForce** — Multiplicative. Pushes to ActiveBumpForces. Reverse: removes matching entry.
- **RampingDamage** — Stacking bonus per impact (any Impacted). Resets on NoBump. No max cap. Reverse: removes RampingDamageState.
- **Attraction** — Steers toward nearest entity of type (Cell, Wall, or Breaker). Deactivates on hit, reactivates on non-attracted bounce. Reverse: removes entry.

### Breaker Modifiers
- **QuickStop** — Multiplicative decel multiplier. Pushes to ActiveQuickStops. Faster reverse on direction flip. Evolution=FlashStep (teleport to bolt X). Reverse: removes matching entry.

### Defensive
- **Shield** — Inserts ShieldActive. On Breaker=bolt-loss immunity; on HP entity=damage immunity. Multiple fires extend duration additively. Reverse: removes ShieldActive.
- **SecondWind** — Invisible wall at bottom. Bounces bolt once then despawns. Reverse: despawns wall.
- **GravityWell** — Spawns well entity; pulls bolts within radius. Enforces max cap. Reverse: no-op (self-despawn).

### Penalties
- **LoseLife** — Decrements LivesCount. Not below zero. Reverse: increments by 1.
- **TimePenalty** — Subtracts seconds from node timer. Reverse: adds seconds back.

### Meta
- **RandomEffect** — Weighted random selection from pool. Fires selected effect. Reverse: no-op.
- **EntropyEngine** — On each CellDestroyed, fires N random effects scaling with kill count up to max_effects. Resets between nodes. Evolution-tier. Reverse: no-op.

---

## Key Design Notes

- All multipliers use **1.x standard**: 2.0 = 2x, 0.5 = half.
- Buff stacking: SpeedBoost/DamageBoost/SizeBoost/BumpForce/QuickStop = multiplicative. Piercing = additive.
- Effects act on the entity they live on (implicit target).
- Design docs live at: docs/design/effects/index.md and docs/design/triggers/index.md
- Architecture docs live at: docs/architecture/effects/index.md
