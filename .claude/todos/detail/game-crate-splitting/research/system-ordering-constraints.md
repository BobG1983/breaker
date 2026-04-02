# System Ordering Constraints — Cross-Domain Analysis

**Project**: brickbreaker  
**Bevy version**: 0.18.1  
**Scope**: All cross-domain `.before()` / `.after()` / `.in_set()` ordering constraints,
cross-domain component reads/writes, and message flow across domain boundaries.  
**Purpose**: Feasibility analysis for splitting `breaker-game` into sub-crates.

---

## 1. Domain Inventory

The monolith contains these domains (each a Bevy plugin):

| Domain | Plugin | Schedule focus |
|--------|--------|----------------|
| `input` | `InputPlugin` | PreUpdate, FixedPostUpdate |
| `bolt` | `BoltPlugin` | OnEnter(Playing), FixedUpdate, Update |
| `breaker` | `BreakerPlugin` | OnEnter(Playing), FixedUpdate, Update |
| `cells` | `CellsPlugin` | OnEnter(Playing), FixedUpdate |
| `effect` | `EffectPlugin` | OnEnter(PlayingActive), FixedUpdate |
| `chips` | `ChipsPlugin` | Update (ChipSelect state) |
| `run` | `RunPlugin` + `NodePlugin` | OnEnter/OnExit, FixedUpdate, Update |
| `wall` | `WallPlugin` | OnEnter(Playing) |
| `fx` | `FxPlugin` | OnEnter/OnExit state transitions, Update |
| `ui` | `UiPlugin` | OnEnter(Playing), Update |
| `audio` | `AudioPlugin` | stub only |
| `debug` | `DebugPlugin` | dev-feature gated only |
| `screen/*` | sub-plugins | various OnEnter/OnExit |

`shared` is **not a plugin** — it is a passive types module (state enums, collision layers,
`PlayfieldConfig`, `CleanupOnNodeExit`, `GameRng`). Every domain imports from `shared`.

---

## 2. Explicit Cross-Domain Ordering Constraints

All constraints collected from plugin `build()` calls and effect `register()` calls.

### 2.1 BoltPlugin (bolt → breaker, bolt → node/run, bolt → effect, bolt → physics)

**OnEnter(GameState::Playing)**

```
apply_entity_scale_to_bolt
    .after(spawn_bolt)             // intra-domain
    .after(NodeSystems::Spawn)     // CROSS: bolt → run::node
reset_bolt
    .after(spawn_bolt)             // intra-domain
    .after(BreakerSystems::Reset)  // CROSS: bolt → breaker
    .in_set(BoltSystems::Reset)
```

**FixedUpdate**

```
hover_bolt
    .after(BreakerSystems::Move)           // CROSS: bolt → breaker

bolt_cell_collision
    .after(PhysicsSystems::EnforceDistanceConstraints)  // CROSS: bolt → physics2d
    .after(BreakerSystems::Move)                        // CROSS: bolt → breaker
    .after(PhysicsSystems::MaintainQuadtree)            // CROSS: bolt → physics2d
    .in_set(BoltSystems::CellCollision)

bolt_wall_collision
    .after(BoltSystems::CellCollision)   // intra-domain

bolt_breaker_collision
    .after(BoltSystems::CellCollision)   // intra-domain

clamp_bolt_to_playfield
    .after(bolt_breaker_collision)       // intra-domain

bolt_lost
    .after(PhysicsSystems::EnforceDistanceConstraints)  // CROSS: bolt → physics2d
    .after(clamp_bolt_to_playfield)                     // intra-domain
    .in_set(BoltSystems::BoltLost)

tick_bolt_lifespan
    .before(BoltSystems::BoltLost)       // intra-domain

dispatch_bolt_effects
    .before(EffectSystems::Bridge)       // CROSS: bolt → effect

cleanup_destroyed_bolts
    .after(EffectSystems::Bridge)        // CROSS: bolt → effect
```

### 2.2 BreakerPlugin (breaker → bolt, breaker → node/run, breaker → physics)

**OnEnter(GameState::Playing)**

