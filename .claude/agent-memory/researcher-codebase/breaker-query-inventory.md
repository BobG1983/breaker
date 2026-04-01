---
name: breaker-query-inventory
description: Complete inventory of every system that queries Breaker entities — component reads/writes, filters, groupings, and QueryData candidates
type: reference
---

# Breaker Query Inventory

Bevy 0.18. Branch: feature/chip-evolution-ecosystem.

## 1. Existing `QueryData` types in `breaker/queries.rs`

All are type aliases (`type X = (...)`) except `BumpTelemetryQuery` which is also a type alias
but gated on `#[cfg(feature = "dev")]`. No `#[derive(QueryData)]` structs yet in the breaker
domain — all are tuple aliases.

| Alias | Components | Mutability | Consumer |
|---|---|---|---|
| `CollisionQueryBreaker` | `Position2D`, `BreakerTilt`, `BreakerWidth`, `BreakerHeight`, `BreakerReflectionSpread`, `Option<ActiveSizeBoosts>`, `Option<EntityScale>` | all read | `bolt_breaker_collision` |
| `MovementQuery` | `mut Position2D`, `mut BreakerVelocity`, `BreakerState`, `MaxSpeed`, `BreakerAcceleration`, `BreakerDeceleration`, `DecelEasing`, `BreakerWidth`, `Option<ActiveSpeedBoosts>`, `Option<ActiveSizeBoosts>` | mut pos + vel | `move_breaker` |
| `DashQuery` | group 1: `mut BreakerState`, `mut BreakerVelocity`, `mut BreakerTilt`, `mut BreakerStateTimer`, `MaxSpeed`, `BreakerDeceleration`, `DecelEasing`, `DashSpeedMultiplier`, `DashDuration`, `DashTilt`, `DashTiltEase`, `BrakeTilt`, `BrakeDecel`, `SettleDuration`, `SettleTiltEase`; group 2: `Option<FlashStepActive>`, `Option<mut Position2D>`, `Option<BreakerWidth>`, `Option<ActiveSpeedBoosts>`, `Option<ActiveSizeBoosts>` | mut most | `update_breaker_state` |
| `ResetQuery` | `mut Position2D`, `mut BreakerState`, `mut BreakerVelocity`, `mut BreakerTilt`, `mut BreakerStateTimer`, `mut BumpState`, `BreakerBaseY`, `Option<mut PreviousPosition>` | mut all writable | `reset_breaker` |
| `BumpTimingQuery` | `mut BumpState`, `BumpPerfectWindow`, `BumpEarlyWindow`, `BumpLateWindow`, `BumpPerfectCooldown`, `BumpWeakCooldown`, `Option<AnchorPlanted>`, `Option<AnchorActive>` | mut BumpState | `update_bump` |
| `BumpGradingQuery` | `mut BumpState`, `BumpPerfectWindow`, `BumpLateWindow`, `BumpPerfectCooldown`, `BumpWeakCooldown`, `Option<AnchorPlanted>`, `Option<AnchorActive>` | mut BumpState | `grade_bump` |
| `WidthBoostVisualQuery` | `BreakerWidth`, `Option<ActiveSizeBoosts>`, `BreakerHeight`, `Option<EntityScale>`, `mut Scale2D` | mut Scale2D | `width_boost_visual` |
| `BumpTelemetryQuery` *(dev)* | `BreakerState`, `BumpState`, `BreakerTilt`, `BreakerVelocity`, `BumpPerfectWindow`, `BumpEarlyWindow`, `BumpLateWindow` | all read | `breaker_state_ui` |

Filters defined in `breaker/filters.rs`:

| Filter alias | Definition |
|---|---|
| `BumpTriggerFilter` | `(With<Breaker>, Without<BumpVisual>)` |
| `CollisionFilterBreaker` | `(With<Breaker>, Without<Bolt>)` |

---

## 2. Systems inside `breaker/` that query Breaker entities

### `move_breaker` — `breaker/systems/move_breaker/system.rs`

- **Schedule**: `FixedUpdate`
- **Filter**: `With<Breaker>`
- **Query**: `MovementQuery` (mut `Position2D`, mut `BreakerVelocity`, `BreakerState`, `MaxSpeed`, `BreakerAcceleration`, `BreakerDeceleration`, `DecelEasing`, `BreakerWidth`, `Option<ActiveSpeedBoosts>`, `Option<ActiveSizeBoosts>`)
- **Resources read**: `InputActions`, `PlayfieldConfig`, `Time<Fixed>`
- **Writes**: `Position2D.x`, `BreakerVelocity.x`
- **Notes**: Movement only in `Idle | Settling` states. Clamps position to playfield minus effective half-width.

