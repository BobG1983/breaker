---
name: system-map
description: Complete system inventory for the brickbreaker codebase — every system function, its plugin, schedule, ordering, and data access (as of 2026-03-12)
type: reference
---

# System Inventory

Bevy version: **0.18.1**. Message API: `#[derive(Message)]`, `MessageWriter<T>`, `MessageReader<T>`.

## InputPlugin

### `read_input_actions` — PreUpdate (after InputSystems)
- Reads: `Res<ButtonInput<KeyCode>>`, `Res<InputConfig>`, `Res<Time<Real>>`
- Writes: `ResMut<InputActions>`, `ResMut<DoubleTapState>`
- Receives: `MessageReader<KeyboardInput>` (Bevy built-in)
- No run_if condition (runs every frame regardless of game state)

## BreakerPlugin

### `spawn_breaker` — OnEnter(GameState::Playing)
- Reads: `Res<BreakerConfig>`
- Writes: `ResMut<Assets<Mesh>>`, `ResMut<Assets<ColorMaterial>>`, `Commands`
- Queries: `Query<Entity, With<Breaker>>` (read-only, idempotency check)
- Ordering: before `reset_breaker`

### `reset_breaker` — OnEnter(GameState::Playing)
- Reads: `Res<BreakerConfig>`, `Res<PlayfieldConfig>`
- Queries: `Query<(&mut Transform, &mut BreakerState, &mut BreakerVelocity, &mut BreakerTilt, &mut BreakerStateTimer), With<Breaker>>` (write)
- Ordering: after `spawn_breaker`

### `update_bump` — FixedUpdate, PlayingState::Active
- Reads: `Res<InputActions>`, `Res<BreakerConfig>`, `Res<Time<Fixed>>`
- Queries: `Query<&mut BumpState, With<Breaker>>` (write)
- Sends: `MessageWriter<BumpPerformed>` (sends BumpGrade::Timeout on timer expiry)
- Ordering: before `move_breaker`; no explicit `.before()` but positionally first in the tuple

### `move_breaker` — FixedUpdate, PlayingState::Active
- Reads: `Res<InputActions>`, `Res<BreakerConfig>`, `Res<PlayfieldConfig>`, `Res<Time<Fixed>>`
- Queries: `Query<(&mut Transform, &mut BreakerVelocity, &BreakerState), With<Breaker>>` (Transform write, BreakerVelocity write, BreakerState read)
- Ordering: `.after(update_bump).in_set(BreakerSystems::Move)`

### `update_breaker_state` — FixedUpdate, PlayingState::Active
- Reads: `Res<InputActions>`, `Res<BreakerConfig>`, `Res<Time<Fixed>>`
- Queries: `Query<(&mut BreakerState, &mut BreakerVelocity, &mut BreakerTilt, &mut BreakerStateTimer), With<Breaker>>` (all write)
- Ordering: `.after(move_breaker)`

### `grade_bump` — FixedUpdate, PlayingState::Active
- Reads: `Res<BreakerConfig>`
- Queries: `Query<&BumpState, With<Breaker>>` (read)
- Receives: `MessageReader<BoltHitBreaker>`
- Sends: `MessageWriter<BumpPerformed>`
- Ordering: `.after(update_breaker_state)`

### `perfect_bump_dash_cancel` — FixedUpdate, PlayingState::Active
- Reads: `Res<BreakerConfig>`
- Receives: `MessageReader<BumpPerformed>`
- Queries: `Query<(&mut BreakerState, &mut BreakerStateTimer), With<Breaker>>` (write)
- Ordering: `.after(grade_bump)`

### `spawn_bump_grade_text` — FixedUpdate, PlayingState::Active
- Receives: `MessageReader<BumpPerformed>`
- Queries: `Query<&Transform, With<Breaker>>` (read)
- Writes: `Commands`
- Ordering: `.after(update_bump)` (no explicit after grade_bump)

### `trigger_bump_visual` — Update, PlayingState::Active
- Reads: `Res<BreakerConfig>`
- Queries: `Query<(Entity, &BumpState), (With<Breaker>, Without<BumpVisual>)>` (read)
- Writes: `Commands`
- Ordering: before `animate_bump_visual`

### `animate_bump_visual` — Update, PlayingState::Active
- Reads: `Res<Time>`, `Res<BreakerConfig>`
- Queries: `Query<(Entity, &mut Transform, &mut BumpVisual), With<Breaker>>` (Transform write, BumpVisual write)
- Writes: `Commands`
- Ordering: `.after(trigger_bump_visual)`

## BoltPlugin