```
spawn_breaker
    → ApplyDeferred
    → init_breaker_params.in_set(BreakerSystems::InitParams)   // chained

init_breaker, dispatch_breaker_effects                          // chained pair
    .after(BreakerSystems::InitParams)
    .after(NodeSystems::Spawn)          // CROSS: breaker → run::node

apply_entity_scale_to_breaker
    .after(BreakerSystems::InitParams)
    .after(NodeSystems::Spawn)          // CROSS: breaker → run::node

reset_breaker
    .after(BreakerSystems::InitParams)
    .in_set(BreakerSystems::Reset)
```

**FixedUpdate**

```
move_breaker
    .after(update_bump)                 // intra-domain
    .in_set(BreakerSystems::Move)

update_breaker_state
    .after(move_breaker)                // intra-domain
    .in_set(BreakerSystems::UpdateState)

grade_bump
    .after(update_bump)                 // intra-domain
    .after(BoltSystems::BreakerCollision)   // CROSS: breaker → bolt
    .in_set(BreakerSystems::GradeBump)

perfect_bump_dash_cancel,
spawn_bump_grade_text,
spawn_whiff_text
    .after(grade_bump)
    .before(BreakerSystems::UpdateState)    // intra-domain

breaker_cell_collision
    .after(BreakerSystems::Move)        // intra-domain

breaker_wall_collision
    .after(BreakerSystems::Move)        // intra-domain
```

### 2.3 CellsPlugin (cells → effect)

**FixedUpdate**

```
cleanup_cell
    .after(EffectSystems::Bridge)       // CROSS: cells → effect
```

### 2.4 RunPlugin (run → breaker, run → node)

**FixedUpdate**

```
handle_node_cleared
    .after(NodeSystems::TrackCompletion)    // CROSS: run → run::node (sub-domain)

handle_timer_expired
    .after(NodeSystems::ApplyTimePenalty)   // CROSS: run → run::node
    .after(handle_node_cleared)             // intra-domain

handle_run_lost
    .after(handle_node_cleared)             // intra-domain
    .after(handle_timer_expired)            // intra-domain

track_node_cleared_stats
    .after(NodeSystems::TrackCompletion)    // CROSS: run → run::node

detect_close_save
    .after(BreakerSystems::GradeBump)       // CROSS: run → breaker

detect_nail_biter
    .after(NodeSystems::TrackCompletion)    // CROSS: run → run::node
```

### 2.5 NodePlugin (run::node)

**FixedUpdate**

```
reverse_time_penalty
    .in_set(NodeSystems::ApplyTimePenalty)
    .after(NodeSystems::TickTimer)
    .before(apply_time_penalty)

apply_time_penalty
    .in_set(NodeSystems::ApplyTimePenalty)
    .after(NodeSystems::TickTimer)
```

### 2.6 EffectPlugin — trigger bridge systems (effect → bolt, effect → breaker, effect → cells, effect → run::node)

All bridge systems run `in_set(EffectSystems::Bridge)` in FixedUpdate with `run_if(in_state(PlayingState::Active))`.

**Impact bridges** (each reads bolt-domain or breaker-domain messages):

```
bridge_impact_bolt_cell
    .after(BoltSystems::CellCollision)      // CROSS: effect → bolt

bridge_impact_bolt_wall
    .after(BoltSystems::CellCollision)      // CROSS: effect → bolt

bridge_impact_bolt_breaker
    .after(BoltSystems::BreakerCollision)   // CROSS: effect → bolt

bridge_impact_breaker_cell     // no explicit ordering (message-driven only)
bridge_impact_breaker_wall     // no explicit ordering
bridge_impact_cell_wall        // no explicit ordering
```

**Impacted bridges** (targeted, same ordering as impact bridges):

```
bridge_impacted_bolt_cell
    .after(BoltSystems::CellCollision)      // CROSS: effect → bolt

bridge_impacted_bolt_wall
    .after(BoltSystems::CellCollision)      // CROSS: effect → bolt

bridge_impacted_bolt_breaker
    .after(BoltSystems::BreakerCollision)   // CROSS: effect → bolt
```

**Bump bridge**:

```
bridge_bump
    .after(BreakerSystems::GradeBump)       // CROSS: effect → breaker
```

**BoltLost bridge**:

```
bridge_bolt_lost
    .after(BoltSystems::BoltLost)           // CROSS: effect → bolt
```