### `update_breaker_state` — `breaker/systems/dash/system.rs`

- **Schedule**: `FixedUpdate`
- **Filter**: `With<Breaker>`
- **Query**: `DashQuery` (nested tuple — all dash state machine components + optional flash-step/position/width/boosts)
- **Resources read**: `InputActions`, `PlayfieldConfig`, `Time<Fixed>`
- **Writes**: `BreakerState`, `BreakerVelocity`, `BreakerTilt`, `BreakerStateTimer`, `Position2D` (flash-step only)
- **Notes**: Handles `Idle → Dashing → Braking → Settling → Idle` state machine. Flash-step teleport only when `FlashStepActive` is present in `Settling`.

### `update_bump` — `breaker/systems/bump/system.rs`

- **Schedule**: `FixedUpdate`
- **Filter**: `With<Breaker>`
- **Query**: `BumpTimingQuery` (mut `BumpState`, 5 window/cooldown params, optional anchor)
- **Resources read**: `InputActions`, `Time<Fixed>`
- **Messages read**: `Query<(), With<BoltServing>>` (to suppress bump press during serve)
- **Messages written**: `BumpPerformed`
- **Writes**: `BumpState` timers, cooldown, `active` flag

### `grade_bump` — `breaker/systems/bump/system.rs`

- **Schedule**: `FixedUpdate`
- **Filter**: `With<Breaker>` (via `single_mut()`)
- **Query**: `BumpGradingQuery` (mut `BumpState`, `BumpPerfectWindow`, `BumpLateWindow`, `BumpPerfectCooldown`, `BumpWeakCooldown`, optional anchor)
- **Messages read**: `BoltImpactBreaker`
- **Messages written**: `BumpPerformed`, `BumpWhiffed`
- **Resource read**: `Option<Res<ForceBumpGrade>>`
- **Writes**: `BumpState` fields on hit (retroactive path)

### `perfect_bump_dash_cancel` — `breaker/systems/bump/system.rs`

- **Schedule**: `FixedUpdate` (after `GradeBump`)
- **Filter**: `With<Breaker>`
- **Query**: `(&mut BreakerState, &mut BreakerStateTimer, &SettleDuration)`
- **Messages read**: `BumpPerformed`
- **Writes**: `BreakerState → Settling`, `BreakerStateTimer.remaining` when grade is `Perfect` and state is `Dashing`

### `animate_tilt_visual` — `breaker/systems/tilt_visual.rs`

- **Schedule**: `Update`
- **Filter**: `With<Breaker>`
- **Query**: `(&BreakerTilt, &mut Rotation2D)`
- **Writes**: `Rotation2D` (mirrors `-BreakerTilt.angle`)

### `width_boost_visual` — `breaker/systems/width_boost_visual.rs`

- **Schedule**: `FixedUpdate`
- **Filter**: `With<Breaker>`
- **Query**: `WidthBoostVisualQuery` (`BreakerWidth`, `Option<ActiveSizeBoosts>`, `BreakerHeight`, `Option<EntityScale>`, `mut Scale2D`)
- **Writes**: `Scale2D.x` (effective width), `Scale2D.y` (height × entity_scale)

### `trigger_bump_visual` — `breaker/systems/bump_visual/system.rs`

- **Schedule**: `Update`
- **Filter**: `BumpTriggerFilter = (With<Breaker>, Without<BumpVisual>)`
- **Query**: `(Entity, &BumpVisualParams)`
- **Resources read**: `InputActions`
- **Writes**: inserts `BumpVisual` component via `commands` on bump press

### `animate_bump_visual` — `breaker/systems/bump_visual/system.rs`

- **Schedule**: `Update`
- **Filter**: `With<Breaker>`
- **Query**: `(Entity, &mut Position2D, &mut BumpVisual, &BreakerBaseY, &BumpVisualParams)`
- **Resources read**: `Time`
- **Writes**: `Position2D.y` (eased Y offset), removes `BumpVisual` when complete

### `spawn_bump_grade_text` — `breaker/systems/bump_feedback.rs`

