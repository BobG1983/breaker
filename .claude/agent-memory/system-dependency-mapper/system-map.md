---
name: system-map
description: Complete system inventory for the brickbreaker codebase — every system function, its plugin, schedule, ordering, and data access (as of 2026-03-16 full re-scan)
type: reference
---

# System Map — Full Inventory

Last updated: 2026-03-16 (full re-scan, Bevy 0.18.1)

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

## ScreenPlugin (sub-plugins: LoadingPlugin, MainMenuPlugin, RunEndPlugin)

### `spawn_loading_screen` — OnEnter(GameState::Loading)
- Commands (spawn UI)

### Seeding systems (x8) — Update, run_if(GameState::Loading), tracked as progress
- Each reads its own Res<Assets<*Defaults>> and Commands (insert_resource)
- Systems: seed_playfield_config, seed_bolt_config, seed_breaker_config, seed_cell_config,
  seed_input_config, seed_main_menu_config, seed_timer_ui_config, seed_archetype_registry,
  seed_cell_type_registry, seed_node_layout_registry

### `update_loading_bar` — Update, run_if(GameState::Loading)
- Reads: Res<ProgressTracker<GameState>>
- Writes (query): Query<&mut Node, With<LoadingBarFill>>, Query<&mut Text, With<LoadingProgressText>>

### `cleanup_entities::<LoadingScreen>` — OnExit(GameState::Loading)
### `spawn_main_menu` — OnEnter(GameState::MainMenu)
- Reads: Res<MainMenuConfig>, Res<AssetServer>
- Commands (spawn, insert_resource MainMenuSelection)

### `handle_main_menu_input` + `update_menu_colors` — Update, run_if(MainMenu), chained
- handle_main_menu_input reads: Res<InputActions>; writes: ResMut<MainMenuSelection>, ResMut<NextState<GameState>>; sends: MessageWriter<AppExit>
- update_menu_colors reads: Res<MainMenuConfig>, Res<MainMenuSelection>; writes: Query<(&MenuItem, &mut TextColor)>

### `cleanup_entities::<MainMenuScreen>` + `cleanup_main_menu` — OnExit(GameState::MainMenu)
### `spawn_run_end_screen` — OnEnter(GameState::RunEnd)
- Reads: Res<RunState>
- Commands (spawn RunEndScreen)

### `handle_run_end_input` — Update, run_if(GameState::RunEnd)
- Reads: Res<InputActions>
- Writes: ResMut<NextState<GameState>>

### `cleanup_entities::<RunEndScreen>` — OnExit(GameState::RunEnd)
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
- Reads (query): Query<Entity, With<Breaker>> (existence check)
- Commands (spawn Breaker entity with CleanupOnRunEnd)

### `init_breaker_params` — OnEnter(GameState::Playing), after(spawn_breaker)
- Reads: Res<BreakerConfig>
- Reads (query): Query<Entity, (With<Breaker>, Without<BreakerMaxSpeed>)>
- Commands (insert ~25 param components on breaker)

### `reset_breaker` — OnEnter(GameState::Playing), after(init_breaker_params)
- Reads: Res<PlayfieldConfig>
- Writes (query): mut Transform, BreakerState, BreakerVelocity, BreakerTilt, BreakerStateTimer; read BreakerBaseY

### `update_bump` — FixedUpdate, run_if(PlayingState::Active)
- Reads: Res<InputActions>, Res<Time<Fixed>>
- Writes (query): mut BumpState; read BumpPerfectWindow, BumpEarlyWindow, BumpLateWindow, BumpPerfectCooldown, BumpWeakCooldown
- Sends: MessageWriter<BumpPerformed>

### `move_breaker` — FixedUpdate, after(update_bump), in_set(BreakerSystems::Move), run_if(PlayingState::Active)
- Reads: Res<InputActions>, Res<PlayfieldConfig>, Res<Time<Fixed>>
- Writes (query): mut Transform, mut BreakerVelocity; read BreakerState, BreakerMaxSpeed, BreakerAcceleration, BreakerDeceleration, DecelEasing, BreakerWidth

### `update_breaker_state` — FixedUpdate, after(move_breaker), run_if(PlayingState::Active)
- Reads: Res<InputActions>, Res<Time<Fixed>>
- Writes (query): mut BreakerState, mut BreakerVelocity, mut BreakerTilt, mut BreakerStateTimer

### `grade_bump` — FixedUpdate, after(update_bump), after(PhysicsSystems::BreakerCollision), run_if(PlayingState::Active)
- Receives: MessageReader<BoltHitBreaker>
- Sends: MessageWriter<BumpPerformed>, MessageWriter<BumpWhiffed>
- Writes (query): mut BumpState, read BumpPerfectWindow, BumpLateWindow, BumpPerfectCooldown, BumpWeakCooldown