**NodeEnd bridge** (no explicit ordering beyond `in_set(EffectSystems::Bridge)`).

**NodeStart bridge** (`OnEnter(PlayingState::Active)` — no ordering constraint).

### 2.7 Effect runtime systems (effect → physics2d, effect → bolt components)

```
// attraction/effect.rs
apply_attraction
    .after(PhysicsSystems::MaintainQuadtree)    // CROSS: effect → physics2d

// shockwave/effect.rs
tick_shockwave → apply_shockwave_damage → despawn_finished_shockwave (chained)
    .after(PhysicsSystems::MaintainQuadtree)    // CROSS: effect → physics2d

// tether_beam/effect.rs
maintain_tether_chain.before(tick_tether_beam)
tick_tether_beam
    .after(PhysicsSystems::MaintainQuadtree)    // CROSS: effect → physics2d

// chain_lightning/effect.rs (register call)
    .after(PhysicsSystems::MaintainQuadtree)    // CROSS: effect → physics2d (inferred from code structure)

// pulse/effect.rs
    .after(PhysicsSystems::MaintainQuadtree)    // CROSS: effect → physics2d (inferred)

// second_wind/system.rs
despawn_second_wind_on_contact
    .after(BoltSystems::WallCollision)          // CROSS: effect → bolt
```

### 2.8 InputPlugin (no cross-domain ordering — anchors on Bevy-internal `InputSystems`)

```
read_input_actions
    .after(InputSystems)    // Bevy internal — PreUpdate
```

---

## 3. Cross-Domain System Set Summary

The following `SystemSet` types cross domain boundaries (i.e., defined in one domain, referenced in another):

| Set | Defined in | Referenced by |
|-----|-----------|---------------|
| `BoltSystems::CellCollision` | bolt | effect (impact/impacted bridges) |
| `BoltSystems::WallCollision` | bolt | effect (second_wind) |
| `BoltSystems::BreakerCollision` | bolt | breaker (grade_bump), effect (bridges) |
| `BoltSystems::BoltLost` | bolt | effect (bolt_lost bridge) |
| `BoltSystems::Reset` | bolt | breaker (reset_bolt ordering anchor) |
| `BreakerSystems::Move` | breaker | bolt (hover_bolt, bolt_cell_collision) |
| `BreakerSystems::GradeBump` | breaker | run (detect_close_save), effect (bridge_bump) |
| `BreakerSystems::Reset` | breaker | bolt (reset_bolt) |
| `BreakerSystems::InitParams` | breaker | bolt, breaker (multiple OnEnter) |
| `NodeSystems::Spawn` | run::node | bolt, breaker (OnEnter ordering) |
| `NodeSystems::TrackCompletion` | run::node | run (handle_node_cleared etc.) |
| `NodeSystems::ApplyTimePenalty` | run::node | run (handle_timer_expired) |
| `NodeSystems::TickTimer` | run::node | run::node (internal) |
| `EffectSystems::Bridge` | effect | bolt (dispatch_bolt_effects before, cleanup after), cells (cleanup_cell after) |
| `PhysicsSystems::MaintainQuadtree` | rantzsoft_physics2d | bolt, effect (multiple systems) |
| `PhysicsSystems::EnforceDistanceConstraints` | rantzsoft_physics2d | bolt (bolt_cell_collision, bolt_lost) |

**Every system set above would need to be visible to its consumers. In a multi-crate setup, a set defined in crate A and used in crate B requires that A's set types are in a shared dependency of B — or in a separate shared-types crate.**

---

## 4. Cross-Domain Component / Resource Reads and Writes

Systems that access components owned by another domain. Intra-domain access (e.g., a bolt system reading `Bolt`) is excluded.

### 4.1 Bolt domain systems reading other domains' components