- **Schedule**: `FixedUpdate`
- **Filter**: `With<Breaker>`
- **Query**: `&Transform` (single)
- **Messages read**: `BumpPerformed`
- **Writes**: spawns `Text2d` entity near breaker (world-space transform)
- **Notes**: reads Bevy `Transform` not `Position2D` — relies on the interpolation system to have committed the world transform

### `spawn_whiff_text` — `breaker/systems/bump_feedback.rs`

- **Schedule**: `FixedUpdate`
- **Filter**: `With<Breaker>`
- **Query**: `&Transform` (single)
- **Messages read**: `BumpWhiffed`
- **Writes**: spawns `Text2d` entity near breaker

### `reset_breaker` — `breaker/systems/spawn_breaker/system.rs`

- **Schedule**: `OnEnter(GameState::Playing)`
- **Filter**: `With<Breaker>`
- **Query**: `ResetQuery` (mut `Position2D`, `BreakerState`, `BreakerVelocity`, `BreakerTilt`, `BreakerStateTimer`, `BumpState`, `BreakerBaseY`, `Option<mut PreviousPosition>`)
- **Resources read**: `PlayfieldConfig`
- **Writes**: all mutable state components to cleared/centered values

### `spawn_breaker` — `breaker/systems/spawn_breaker/system.rs`

- **Schedule**: `OnEnter(GameState::Playing)`
- **Filter**: `With<Breaker>` (existence check only — `existing.is_empty()`)
- **Writes**: spawns new breaker entity with full component bundle when none exists

### `init_breaker_params` — `breaker/systems/init_breaker_params.rs`

- **Schedule**: `OnEnter(GameState::Playing)`
- **Filter**: `(With<Breaker>, Without<MaxSpeed>)` — skips already-initialized
- **Query**: `Entity` only
- **Resources read**: `BreakerConfig`
- **Writes**: inserts `BreakerWidth`, `BreakerHeight`, `BreakerBaseY`, `MaxSpeed`, `BreakerAcceleration`, `BreakerDeceleration`, `DecelEasing`, `DashSpeedMultiplier`, `DashDuration`, `DashTilt`, `DashTiltEase`, `BrakeTilt`, `BrakeDecel`, `BreakerReflectionSpread`, `SettleDuration`, `SettleTiltEase`, `BumpPerfectWindow`, `BumpEarlyWindow`, `BumpLateWindow`, `BumpPerfectCooldown`, `BumpWeakCooldown`, `BumpVisualParams`

### `init_breaker` — `breaker/systems/init_breaker/system.rs`

- **Schedule**: `OnEnter(GameState::Playing)` (after `init_breaker_params`)
- **Filter**: `(With<Breaker>, Without<BreakerInitialized>)`
- **Query**: `Entity` only
- **Writes**: inserts `BreakerInitialized`, optionally `LivesCount` from registry

### `apply_entity_scale_to_breaker` — `breaker/systems/apply_entity_scale_to_breaker.rs`

- **Schedule**: `OnEnter(GameState::Playing)`
- **Filter**: `With<Breaker>`
- **Query**: `Entity` only
- **Resources read**: `Option<Res<ActiveNodeLayout>>`
- **Writes**: inserts/overwrites `EntityScale` from `ActiveNodeLayout.entity_scale`

### `dispatch_breaker_effects` — `breaker/systems/dispatch_breaker_effects/system.rs`

- **Schedule**: `OnEnter(GameState::Playing)` (wiring phase)
- **Filter (breaker)**: `With<Breaker>` — entity list only
- **Also queries**: `With<Bolt>`, `With<Cell>`, `With<Wall>` for target routing
- **Resources read**: `SelectedBreaker`, `BreakerRegistry`
- **Writes**: `commands.fire_effect()` and `commands.push_bound_effects()` on target entities

### `breaker_cell_collision` — `breaker/systems/breaker_cell_collision.rs`

- **Schedule**: `FixedUpdate` (after `PhysicsSystems::MaintainQuadtree`)
- **Filter**: `With<Breaker>` (single)
- **Query (breaker)**: `(Entity, &Position2D, &BreakerWidth, &BreakerHeight, Option<&EntityScale>)`
- **Query (cell)**: `(&Position2D, &Aabb2D)` with `With<Cell>`
- **Resources read**: `CollisionQuadtree`
- **Messages written**: `BreakerImpactCell { breaker, cell }`

### `breaker_wall_collision` — `breaker/systems/breaker_wall_collision.rs`

