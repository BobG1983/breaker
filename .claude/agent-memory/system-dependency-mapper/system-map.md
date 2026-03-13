---
name: system-map
description: Complete system inventory for the brickbreaker codebase — every system function, its plugin, schedule, ordering, and data access (as of 2026-03-13 full re-scan)
type: reference
---

# System Map — Full Inventory

Last updated: 2026-03-13 (full re-scan, Bevy 0.18.1)

## Plugin Registration Order (game.rs)
InputPlugin → ScreenPlugin → PhysicsPlugin → WallPlugin → BreakerPlugin → BoltPlugin → CellsPlugin → UpgradesPlugin → RunPlugin → AudioPlugin → UiPlugin → DebugPlugin

---

## InputPlugin

### `read_input_actions` — PreUpdate, after(InputSystems)
- Reads: Res<ButtonInput<KeyCode>>, Res<InputConfig>, Res<Time<Real>>
- Writes: ResMut<InputActions>, ResMut<DoubleTapState>
- Receives: MessageReader<KeyboardInput> (Bevy built-in)

### `clear_input_actions` — FixedPostUpdate
- Writes: ResMut<InputActions>

---

## ScreenPlugin

### `spawn_loading_screen` — OnEnter(GameState::Loading)
- Commands (spawn UI)

### `seed_configs_from_defaults` — Update, run_if(GameState::Loading), tracked as progress
- Reads: Option<Res<DefaultsCollection>>, Res<Assets<*Defaults>> x6
- Commands (insert_resource for PlayfieldConfig, BoltConfig, BreakerConfig, CellConfig, InputConfig, MainMenuConfig)
- Local<bool> seeded flag

### `update_loading_bar` — Update, run_if(GameState::Loading)
- Reads: Res<ProgressTracker<GameState>>
- Writes: Query<&mut Node, With<LoadingBarFill>>, Query<&mut Text, With<LoadingProgressText>>

### `cleanup_entities::<LoadingScreen>` — OnExit(GameState::Loading)
### `spawn_main_menu` — OnEnter(GameState::MainMenu)
- Reads: Res<MainMenuConfig>, Res<AssetServer>
- Commands (spawn, insert_resource MainMenuSelection)

### `handle_main_menu_input` + `update_menu_colors` — Update, run_if(MainMenu), chained
- handle_main_menu_input reads: Res<InputActions>; writes: ResMut<MainMenuSelection>, ResMut<NextState<GameState>>; sends: MessageWriter<AppExit>; reads Query<(&Interaction, &MenuItem), Changed<Interaction>>
- update_menu_colors reads: Res<MainMenuConfig>, Res<MainMenuSelection>; writes: Query<(&MenuItem, &mut TextColor)>

### `cleanup_entities::<MainMenuScreen>` — OnExit(GameState::MainMenu)
### `cleanup_main_menu` — OnExit(GameState::MainMenu), Commands remove_resource MainMenuSelection
### `cleanup_entities::<CleanupOnNodeExit>` — OnExit(GameState::Playing)
### `cleanup_entities::<CleanupOnRunEnd>` — OnExit(GameState::RunEnd)

---

## WallPlugin

### `spawn_walls` — OnEnter(GameState::Playing)
- Reads: Res<PlayfieldConfig>
- Commands (spawn 3 Wall entities: left, right, ceiling)

---

## BreakerPlugin

### `spawn_breaker` — OnEnter(GameState::Playing)
- Reads: Res<BreakerConfig>
- Writes: ResMut<Assets<Mesh>>, ResMut<Assets<ColorMaterial>>
- Reads (query): Query<Entity, With<Breaker>> (existence check — no double spawn)
- Commands (spawn Breaker entity with CleanupOnRunEnd)

### `init_breaker_params` — OnEnter(GameState::Playing), after(spawn_breaker)
- Reads: Res<BreakerConfig>
- Reads (query): Query<Entity, (With<Breaker>, Without<BreakerMaxSpeed>)>
- Commands (insert ~25 param components on breaker)

### `reset_breaker` — OnEnter(GameState::Playing), after(init_breaker_params)
- Reads: Res<PlayfieldConfig>
- Writes (query): BreakerResetQuery — mut Transform, BreakerState, BreakerVelocity, BreakerTilt, BreakerStateTimer; read BreakerBaseY

### `update_bump` — FixedUpdate, (no explicit ordering relative to move/state), run_if(PlayingState::Active)
- Reads: Res<InputActions>, Res<Time<Fixed>>
- Writes (query): BumpTimingQuery — mut BumpState; read BumpPerfectWindow, BumpEarlyWindow, BumpLateWindow, BumpPerfectCooldown, BumpWeakCooldown
- Sends: MessageWriter<BumpPerformed>

### `move_breaker` — FixedUpdate, after(update_bump), in_set(BreakerSystems::Move), run_if(PlayingState::Active)
- Reads: Res<InputActions>, Res<PlayfieldConfig>, Res<Time<Fixed>>
- Writes (query): BreakerMovementQuery — mut Transform, mut BreakerVelocity; read BreakerState, BreakerMaxSpeed, BreakerAcceleration, BreakerDeceleration, DecelEasing, BreakerWidth