| System | File | Foreign component | Domain | Access |
|--------|------|------------------|--------|--------|
| `bolt_breaker_collision` | bolt/systems/bolt_breaker_collision | `CollisionQueryBreaker` (Position2D, BreakerWidth, BreakerHeight, BreakerTilt, MaxAngle, EntityScale, ActiveSizeBoosts) | breaker | read |
| `bolt_cell_collision` | bolt/systems/bolt_cell_collision | `Cell`, `CellHealth` | cells | read |
| `hover_bolt` | bolt/systems/hover_bolt | `Breaker`, `Position2D` (on breaker entity) | breaker | read |
| `dispatch_bolt_effects` | bolt/systems/dispatch_bolt_effects | `Breaker`, `Cell`, `Wall` (marker queries) | breaker, cells, wall | read |
| `reset_bolt` | bolt/systems/reset_bolt | `Breaker` (for spawn-position calculation) | breaker | read |

### 4.2 Breaker domain systems reading other domains' components

| System | File | Foreign component | Domain | Access |
|--------|------|------------------|--------|--------|
| `update_bump` / `grade_bump` | breaker/systems/bump | `BoltServing` (serving sentinel), `BoltImpactBreaker` (message) | bolt | read |
| `tick_anchor` | effect/effects/anchor | `BreakerVelocity`, `BreakerState` | breaker | read (effect reads breaker) |

### 4.3 Effect domain systems reading bolt-domain components

| System | File | Foreign component | Domain | Access |
|--------|------|------------------|--------|--------|
| `apply_gravity_pull` | effect/effects/gravity_well | `Bolt` marker (filter), `ActiveSpeedBoosts` (bolt component), `Velocity2D`+`Position2D` (bolt spatial) | bolt, effect | write Velocity2D |
| `apply_attraction` | effect/effects/attraction | `ActiveAttractions`, `ActiveSpeedBoosts`, `GlobalPosition2D`, `Velocity2D` (on attracted entity) | bolt/effect | write Velocity2D |
| `manage_attraction_types` | effect/effects/attraction | `ActiveAttractions` (on bolt) | bolt | write |
| `shockwave::fire()` | effect/effects/shockwave | `ActiveDamageBoosts` (bolt), `BoltBaseDamage` (bolt) | bolt | read |
| `tick_tether_beam` | effect/effects/tether_beam | `Bolt`, `BoltRadius`, `Position2D` (bolt), `Cell`, `Aabb2D`, `GlobalPosition2D` (cell), `DamageCell` message | bolt, cells | read |
| `maintain_tether_chain` | effect/effects/tether_beam | `Bolt` marker | bolt | read |
| `chain_lightning::fire()` | effect/effects/chain_lightning | `BoltBaseDamage`, `ActiveDamageBoosts`, `Cell`, `DamageCell` message | bolt, cells | read |
| `pulse::apply_damage` | effect/effects/pulse | `BoltBaseDamage`, `ActiveDamageBoosts`, `DamageCell` message | bolt, cells | read |
| `second_wind::despawn_on_contact` | effect/effects/second_wind | `BoltImpactWall` (message), `Wall`, `SecondWindWall` | bolt, wall | read |
| `second_wind::fire()` | effect/effects/second_wind | `Wall`, `WallSize`, `PlayfieldConfig` | wall, shared | read |
| `tick_anchor` | effect/effects/anchor | `BreakerVelocity`, `BreakerState`, `ActiveBumpForces` | breaker, effect | read/write |

### 4.4 Run domain systems reading other domains' components

| System | File | Foreign component | Domain | Access |
|--------|------|------------------|--------|--------|
| `detect_close_save` | run/highlights/systems | `Bolt`, `BoltServing`, `Position2D` (on bolt) | bolt | read |
| `spawn_cells_from_layout` | run/node/systems | all `cells::components::*`, `CellConfig`, `CellTypeRegistry` | cells | read (spawns) |
| `dispatch_cell_effects` (OnEnter) | cells/systems | `Bolt`, `Breaker`, `Wall` (marker queries for target resolution) | bolt, breaker, wall | read |

### 4.5 Chips domain reading all domains

| System | File | Foreign component | Domain | Access |
|--------|------|------------------|--------|--------|
| `dispatch_chip_effects` | chips/systems | `Bolt`, `Breaker`, `Cell`, `Wall` (marker queries for target resolution) | bolt, breaker, cells, wall | read |

---

## 5. Message Flow Across Domain Boundaries

Bevy 0.18 messages (registered via `add_message::<T>()`) that cross domain boundaries. Message registration is the responsibility of the **producing domain's plugin**.

### 5.1 Complete cross-domain message map