### `spawn_bolt` — OnEnter(GameState::Playing)
- Reads: `Res<BoltConfig>`, `Res<BreakerConfig>`, `Res<RunState>`
- Writes: `ResMut<Assets<Mesh>>`, `ResMut<Assets<ColorMaterial>>`, `Commands`

### `launch_bolt` — FixedUpdate, PlayingState::Active
- Reads: `Res<InputActions>`, `Res<BoltConfig>`
- Queries: `Query<(Entity, &mut BoltVelocity), (With<Bolt>, With<BoltServing>)>` (write)
- Writes: `Commands`
- No explicit ordering constraint

### `hover_bolt` — FixedUpdate, PlayingState::Active
- Reads: `Res<BoltConfig>`
- Queries: `Query<&Transform, (With<Breaker>, Without<Bolt>)>` (read), `Query<&mut Transform, (With<Bolt>, With<BoltServing>)>` (write)
- Ordering: `.after(BreakerSystems::Move)`

### `prepare_bolt_velocity` — FixedUpdate, PlayingState::Active
- Reads: `Res<BoltConfig>`
- Queries: `Query<&mut BoltVelocity, ActiveBoltFilter>` (write — filter excludes BoltServing)
- Ordering: `.after(BreakerSystems::Move).in_set(BoltSystems::PrepareVelocity)`

### `apply_bump_velocity` — FixedUpdate, PlayingState::Active
- Reads: `Res<BoltConfig>`, `Res<BreakerConfig>`
- Receives: `MessageReader<BumpPerformed>`
- Queries: `Query<&mut BoltVelocity, With<Bolt>>` (write)
- No explicit ordering constraint

### `spawn_bolt_lost_text` — FixedUpdate, PlayingState::Active
- Receives: `MessageReader<BoltLost>`
- Writes: `Commands`
- No explicit ordering constraint

### `animate_fade_out` — Update, PlayingState::Active
- Reads: `Res<Time>`
- Queries: `Query<(Entity, &mut FadeOut, &mut TextColor)>` (write)
- Writes: `Commands`
- No ordering constraint relative to other systems

## PhysicsPlugin

### `spawn_walls` — OnEnter(GameState::Playing)
- Reads: `Res<PlayfieldConfig>`
- Writes: `Commands`

### `bolt_cell_collision` — FixedUpdate, PlayingState::Active
- Reads: `Res<Time<Fixed>>`, `Res<BoltConfig>`, `Res<CellConfig>`
- Queries: `Query<(Entity, &mut Transform, &mut BoltVelocity), ActiveBoltFilter>` (Transform write, BoltVelocity write), `Query<(Entity, &Transform), CellFilter>` (read), `Query<(Entity, &Transform, &WallSize), WallFilter>` (read)
- Sends: `MessageWriter<BoltHitCell>`
- Ordering: `.after(BoltSystems::PrepareVelocity)`

### `bolt_breaker_collision` — FixedUpdate, PlayingState::Active
- Reads: `Res<Time<Fixed>>`, `Res<BoltConfig>`, `Res<BreakerConfig>`, `Res<PhysicsConfig>`
- Queries: `Query<(Entity, &mut Transform, &mut BoltVelocity), ActiveBoltFilter>` (write), `Query<(&Transform, &BreakerTilt), (With<Breaker>, Without<Bolt>)>` (read)
- Sends: `MessageWriter<BoltHitBreaker>`
- Ordering: `.after(bolt_cell_collision)`

### `bolt_lost` — FixedUpdate, PlayingState::Active
- Reads: `Res<BoltConfig>`, `Res<PlayfieldConfig>`
- Queries: `Query<(&mut Transform, &mut BoltVelocity), ActiveBoltFilter>` (write), `Query<&Transform, (With<Breaker>, Without<Bolt>)>` (read)
- Sends: `MessageWriter<BoltLost>`
- Ordering: `.after(bolt_breaker_collision)`

## CellsPlugin

### `spawn_cells` — OnEnter(GameState::Playing)
- Reads: `Res<CellConfig>`, `Res<PlayfieldConfig>`
- Writes: `ResMut<Assets<Mesh>>`, `ResMut<Assets<ColorMaterial>>`, `Commands`

### `handle_cell_hit` — FixedUpdate, PlayingState::Active
- Reads: `Res<CellConfig>`
- Receives: `MessageReader<BoltHitCell>`
- Queries: `Query<(&mut CellHealth, &MeshMaterial2d<ColorMaterial>), With<Cell>>` (write)
- Writes: `ResMut<Assets<ColorMaterial>>`, `Commands`, `MessageWriter<CellDestroyed>`
- No explicit ordering constraint

## ScreenPlugin