### `update_breaker_state` — FixedUpdate, after(move_breaker), run_if(PlayingState::Active)
- Reads: Res<InputActions>, Res<Time<Fixed>>
- Writes (query): BreakerDashQuery — mut BreakerState, mut BreakerVelocity, mut BreakerTilt, mut BreakerStateTimer; read BreakerMaxSpeed, BreakerDeceleration, DecelEasing, DashSpeedMultiplier, DashDuration, DashTilt, DashTiltEase, BrakeTilt, BrakeDecel, SettleDuration, SettleTiltEase

### `grade_bump` — FixedUpdate, after(update_bump), after(PhysicsSystems::BreakerCollision), run_if(PlayingState::Active)
- Receives: MessageReader<BoltHitBreaker>
- Sends: MessageWriter<BumpPerformed>, MessageWriter<BumpWhiffed>
- Writes (query): Query<(&mut BumpState, &BumpPerfectWindow, &BumpLateWindow, &BumpPerfectCooldown, &BumpWeakCooldown), With<Breaker>>

### `perfect_bump_dash_cancel` — FixedUpdate, after(grade_bump), run_if(PlayingState::Active)
- Receives: MessageReader<BumpPerformed>
- Writes (query): Query<(&mut BreakerState, &mut BreakerStateTimer, &SettleDuration), With<Breaker>>

### `spawn_bump_grade_text` — FixedUpdate, after(grade_bump), run_if(PlayingState::Active)
- Receives: MessageReader<BumpPerformed>
- Reads (query): Query<&Transform, With<Breaker>>
- Commands (spawn FadeOut text)

### `spawn_whiff_text` — FixedUpdate, after(grade_bump), run_if(PlayingState::Active)
- Receives: MessageReader<BumpWhiffed>
- Reads (query): Query<&Transform, With<Breaker>>
- Commands (spawn FadeOut text)

### `trigger_bump_visual` — FixedUpdate, after(update_bump), run_if(PlayingState::Active)
- Reads: Res<InputActions>
- Reads (query): Query<(Entity, &BumpVisualParams), (With<Breaker>, With<BumpState>, Without<BumpVisual>)>
- Commands (insert BumpVisual)

### `animate_bump_visual` — Update, run_if(PlayingState::Active)
- Reads: Res<Time>
- Writes (query): Query<(Entity, &mut Transform, &mut BumpVisual, &BreakerBaseY, &BumpVisualParams), With<Breaker>>
- Commands (remove BumpVisual)

### `animate_tilt_visual` — Update, run_if(PlayingState::Active)
- Writes (query): Query<(&BreakerTilt, &mut Transform), With<Breaker>>

---

## BoltPlugin

### `spawn_bolt` — OnEnter(GameState::Playing)
- Reads: Res<BoltConfig>, Res<BreakerConfig>, Res<RunState>
- Writes: ResMut<Assets<Mesh>>, ResMut<Assets<ColorMaterial>>
- Reads (query): Query<&Transform, With<Breaker>> (position fallback)
- Commands (spawn Bolt entity with CleanupOnNodeExit)

### `init_bolt_params` — OnEnter(GameState::Playing), after(spawn_bolt)
- Reads: Res<BoltConfig>
- Reads (query): Query<Entity, (With<Bolt>, Without<BoltBaseSpeed>)>
- Commands (insert bolt param components)

### `launch_bolt` — FixedUpdate, run_if(PlayingState::Active)
- Reads: Res<InputActions>
- Writes (query): Query<(Entity, &mut BoltVelocity, &BoltBaseSpeed, &BoltInitialAngle), ServingBoltFilter>
- Commands (remove BoltServing)

### `hover_bolt` — FixedUpdate, after(BreakerSystems::Move), run_if(PlayingState::Active)
- Reads (query): Query<&Transform, (With<Breaker>, Without<Bolt>)>
- Writes (query): Query<(&mut Transform, &BoltSpawnOffsetY), ServingBoltFilter>

### `prepare_bolt_velocity` — FixedUpdate, after(BreakerSystems::Move), in_set(BoltSystems::PrepareVelocity), run_if(PlayingState::Active)
- Writes (query): Query<(&mut BoltVelocity, &BoltMinSpeed, &BoltMaxSpeed), ActiveBoltFilter>
- Reads (query): Query<&MinAngleFromHorizontal, (With<Breaker>, Without<Bolt>)>

### `apply_bump_velocity` — FixedUpdate, after(PhysicsSystems::BreakerCollision), run_if(PlayingState::Active)
- Receives: MessageReader<BumpPerformed>
- Writes (query): Query<(&mut BoltVelocity, &BoltBaseSpeed, &BoltMaxSpeed), With<Bolt>>
- Reads (query): Query<(&BumpPerfectMultiplier, &BumpWeakMultiplier), With<Breaker>>

### `spawn_bolt_lost_text` — FixedUpdate, run_if(PlayingState::Active)
- Receives: MessageReader<BoltLost>
- Commands (spawn FadeOut text)