| Message | Registered by | Sender(s) | Receivers (cross-domain) |
|---------|--------------|-----------|--------------------------|
| `BoltImpactCell` | BoltPlugin | bolt (bolt_cell_collision) | cells (handle_cell_hit), effect (bridge_impact_bolt_cell, bridge_impacted_bolt_cell), attraction (manage_attraction_types), run (track_cells indirectly via DamageCell) |
| `BoltImpactWall` | BoltPlugin | bolt (bolt_wall_collision) | effect (bridge_impact_bolt_wall, bridge_impacted_bolt_wall, second_wind::despawn_on_contact) |
| `BoltImpactBreaker` | BoltPlugin | bolt (bolt_breaker_collision) | breaker (grade_bump reads it), effect (bridge_impact_bolt_breaker, bridge_impacted_bolt_breaker), attraction (manage_attraction_types) |
| `BoltLost` | BoltPlugin | bolt (bolt_lost) | effect (bridge_bolt_lost), run (track_bolts_lost) |
| `BoltSpawned` | BoltPlugin | bolt (spawn_bolt) | _(no observed cross-domain readers — may be unconsumed)_ |
| `RequestBoltDestroyed` | BoltPlugin | tick_bolt_lifespan, bolt_lost | bolt (cleanup_destroyed_bolts) — appears intra-domain |
| `BumpPerformed` | BreakerPlugin | breaker (update_bump, grade_bump) | effect (bridge_bump), run (track_bumps, detect_close_save, detect_nail_biter indirectly) |
| `BumpWhiffed` | BreakerPlugin | breaker (grade_bump) | effect (bridge_bump_whiff) |
| `BreakerSpawned` | BreakerPlugin | breaker (spawn_breaker) | _(no observed cross-domain readers)_ |
| `BreakerImpactCell` | BreakerPlugin | breaker (breaker_cell_collision) | effect (bridge_impact_breaker_cell, bridge_impacted_breaker_cell) |
| `BreakerImpactWall` | BreakerPlugin | breaker (breaker_wall_collision) | effect (bridge_impact_breaker_wall, bridge_impacted_breaker_wall) |
| `DamageCell` | CellsPlugin | bolt (bolt_cell_collision), effect (shockwave, tether_beam, chain_lightning, pulse, explode, piercing_beam) | cells (handle_cell_hit) |
| `RequestCellDestroyed` | CellsPlugin | cells (handle_cell_hit) | cells (cleanup_cell) — intra-domain |
| `CellDestroyedAt` | NodePlugin (re-registered) | cells (cleanup_cell) | run (track_cells_destroyed), run::node (track_node_completion), effect (bridge_cell_destroyed) |
| `CellImpactWall` | CellsPlugin | cells (cell_wall_collision) | effect (bridge_impact_cell_wall, bridge_impacted_cell_wall) |
| `NodeCleared` | NodePlugin | run::node (track_node_completion) | run (handle_node_cleared, track_node_cleared_stats, detect_nail_biter), effect (bridge_node_end) |
| `TimerExpired` | NodePlugin | run::node (tick_node_timer) | run (handle_timer_expired) |
| `ApplyTimePenalty` | NodePlugin | effect (time_penalty) | run::node (apply_time_penalty) |
| `ReverseTimePenalty` | NodePlugin | effect (time_penalty reverse) | run::node (reverse_time_penalty) |
| `CellsSpawned` | NodePlugin | run::node (check_spawn_complete) | _(no observed cross-domain readers)_ |
| `SpawnNodeComplete` | NodePlugin | run::node (check_spawn_complete) | _(no observed cross-domain readers)_ |
| `ChipSelected` | UiPlugin | ui (chip selection screen) | chips (dispatch_chip_effects), run (track_chips_collected, detect_first_evolution) |
| `WallsSpawned` | WallPlugin | wall (spawn_walls) | _(no observed cross-domain readers)_ |
| `RunLost` | RunPlugin | run (handle_run_lost) | _(no observed cross-domain readers currently)_ |
| `HighlightTriggered` | RunPlugin | run (detect_* systems) | run (spawn_highlight_text) — appears intra-domain |

---

## 6. The Ordering Chain That Matters Most

The central FixedUpdate ordering chain (the one most systems depend on) is:

```
PhysicsSystems::MaintainQuadtree
    ↓
PhysicsSystems::EnforceDistanceConstraints
    ↓
BreakerSystems::Move
    ↓
BoltSystems::CellCollision          ← BoltImpactCell, DamageCell written here
    ↓ (parallel with CellCollision)
BoltSystems::WallCollision          ← BoltImpactWall written here
BoltSystems::BreakerCollision       ← BoltImpactBreaker written here
    ↓ (after BreakerCollision)
BreakerSystems::GradeBump           ← BumpPerformed written here
    ↓
EffectSystems::Bridge               ← all effect chains evaluated here
    ↓
BoltSystems::BoltLost               ← BoltLost written (bolt_lost uses .after(clamp) which uses .after(BreakerCollision))
    ↓ (after BoltLost)
EffectSystems::Bridge               ← bridge_bolt_lost runs after BoltSystems::BoltLost but in same set
```

Note: `EffectSystems::Bridge` is a single set that contains systems with both `.after(BoltSystems::CellCollision)` and `.after(BoltSystems::BoltLost)` — the effect systems within Bridge are not explicitly ordered relative to each other, meaning Bevy is free to run them in any order (they are parallel within the set, all using Commands).

---

## 7. Systems That Query Components From Multiple Domains Simultaneously

These systems are the tightest coupling points. They read components from 2+ domains in a single query.

| System | File | Components from domain A | Components from domain B | Notes |
|--------|------|--------------------------|--------------------------|-------|
| `bolt_cell_collision` | bolt/systems | `BoltCollisionData` (bolt), `ActiveFilter` (bolt) | `Cell`, `CellHealth` (cells), `ActiveDamageBoosts` (effect) | The most complex cross-domain query |
| `bolt_breaker_collision` | bolt/systems | `BoltCollisionData` (bolt) | `CollisionQueryBreaker` (breaker), `ActiveSizeBoosts` (effect) | bolt reads breaker's entire surface model |
| `dispatch_bolt_effects` | bolt/systems | `BoltDefinitionRef` (bolt), `BoltRegistry` (bolt) | `Breaker`, `Cell`, `Wall` (marker queries) | reads 3 other domains |
| `dispatch_chip_effects` | chips/systems | `ChipInventory`, `ChipCatalog` (chips), `ChipSelected` msg (ui) | `Bolt`, `Breaker`, `Cell`, `Wall` (queries) | reads 4 other domains |
| `dispatch_cell_effects` | cells/systems | `Cell`, `CellTypeRegistry` (cells) | `Bolt`, `Breaker`, `Wall` (queries) | reads 3 other domains |
| `apply_gravity_pull` | effect/effects/gravity_well | `GravityWellConfig` (effect) | `Bolt` filter + `ActiveSpeedBoosts` + `SpatialData` (bolt/effect) | writes Velocity2D on bolt entities |
| `apply_attraction` | effect/effects/attraction | `ActiveAttractions` (effect) | `ActiveSpeedBoosts` (effect), `GlobalPosition2D` (spatial), `CollisionQuadtree` (physics2d) | writes Velocity2D on any attracted entity |
| `tick_tether_beam` | effect/effects/tether_beam | `TetherBeamComponent` (effect) | `Bolt`, `BoltRadius`, `Position2D` (bolt), `Cell`, `Aabb2D`, `GlobalPosition2D` (cells) | reads 3 domains |
| `tick_anchor` | effect/effects/anchor | `AnchorActive`, `AnchorTimer`, `AnchorPlanted`, `ActiveBumpForces` (effect) | `BreakerVelocity`, `BreakerState` (breaker) | effect reads breaker movement state |
| `detect_close_save` | run/highlights | `RunStats`, `RunState`, `HighlightConfig` (run) | `Bolt`, `Position2D` on bolt, `BoltServing` (bolt), `BumpPerformed` msg (breaker) | run observes bolt spatial state |
| `hover_bolt` | bolt/systems | `BoltSpawnOffsetY`, `Position2D` (bolt) | `Breaker`, `Position2D` on breaker (breaker) | bolt reads breaker position directly |
| `update_bump` / `grade_bump` | breaker/systems | `BreakerState`, `BumpTimingQuery` (breaker) | `BoltServing` (bolt), `BoltImpactBreaker` msg (bolt) | breaker checks bolt serving state |
| `second_wind::fire()` | effect/effects | effect components | `Wall`, `WallSize` (wall), `PlayfieldConfig` (shared) | effect spawns wall entities |