- **Schedule**: `FixedUpdate` (after `PhysicsSystems::MaintainQuadtree`)
- **Filter**: `With<Breaker>` (single)
- **Query (breaker)**: `(Entity, &Position2D, &BreakerWidth, &BreakerHeight, Option<&EntityScale>)`
- **Query (wall)**: `(&Position2D, &Aabb2D)` with `With<Wall>`
- **Resources read**: `CollisionQuadtree`
- **Messages written**: `BreakerImpactWall { breaker, wall }`

---

## 3. Systems OUTSIDE `breaker/` that query Breaker entities

### `hover_bolt` — `bolt/systems/hover_bolt.rs`

- **Schedule**: `FixedUpdate`
- **Filter (breaker)**: `(With<Breaker>, Without<Bolt>)`
- **Query (breaker)**: `&Position2D` (single)
- **Writes**: bolt `Position2D` to track breaker x

### `bolt_lost` — `bolt/systems/bolt_lost/system.rs`

- **Schedule**: `FixedUpdate`
- **Filter (breaker)**: `CollisionFilterBreaker = (With<Breaker>, Without<Bolt>)` (single_mut)
- **Query (breaker)**: `(Entity, &Position2D, Option<&mut ShieldActive>)`
- **Writes**: bolt respawn position from `breaker_position.0`, decrements `ShieldActive.charges`, removes `ShieldActive` from breaker if charges reach 0

### `bolt_breaker_collision` — `bolt/systems/bolt_breaker_collision/system.rs`

- **Schedule**: `FixedUpdate` (CCD phase)
- **Filter (breaker)**: `CollisionFilterBreaker = (With<Breaker>, Without<Bolt>)` (single)
- **Query (breaker)**: `(Entity, CollisionQueryBreaker)` = `(Entity, &Position2D, &BreakerTilt, &BreakerWidth, &BreakerHeight, &BreakerReflectionSpread, Option<&ActiveSizeBoosts>, Option<&EntityScale>)`
- **Resources read**: `CollisionQuadtree`, `Time<Fixed>`
- **Messages written**: `BoltImpactBreaker { bolt, breaker }`
- **Writes** (on bolt): `Position2D`, `Velocity2D`, inserts `LastImpact`

### `draw_velocity_vectors` *(dev)* — `debug/overlays/systems/draw_velocity_vectors.rs`

- **Schedule**: `Update`
- **Filter (breaker)**: `With<Breaker>`
- **Query (breaker)**: `(&Transform, &BreakerVelocity)` — uses Bevy `Transform`, not `Position2D`

### `breaker_state_ui` *(dev)* — `debug/telemetry/systems/breaker_state_ui.rs`

- **Schedule**: `Update`
- **Filter**: `With<Breaker>`
- **Query**: `BumpTelemetryQuery` = `(&BreakerState, &BumpState, &BreakerTilt, &BreakerVelocity, &BumpPerfectWindow, &BumpEarlyWindow, &BumpLateWindow)` (single)

---

## 4. Scenario runner systems that query Breaker entities

All use `ScenarioTagBreaker` as the filter, not `With<Breaker>` directly, since tagging
is applied by `tag_game_entities`.

### `tag_game_entities` — `lifecycle/systems/entity_tagging.rs`

- **Schedule**: `OnEnter(GameState::Playing)`
- **Filter**: `(With<Breaker>, Without<ScenarioTagBreaker>)`
- **Query**: `Entity` only
- **Writes**: inserts `ScenarioTagBreaker`

### `apply_debug_setup` / `deferred_debug_setup` — `lifecycle/systems/debug_setup.rs`

- **Schedule**: `OnEnter(GameState::Playing)` + `FixedUpdate` (deferred)
- **Query alias**: `BreakerDebugQuery = (Entity, &mut Position2D)` with `(With<ScenarioTagBreaker>, Without<ScenarioTagBolt>)`
- **Writes**: `Position2D` from `DebugSetup.breaker_position`, inserts `ScenarioPhysicsFrozen`

### `apply_perfect_tracking` — `lifecycle/systems/perfect_tracking.rs`

- **Schedule**: `FixedPreUpdate`
- **Query alias**: `BreakerTrackingQuery = (&mut Position2D, &BreakerWidth)` with `(With<ScenarioTagBreaker>, Without<ScenarioTagBolt>)`
- **Writes**: `Position2D.x` (tracks bolt x with random offset)