### `animate_fade_out` — Update, run_if(PlayingState::Active)
- Reads: Res<Time>
- Writes (query): Query<(Entity, &mut FadeOut, &mut TextColor)>
- Commands (despawn)

---

## PhysicsPlugin

### `bolt_cell_collision` — FixedUpdate, after(BoltSystems::PrepareVelocity), run_if(PlayingState::Active)
- Reads: Res<Time<Fixed>>
- Writes (query): Query<(Entity, &mut Transform, &mut BoltVelocity, &BoltRadius), ActiveBoltFilter>
- Reads (query): Query<(Entity, &Transform, &CellWidth, &CellHeight), CellCollisionFilter>
- Reads (query): Query<(Entity, &Transform, &WallSize), WallCollisionFilter>
- Sends: MessageWriter<BoltHitCell>

### `bolt_breaker_collision` — FixedUpdate, after(bolt_cell_collision), in_set(PhysicsSystems::BreakerCollision), run_if(PlayingState::Active)
- Reads: Res<Time<Fixed>>
- Writes (query): BoltPhysicsQuery — mut Transform, mut BoltVelocity; read BoltBaseSpeed, BoltRadius
- Reads (query): Query<(&Transform, &BreakerTilt, &BreakerWidth, &BreakerHeight, &MaxReflectionAngle, &MinAngleFromHorizontal), BreakerCollisionFilter>
- Sends: MessageWriter<BoltHitBreaker>

### `bolt_lost` — FixedUpdate, after(bolt_breaker_collision), run_if(PlayingState::Active)
- Reads: Res<PlayfieldConfig>
- Writes (query): Query<(&mut Transform, &mut BoltVelocity, &BoltBaseSpeed, &BoltRadius, &BoltRespawnOffsetY), ActiveBoltFilter>
- Reads (query): Query<&Transform, (With<Breaker>, Without<Bolt>)>
- Sends: MessageWriter<BoltLost>

---

## CellsPlugin

### `spawn_cells` — OnEnter(GameState::Playing)
- Reads: Res<CellConfig>, Res<PlayfieldConfig>
- Writes: ResMut<Assets<Mesh>>, ResMut<Assets<ColorMaterial>>
- Commands (spawn grid of Cell entities with CleanupOnNodeExit)

### `handle_cell_hit` — FixedUpdate, run_if(PlayingState::Active)
- Receives: MessageReader<BoltHitCell>
- Writes (query): Query<(&mut CellHealth, &MeshMaterial2d<ColorMaterial>, &CellDamageVisuals), With<Cell>>
- Writes: ResMut<Assets<ColorMaterial>>
- Sends: MessageWriter<CellDestroyed>
- Commands (despawn destroyed cells)

---

## RunPlugin — no systems
- Init: Res<RunState>
- Registers messages: NodeCleared, TimerExpired

## AudioPlugin — no systems (Phase 0 stub)
## UpgradesPlugin — no systems (Phase 0 stub)
## UiPlugin — no systems (Phase 0 stub)
- Registers messages: UpgradeSelected

---

## DebugPlugin (cfg(feature = "dev") only)

### `debug_ui_system` — EguiPrimaryContextPass, run_if(resource_exists::<DebugOverlays>)
- Reads: Res<State<GameState>>, Res<DiagnosticsStore>
- Writes: ResMut<DebugOverlays>, EguiContexts

### `bolt_info_ui` — EguiPrimaryContextPass, run_if(resource_exists::<DebugOverlays>)
- Reads: Res<DebugOverlays>, EguiContexts
- Reads (query): Query<(&Transform, &BoltVelocity), With<Bolt>>

### `breaker_state_ui` — EguiPrimaryContextPass, run_if(resource_exists::<DebugOverlays>)
- Reads: Res<DebugOverlays>, Res<LastBumpResult>, EguiContexts
- Reads (query): BreakerBumpTelemetryQuery — BreakerState, BumpState, BreakerTilt, BreakerVelocity, BumpPerfectWindow, BumpEarlyWindow, BumpLateWindow

### `input_actions_ui` — EguiPrimaryContextPass, run_if(resource_exists::<DebugOverlays>)
- Reads: Res<DebugOverlays>, Res<InputActions>, EguiContexts

### `track_bump_result` — FixedUpdate, after(PhysicsSystems::BreakerCollision), run_if(PlayingState::Active)
- Receives: MessageReader<BumpPerformed>, MessageReader<BumpWhiffed>
- Writes: ResMut<LastBumpResult>

### `draw_hitboxes` — Update, run_if(resource_exists::<DebugOverlays>)
- Reads: Res<DebugOverlays>, Gizmos
- Reads (query): Bolt Transform+BoltRadius; Breaker Transform+BreakerWidth+BreakerHeight; Cell Transform+CellWidth+CellHeight

### `draw_velocity_vectors` — Update, run_if(resource_exists::<DebugOverlays>)
- Reads: Res<DebugOverlays>, Gizmos
- Reads (query): Bolt Transform+BoltVelocity; Breaker Transform+BreakerVelocity

---

## app.rs

### `spawn_camera` — Startup
- Commands (spawn Camera2d)