---

## 8. Ordering Constraints Hardest to Maintain Across Crate Boundaries

Ranked from most to least difficult.

### 8.1 CRITICAL: `EffectSystems::Bridge` anchors

`EffectSystems::Bridge` is defined in `effect::sets`. Systems in **three other domains** reference it for ordering:

- `bolt::dispatch_bolt_effects` — `.before(EffectSystems::Bridge)`
- `bolt::cleanup_destroyed_bolts` — `.after(EffectSystems::Bridge)`
- `cells::cleanup_cell` — `.after(EffectSystems::Bridge)`

If `effect` becomes a separate crate, then `bolt` and `cells` would need a **compile-time dependency on effect** just to reference `EffectSystems::Bridge`. This would create a circular dependency: `effect` reads bolt and cells messages, and bolt/cells depend on effect's sets.

**Resolution options**: Extract `EffectSystems` into a separate `breaker-effect-sets` crate that all domains can depend on without introducing circular deps. Or keep `effect` in the same crate as `bolt`/`cells`.

### 8.2 CRITICAL: `BoltSystems::*` referenced by effect and breaker

`BoltSystems::CellCollision`, `BoltSystems::BreakerCollision`, `BoltSystems::WallCollision`, `BoltSystems::BoltLost`, `BoltSystems::Reset` are defined in `bolt::sets` but referenced in:

- `effect` (10+ trigger bridge systems)
- `breaker` (`grade_bump.after(BoltSystems::BreakerCollision)`)
- `effect/second_wind` (`.after(BoltSystems::WallCollision)`)

If `bolt` is a separate crate, `effect` and `breaker` need a compile-time dep on `bolt` for these sets. Since `bolt` already reads `BreakerSystems::Move` and `NodeSystems::Spawn`, this would create a **circular dependency** unless system sets are extracted.

### 8.3 CRITICAL: `BreakerSystems::*` referenced by bolt, run, and effect

`BreakerSystems::Move`, `BreakerSystems::GradeBump`, `BreakerSystems::Reset`, `BreakerSystems::InitParams` are used in `bolt`, `run`, and `effect` for ordering. Same circular dependency problem.

### 8.4 HIGH: `NodeSystems::Spawn` referenced by bolt and breaker (OnEnter)

`NodeSystems::Spawn` is defined in `run::node::sets`. Both `bolt` and `breaker` use it in their `OnEnter(GameState::Playing)` registrations. This creates: `run::node` is depended on by `bolt` and `breaker`.

### 8.5 HIGH: `PhysicsSystems::*` referenced by bolt and effect

`PhysicsSystems::EnforceDistanceConstraints` and `PhysicsSystems::MaintainQuadtree` are defined in `rantzsoft_physics2d`. Both `bolt` and `effect` reference them. These sets are already in a separate crate, so this constraint is **already handled** — they would remain a dependency of both.

### 8.6 MEDIUM: `NodeSystems::TrackCompletion` and `NodeSystems::ApplyTimePenalty` (run → node)

`RunPlugin` references `NodeSystems::TrackCompletion` and `NodeSystems::ApplyTimePenalty`. Since `RunPlugin` and `NodePlugin` are both in `run`, this is already an intra-crate relationship. If `run` is split from `node`, it becomes the same problem.

---

## 9. Components That Would Need To Live In Shared Dependencies

If splitting, these types are used across what would be multiple crates and cannot live in a leaf crate:

| Type | Currently in | Needed by |
|------|-------------|-----------|
| `BoltSystems` (SystemSet enum) | bolt | breaker, effect, effect/second_wind |
| `BreakerSystems` (SystemSet enum) | breaker | bolt, effect/anchor, run |
| `NodeSystems` (SystemSet enum) | run::node | bolt, breaker, run |
| `EffectSystems` (SystemSet enum) | effect | bolt, cells |
| `Bolt` (marker component) | bolt | effect, chips, cells, run |
| `Breaker` (marker component) | breaker | bolt, effect, chips, cells, run |
| `Cell` (marker component) | cells | bolt, effect, chips, run::node |
| `Wall` (marker component) | wall | bolt, effect, chips |
| `BoltImpactCell`, `BoltImpactWall`, `BoltImpactBreaker`, `BoltLost` (messages) | bolt | effect, breaker, run, attraction |
| `BumpPerformed`, `BumpWhiffed`, `BreakerImpactCell`, `BreakerImpactWall` (messages) | breaker | effect, run |
| `DamageCell` (message) | cells | bolt, effect (multiple) |
| `CellDestroyedAt` (message) | cells, re-registered NodePlugin | run, run::node, effect |
| `NodeCleared` (message) | run::node | run, effect |
| `BreakerVelocity`, `BreakerState` (components) | breaker | effect/anchor |
| `ActiveDamageBoosts`, `ActiveSpeedBoosts` (components) | effect | bolt, effect/shockwave, effect/tether_beam |
| `BoltBaseDamage` (component) | bolt | effect (shockwave, tether_beam, chain_lightning, pulse) |
| `BoltRadius` (component) | bolt | effect/tether_beam |
| `ChipSelected` (message) | ui | chips, run |

---

## 10. Overall Feasibility Assessment

### What makes splitting hard

1. **Circular ordering dependency**: `bolt` ↔ `breaker` ↔ `effect` is a triangle. Each references the other's system sets for ordering. Bevy's scheduling requires set types to be visible at registration time. This is the single hardest constraint.

2. **Marker components scattered everywhere**: `Bolt`, `Breaker`, `Cell`, `Wall` are used as query filters in 5+ other domains. They would need to live in a very thin shared types crate, not in their respective domain crates.

3. **Messages registered by the producing domain**: `BoltImpactCell` is registered in `BoltPlugin::build()` but read by `cells`, `effect`, and `run`. In Bevy 0.18, `add_message::<T>()` only needs to happen once, but all consumers need the type visible at compile time.

4. **Effect runtime systems reach deeply into bolt/breaker/cells**: `apply_gravity_pull`, `apply_attraction`, `tick_tether_beam`, `tick_anchor`, etc. directly query bolt spatial components, breaker movement state, and cell AABBs. `effect` cannot be a separate crate without depending on `bolt`, `breaker`, and `cells`.

5. **`spawn_cells_from_layout` is in `run::node` but uses all of `cells::components::*`**: The run domain physically constructs cell entities using the full cells component set. This is a strong compile-time dependency: `run::node` → `cells`.

### What would be cleanly separable

- `input` — fully isolated, no cross-domain ordering references
- `fx` — only depends on `shared` (GameState, PlayingState), no physics or game entity queries
- `ui` — registers `ChipSelected`; depends on `shared` states only
- `audio` — currently a stub, no cross-domain coupling
- `screen/*` — pure state machine / UI, no game entity queries
- `debug` — dev-feature-gated, no scheduling constraints
- `wall` — spawn-only, no FixedUpdate systems; one message (`WallsSpawned`) with no observed readers

### Viable splitting boundary

If splitting, the natural boundary is a **core-game crate** containing:
- `shared` (already passive types only)
- `bolt`, `breaker`, `cells`, `wall` — tightly coupled via component queries and system set ordering
- `effect` — deeply entangled with bolt/breaker/cells

And separate crates for:
- `input`, `ui`, `audio`, `fx`, `screen`, `debug` — presentation layer, message-driven in

`run` sits in the middle: it needs `cells::components` to spawn cells, and references `BreakerSystems::GradeBump` for one highlight detection system. Separating it would require extracting marker types and system sets into shared packages.

---

## 11. Known Issue: Stale Ordering Anchors

As noted in stable memory, `apply_gravity_pull` and `apply_attraction` previously used `.before(BoltSystems::PrepareVelocity)` for ordering. `BoltSystems::PrepareVelocity` was eliminated in the `feature/chip-evolution-ecosystem` merge. Current ordering of these two effect systems relative to collision sets is **implicit only** — they run without an ordering anchor to `BoltSystems::CellCollision` or `BoltSystems::BreakerCollision`. This is a pre-existing ambiguity regardless of crate splitting.
