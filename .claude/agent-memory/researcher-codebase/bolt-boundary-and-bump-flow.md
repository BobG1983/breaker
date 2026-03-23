---
name: bolt-boundary-and-bump-flow
description: System chain map for bolt-lost detection, bump grading, and node completion -- stable architectural flow
type: reference
---

## Bolt Position

- Bolt position: `Transform.translation` (standard Bevy Transform)
- Authoritative physics position: `PhysicsTranslation { previous, current }` (for visual interpolation)
- Velocity: `BoltVelocity { value: Vec2 }` component
- Radius: `BoltRadius(f32)` component, scaled by optional `EntityScale(f32)`

## Bolt-Lost Boundary Detection

System: `bolt_lost` (PhysicsPlugin, FixedUpdate, run_if PlayingState::Active)
Ordering: after `clamp_bolt_to_playfield`, in set `PhysicsSystems::BoltLost`

Boundary check:
```
tf.translation.y < playfield.bottom() - effective_radius
where effective_radius = BoltRadius.0 * EntityScale.map_or(1.0, |s| s.0)
```

With production RON values:
- playfield.bottom() = -height/2 = -1080/2 = -540.0
- BoltRadius = 14.0 (default EntityScale = 1.0)
- Lost threshold = -540.0 - 14.0 = -554.0

Behavior: baseline bolts respawn above breaker; ExtraBolt entities despawn.
Message: `BoltLost` (unit struct, no fields -- no bolt entity reference).

## Bump System Chain

Schedule: FixedUpdate, all run_if PlayingState::Active

1. `prepare_bolt_velocity` (BoltPlugin, in set BoltSystems::PrepareVelocity) -- clamp speed
2. `bolt_cell_collision` (PhysicsPlugin) -- after PrepareVelocity
3. `bolt_breaker_collision` (PhysicsPlugin) -- after bolt_cell_collision, in set PhysicsSystems::BreakerCollision
   Sends: `BoltHitBreaker { bolt: Entity }`
4. `update_bump` (BreakerPlugin, FixedUpdate) -- reads InputActions, ticks timers
   May send: `BumpPerformed { grade: BumpGrade, bolt: Entity }` (retroactive path)
5. `grade_bump` (BreakerPlugin, FixedUpdate, after update_bump) -- reads BoltHitBreaker
   May send: `BumpPerformed { grade, bolt }` (forward path) or `BumpWhiffed`

`BumpPerformed` fields: `grade: BumpGrade` (Early/Perfect/Late), `bolt: Entity`

## Node Completion Chain

1. `bolt_cell_collision` sends `BoltHitCell { cell, bolt }` -> physics writes `DamageCell { cell, damage, source_bolt }`
2. `handle_cell_hit` (CellsPlugin, FixedUpdate) reads `DamageCell`, decrements HP
   Sends: `CellDestroyed { was_required_to_clear: bool }` when HP reaches 0
3. `track_node_completion` (NodePlugin, FixedUpdate) reads `CellDestroyed`
   Decrements `ClearRemainingCount`, sends `NodeCleared` when count hits 0

All in same FixedUpdate frame (message chain within one tick).

## Multiple Bolts

Yes -- `ExtraBolt` marker enables multiple simultaneous bolts (spawned via `spawn_additional_bolt`).
The baseline bolt (without ExtraBolt) always exists; extras despawn on loss.
bolt_lost iterates all active bolts: `Query<LostQuery, ActiveFilter>` where ActiveFilter = (With<Bolt>, Without<BoltServing>).

## Key Coordinates (Production RON)

- Playfield: 1440x1080, centered at origin
- playfield.bottom() = -540.0, playfield.top() = 540.0
- Breaker y_position = -520.0 (20 units above bottom)
- Bolt radius = 14.0
- Bolt lost threshold = -554.0 (bottom - radius)
- Bolt spawn/respawn offset = 54.0 above breaker = y=-466.0