### `check_breaker_in_bounds` — `invariants/checkers/breaker_in_bounds.rs`

- **Schedule**: `FixedUpdate`
- **Filter**: `With<ScenarioTagBreaker>`
- **Query**: `(Entity, &Position2D)`
- **Checks**: `x` outside `[left - 50.0, right + 50.0]`

### `check_breaker_position_clamped` — `invariants/checkers/breaker_position_clamped.rs`

- **Schedule**: `FixedUpdate`
- **Filter**: `With<ScenarioTagBreaker>`
- **Query**: `(Entity, &Position2D, &BreakerWidth)`
- **Checks**: `x` outside tight `[left + half_w, right - half_w]` bounds (1px tolerance)

### `check_valid_breaker_state` — `invariants/checkers/valid_breaker_state/checker.rs`

- **Schedule**: `FixedUpdate`
- **Filter**: `With<ScenarioTagBreaker>`
- **Query**: `(Entity, &BreakerState)`
- **Checks**: state machine transitions against legal set

---

## 5. Component groupings by category

### Spatial reads (4+ systems use together)

The combination `(Position2D, BreakerWidth, BreakerHeight, Option<EntityScale>)` appears
identically in three places:
- `breaker_cell_collision` (local type alias `BreakerCellCollisionQuery`)
- `breaker_wall_collision` (local type alias `BreakerWallCollisionQuery`)
- `bolt_breaker_collision` (as part of `CollisionQueryBreaker` from `queries.rs`)

`CollisionQueryBreaker` adds `BreakerTilt`, `BreakerReflectionSpread`, and `Option<ActiveSizeBoosts>`
on top of the position/size/scale core.

The `WidthBoostVisualQuery` and `MovementQuery` also both read `BreakerWidth`.

**Candidate: `BreakerSpatial`** = `(&Position2D, &BreakerWidth, &BreakerHeight, Option<&EntityScale>)`
This exact 4-tuple is shared by the two local collision aliases but NOT by `CollisionQueryBreaker`
(which extends it). Could unify the cell/wall collision local aliases.

### State reads

`BreakerState` is read by `move_breaker` (guard), `update_breaker_state` (mut), `reset_breaker`
(mut), `perfect_bump_dash_cancel` (mut), `check_valid_breaker_state` (read), `breaker_state_ui`
(read), `tag_game_entities` (no read — just existence).

`BreakerTilt` is read/mutated by `update_breaker_state`, `animate_tilt_visual`, `bolt_breaker_collision`,
`breaker_state_ui`.

### Stat reads

`MaxSpeed` is read by `move_breaker` (via `MovementQuery`) and `update_breaker_state` (via
`DashQuery`). `BreakerAcceleration` and `BreakerDeceleration` are only used in movement/dash
systems.

### Bump reads

`BumpState` is mutated by `update_bump`, `grade_bump`, `reset_breaker`, and read by
`breaker_state_ui`. `BumpPerfectWindow` is read by both `update_bump` (`BumpTimingQuery`) and
`grade_bump` (`BumpGradingQuery`). These two queries differ only in `BumpEarlyWindow` — timing
needs it, grading does not.

### Cross-domain reads (components read by systems outside `breaker/`)

| Component | External systems reading it |
|---|---|
| `Position2D` (on Breaker) | `hover_bolt`, `bolt_lost`, `bolt_breaker_collision`, scenario debug_setup, perfect_tracking, all scenario invariants |
| `BreakerWidth` | `bolt_breaker_collision` (via `CollisionQueryBreaker`), `check_breaker_position_clamped`, `apply_perfect_tracking` (via `BreakerTrackingQuery`), `breaker_cell_collision`, `breaker_wall_collision` |
| `BreakerHeight` | `bolt_breaker_collision`, `breaker_cell_collision`, `breaker_wall_collision` |
| `BreakerTilt` | `bolt_breaker_collision` (via `CollisionQueryBreaker`) |
| `BreakerReflectionSpread` | `bolt_breaker_collision` (via `CollisionQueryBreaker`) |
| `BreakerState` | `check_valid_breaker_state`, entity_tagging `map_scenario_breaker_state` |
| `BreakerVelocity` | `draw_velocity_vectors` (dev gizmo) |
| `EntityScale` (on Breaker) | `bolt_breaker_collision`, `breaker_cell_collision`, `breaker_wall_collision` |
| `ShieldActive` (on Breaker) | `bolt_lost` (mut — decrements charges, removes component) |
| `ActiveSizeBoosts` (on Breaker) | `bolt_breaker_collision` (effective half-width), `move_breaker` (effective half-width), `update_breaker_state` / `DashQuery` (flash-step clamp), `width_boost_visual` |
| `ActiveSpeedBoosts` (on Breaker) | `move_breaker`, `update_breaker_state` / `DashQuery` |
| `FlashStepActive` (on Breaker) | `update_breaker_state` / `DashQuery` (optional, triggers teleport path) |
| `Transform` (on Breaker) | `spawn_bump_grade_text`, `spawn_whiff_text`, `draw_velocity_vectors` — all read Bevy Transform, not Position2D |