### `seed_configs_from_defaults` — Update (run_if Loading)
- Reads: `Option<Res<DefaultsCollection>>`, `Res<Assets<PlayfieldDefaults>>`, `Res<Assets<BoltDefaults>>`, `Res<Assets<BreakerDefaults>>`, `Res<Assets<CellDefaults>>`, `Res<Assets<PhysicsDefaults>>`, `Res<Assets<MainMenuDefaults>>`
- Writes: `Commands` (insert_resource calls), `Local<bool>`
- Returns Progress (iyes_progress)

### `spawn_loading_screen` — OnEnter(GameState::Loading)
- Writes: `Commands`

### `update_loading_bar` — Update (run_if Loading)
- Reads: `Res<ProgressTracker<GameState>>`
- Queries: `Query<&mut Node, With<LoadingBarFill>>` (write), `Query<&mut Text, With<LoadingProgressText>>` (write)

### `cleanup_entities::<LoadingScreen>` — OnExit(GameState::Loading)
- Queries: `Query<Entity, With<LoadingScreen>>` (read)
- Writes: `Commands`

### `spawn_main_menu` — OnEnter(GameState::MainMenu)
- Reads: `Res<MainMenuConfig>`, `Res<AssetServer>`
- Writes: `Commands`

### `handle_main_menu_input` — Update (run_if MainMenu), chained before `update_menu_colors`
- Reads: `Res<ButtonInput<KeyCode>>`
- Writes: `ResMut<MainMenuSelection>`, `ResMut<NextState<GameState>>`
- Sends: `MessageWriter<AppExit>`
- Queries: `Query<(&Interaction, &MenuItem), Changed<Interaction>>` (read)

### `update_menu_colors` — Update (run_if MainMenu), chained after `handle_main_menu_input`
- Reads: `Res<MainMenuConfig>`, `Res<MainMenuSelection>`
- Queries: `Query<(&MenuItem, &mut TextColor)>` (write)

### `cleanup_entities::<MainMenuScreen>` — OnExit(GameState::MainMenu)
- Queries: `Query<Entity, With<MainMenuScreen>>` (read)
- Writes: `Commands`

### `cleanup_main_menu` — OnExit(GameState::MainMenu)
- Writes: `Commands` (remove_resource::<MainMenuSelection>)

### `cleanup_entities::<CleanupOnNodeExit>` — OnExit(GameState::Playing)
- Queries: `Query<Entity, With<CleanupOnNodeExit>>` (read)
- Writes: `Commands`

### `cleanup_entities::<CleanupOnRunEnd>` — OnExit(GameState::RunEnd)
- Queries: `Query<Entity, With<CleanupOnRunEnd>>` (read)
- Writes: `Commands`

## DebugPlugin (feature = "dev" only)

### `debug_ui_system` — EguiPrimaryContextPass (run_if resource_exists::<DebugOverlays>)
- Reads: `Res<State<GameState>>`, `Res<DiagnosticsStore>`
- Writes: `ResMut<DebugOverlays>`, `EguiContexts`

### `bolt_info_ui` — EguiPrimaryContextPass (run_if resource_exists::<DebugOverlays>)
- Reads: `Res<DebugOverlays>`, `EguiContexts`
- Queries: `Query<(&Transform, &BoltVelocity), With<Bolt>>` (read)

### `breaker_state_ui` — EguiPrimaryContextPass (run_if resource_exists::<DebugOverlays>)
- Reads: `Res<DebugOverlays>`, `EguiContexts`
- Queries: `Query<(&BreakerState, &BumpState, &BreakerTilt, &BreakerVelocity), With<Breaker>>` (read)

### `draw_hitboxes` — Update (run_if resource_exists::<DebugOverlays>)
- Reads: `Res<DebugOverlays>`, `Res<BoltConfig>`, `Res<BreakerConfig>`, `Res<CellConfig>`
- Queries: `Query<&Transform, With<Bolt>>` (read), `Query<&Transform, With<Breaker>>` (read), `Query<&Transform, With<Cell>>` (read)
- Writes: `Gizmos`

### `draw_velocity_vectors` — Update (run_if resource_exists::<DebugOverlays>)
- Reads: `Res<DebugOverlays>`
- Queries: `Query<(&Transform, &BoltVelocity), With<Bolt>>` (read), `Query<(&Transform, &BreakerVelocity), With<Breaker>>` (read)
- Writes: `Gizmos`

## Stub Plugins (no systems yet)

- **AudioPlugin**: registered, no systems
- **RunPlugin**: registers messages NodeCleared, TimerExpired and RunState resource only; no systems
- **UiPlugin**: registers UpgradeSelected message only; no systems
- **UpgradesPlugin**: completely empty stub