### `perfect_bump_dash_cancel` — FixedUpdate, after(grade_bump), run_if(PlayingState::Active)
- Receives: MessageReader<BumpPerformed>
- Writes (query): mut BreakerState, mut BreakerStateTimer

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
- Writes (query): mut Transform, mut BumpVisual; read BreakerBaseY, BumpVisualParams (With<Breaker>)
- Commands (remove BumpVisual)

### `animate_tilt_visual` — Update, run_if(PlayingState::Active)
- Writes (query): Query<(&BreakerTilt, &mut Transform), With<Breaker>>

---

## BoltPlugin

### `spawn_bolt` — OnEnter(GameState::Playing)
- Reads: Res<BoltConfig>, Res<BreakerConfig>, Res<RunState>
- Writes: ResMut<Assets<Mesh>>, ResMut<Assets<ColorMaterial>>
- Reads (query): Query<&Transform, With<Breaker>>
- Commands (spawn Bolt entity with CleanupOnNodeExit)

### `init_bolt_params` — OnEnter(GameState::Playing), after(spawn_bolt)
- Reads: Res<BoltConfig>
- Commands (insert bolt param components)

### `launch_bolt` — FixedUpdate, run_if(PlayingState::Active) [NO ordering]
- Reads: Res<InputActions>
- Writes (query): mut BoltVelocity, read BoltBaseSpeed, BoltInitialAngle (ServingBoltFilter)
- Commands (remove BoltServing)

### `hover_bolt` — FixedUpdate, after(BreakerSystems::Move), run_if(PlayingState::Active)
- Reads (query): Query<&Transform, (With<Breaker>, Without<Bolt>)>
- Writes (query): mut Transform, read BoltSpawnOffsetY (ServingBoltFilter)

### `prepare_bolt_velocity` — FixedUpdate, after(BreakerSystems::Move), in_set(BoltSystems::PrepareVelocity), run_if(PlayingState::Active)
- Writes (query): mut BoltVelocity, read BoltMinSpeed, BoltMaxSpeed (ActiveBoltFilter)
- Reads (query): Query<&MinAngleFromHorizontal, (With<Breaker>, Without<Bolt>)>

### `apply_bump_velocity` — FixedUpdate, after(PhysicsSystems::BreakerCollision), run_if(PlayingState::Active)
- Receives: MessageReader<BumpPerformed>
- Writes (query): mut BoltVelocity, read BoltBaseSpeed, BoltMaxSpeed (With<Bolt>)
- Reads (query): Query<(&BumpPerfectMultiplier, &BumpWeakMultiplier), With<Breaker>>

### `spawn_bolt_lost_text` — FixedUpdate, run_if(PlayingState::Active) [NO ordering]
- Receives: MessageReader<BoltLost>
- Commands (spawn FadeOut text)

### `animate_fade_out` — Update, run_if(PlayingState::Active)
- Reads: Res<Time>
- Writes (query): mut FadeOut, mut TextColor (Any entity with FadeOut)
- Commands (despawn)

---

## PhysicsPlugin

### `bolt_cell_collision` — FixedUpdate, after(BoltSystems::PrepareVelocity), run_if(PlayingState::Active)
- Reads: Res<Time<Fixed>>
- Writes (query): mut Transform, mut BoltVelocity, read BoltRadius (ActiveBoltFilter)
- Reads (query): Entity+Transform+CellWidth+CellHeight (CellCollisionFilter)
- Reads (query): Entity+Transform+WallSize (WallCollisionFilter)
- Sends: MessageWriter<BoltHitCell>

### `bolt_breaker_collision` — FixedUpdate, after(bolt_cell_collision), in_set(PhysicsSystems::BreakerCollision), run_if(PlayingState::Active)
- Reads: Res<Time<Fixed>>
- Writes (query): mut Transform, mut BoltVelocity, read BoltBaseSpeed, BoltRadius
- Reads (query): Transform+BreakerTilt+BreakerWidth+BreakerHeight+MaxReflectionAngle+MinAngleFromHorizontal (BreakerCollisionFilter)
- Sends: MessageWriter<BoltHitBreaker>

### `bolt_lost` — FixedUpdate, after(bolt_breaker_collision), run_if(PlayingState::Active)
- Reads: Res<PlayfieldConfig>
- Writes (query): mut Transform, mut BoltVelocity, read BoltBaseSpeed, BoltRadius, BoltRespawnOffsetY (ActiveBoltFilter)
- Reads (query): Query<&Transform, (With<Breaker>, Without<Bolt>)>
- Sends: MessageWriter<BoltLost>

---