---

## 6. QueryData struct migration candidates

The bolt domain uses `#[derive(QueryData)]` structs (`BoltCollisionData`, `BoltCollisionParams`,
`ResetBoltData`, `LostBoltData`) with named fields and doc comments. The breaker domain uses only
type aliases. Candidates for migration to named `QueryData` structs:

| Candidate struct name | Components | Justification |
|---|---|---|
| `BreakerCollisionParams` | `Position2D`, `BreakerTilt`, `BreakerWidth`, `BreakerHeight`, `BreakerReflectionSpread`, `Option<ActiveSizeBoosts>`, `Option<EntityScale>` | Replaces `CollisionQueryBreaker`; consumed in `bolt_breaker_collision` by name |
| `BreakerSizeParams` | `Position2D`, `BreakerWidth`, `BreakerHeight`, `Option<EntityScale>` | Shared 4-tuple in `breaker_cell_collision` and `breaker_wall_collision`; unifies local aliases |
| `BreakerMovementState` | `BreakerState`, `BreakerVelocity`, `BreakerTilt`, `BreakerStateTimer` | Core mutable state accessed by movement and dash systems |
| `BreakerBumpState` | `BumpState`, `BumpPerfectWindow`, `BumpLateWindow`, `BumpPerfectCooldown`, `BumpWeakCooldown`, `Option<AnchorPlanted>`, `Option<AnchorActive>` | Common subset of `BumpTimingQuery`/`BumpGradingQuery`; grading query is this minus `BumpEarlyWindow` |

`DashQuery` is already split into nested tuples to stay within Bevy's 15-element tuple limit; a
`#[derive(QueryData)]` struct would avoid the nested tuple syntax.

---

## Key files

- `/Users/bgardner/dev/brickbreaker/breaker-game/src/breaker/queries.rs` — all existing type aliases
- `/Users/bgardner/dev/brickbreaker/breaker-game/src/breaker/filters.rs` — `BumpTriggerFilter`, `CollisionFilterBreaker`
- `/Users/bgardner/dev/brickbreaker/breaker-game/src/bolt/queries.rs` — reference pattern for `#[derive(QueryData)]`
- `/Users/bgardner/dev/brickbreaker/breaker-game/src/bolt/systems/bolt_breaker_collision/system.rs` — largest cross-domain breaker query consumer
- `/Users/bgardner/dev/brickbreaker/breaker-game/src/bolt/systems/bolt_lost/system.rs` — reads `ShieldActive` and `Position2D` on breaker
- `/Users/bgardner/dev/brickbreaker/breaker-game/src/breaker/systems/breaker_cell_collision.rs` — local `BreakerCellCollisionQuery` alias
- `/Users/bgardner/dev/brickbreaker/breaker-game/src/breaker/systems/breaker_wall_collision.rs` — local `BreakerWallCollisionQuery` alias (identical shape to cell alias)
- `/Users/bgardner/dev/brickbreaker/breaker-scenario-runner/src/lifecycle/systems/types.rs` — `BreakerDebugQuery`, `BreakerTrackingQuery`
- `/Users/bgardner/dev/brickbreaker/breaker-scenario-runner/src/invariants/checkers/breaker_in_bounds.rs` — `(Entity, &Position2D)` with `ScenarioTagBreaker`
- `/Users/bgardner/dev/brickbreaker/breaker-scenario-runner/src/invariants/checkers/breaker_position_clamped.rs` — `(Entity, &Position2D, &BreakerWidth)` with `ScenarioTagBreaker`
- `/Users/bgardner/dev/brickbreaker/breaker-scenario-runner/src/invariants/checkers/valid_breaker_state/checker.rs` — `(Entity, &BreakerState)` with `ScenarioTagBreaker`