## CellsPlugin

### `handle_cell_hit` — FixedUpdate, run_if(PlayingState::Active) [NO ordering vs RunPlugin systems]
- Receives: MessageReader<BoltHitCell>
- Writes (query): mut CellHealth, read MeshMaterial2d<ColorMaterial>, CellDamageVisuals (With<Cell>)
- Writes: ResMut<Assets<ColorMaterial>>
- Sends: MessageWriter<CellDestroyed>
- Commands (despawn destroyed cells)

---

## RunPlugin

### `set_active_layout` — OnEnter(GameState::Playing), step 1 of chained sequence
- Reads: Res<RunState>, Res<NodeLayoutRegistry>
- Commands (insert_resource ActiveNodeLayout)

### `spawn_cells_from_layout` — OnEnter(GameState::Playing), step 2, in_set(NodeSystems::Spawn)
- Reads: Res<CellConfig>, Res<PlayfieldConfig>, Res<ActiveNodeLayout>, Res<CellTypeRegistry>
- Writes: ResMut<Assets<Mesh>>, ResMut<Assets<ColorMaterial>>
- Commands (spawn Cell entities with CleanupOnNodeExit)

### `init_clear_remaining` — OnEnter(GameState::Playing), step 3
- Reads (query): Query<(), With<RequiredToClear>>
- Commands (insert_resource ClearRemainingCount)

### `init_node_timer` — OnEnter(GameState::Playing), step 4
- Reads: Res<ActiveNodeLayout>
- Commands (insert_resource NodeTimer)

### `track_node_completion` — FixedUpdate, run_if(PlayingState::Active) [NO ordering vs handle_cell_hit]
- Receives: MessageReader<CellDestroyed>
- Writes: ResMut<ClearRemainingCount>
- Sends: MessageWriter<NodeCleared>

### `handle_node_cleared` — FixedUpdate, run_if(PlayingState::Active) [NO ordering vs track_node_completion]
- Receives: MessageReader<NodeCleared>
- Reads: Res<NodeLayoutRegistry>
- Writes: ResMut<RunState>, ResMut<NextState<GameState>>

### `tick_node_timer` — FixedUpdate, run_if(PlayingState::Active) [NO ordering vs handle_timer_expired]
- Reads: Res<Time<Fixed>>
- Writes: ResMut<NodeTimer>
- Sends: MessageWriter<TimerExpired>

### `handle_timer_expired` — FixedUpdate, run_if(PlayingState::Active) [NO ordering vs tick_node_timer]
- Receives: MessageReader<TimerExpired>
- Writes: ResMut<RunState>, ResMut<NextState<GameState>>

### `advance_node` — OnEnter(GameState::NodeTransition)
- Writes: ResMut<RunState>, ResMut<NextState<GameState>>

### `reset_run_state` — OnExit(GameState::MainMenu)
- Writes: ResMut<RunState>

---

## UiPlugin (now has active systems!)

### `spawn_side_panels` — OnEnter(GameState::Playing) [unordered vs spawn_timer_hud]
- Reads (query): Query<(), With<SidePanels>> (existence check)
- Commands (spawn SidePanels with CleanupOnRunEnd)

### `spawn_timer_hud` — OnEnter(GameState::Playing) [unordered vs spawn_side_panels]
- Reads: Res<TimerUiConfig>, Res<NodeTimer>, Res<AssetServer>
- Commands (spawn NodeTimerDisplay with CleanupOnNodeExit)

### `update_timer_display` — Update, run_if(PlayingState::Active)
- Reads: Res<NodeTimer>, Res<TimerUiConfig>
- Writes (query): mut Text, mut TextColor (With<NodeTimerDisplay>)

---

## AudioPlugin — no systems (stub)
## UpgradesPlugin — no systems (stub)

---

## DebugPlugin (cfg(feature = "dev") only)

### `debug_ui_system` — EguiPrimaryContextPass, run_if(resource_exists::<DebugOverlays>)
- Reads: Res<State<GameState>>, Res<DiagnosticsStore>
- Writes: ResMut<DebugOverlays>, EguiContexts

### `bolt_info_ui` — EguiPrimaryContextPass, run_if(resource_exists::<DebugOverlays>)
- Reads: Res<DebugOverlays>, EguiContexts
- Reads (query): Transform+BoltVelocity (With<Bolt>)

### `breaker_state_ui` — EguiPrimaryContextPass, run_if(resource_exists::<DebugOverlays>)
- Reads: Res<DebugOverlays>, Res<LastBumpResult>, EguiContexts
- Reads (query): BreakerState+BumpState+BreakerTilt+BreakerVelocity+BumpPerfectWindow+BumpEarlyWindow+BumpLateWindow

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
