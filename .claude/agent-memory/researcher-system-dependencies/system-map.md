---
name: system-map
description: Complete system inventory for the brickbreaker codebase — every system function, its plugin, schedule, ordering, and data access (as of 2026-03-19 post-spawn-coordinator and clamp-bolt additions)
type: reference
---

# System Map — Full Inventory

Last updated: 2026-03-19 (new: clamp_bolt_to_playfield in PhysicsPlugin; check_spawn_complete in NodePlugin; BoltSpawned/BreakerSpawned/CellsSpawned/WallsSpawned/SpawnNodeComplete messages; NodeSystems::ApplyTimePenalty set; RecordingPlugin; seed_chip_registry/seed_chip_select_config in LoadingPlugin; inject_scenario_input moved to FixedPreUpdate; invariant checkers now .after(update_breaker_state).before(BoltLost))

## Plugin Registration Order (game.rs)
InputPlugin → ScreenPlugin → InterpolatePlugin → PhysicsPlugin → WallPlugin → BreakerPlugin →
BehaviorsPlugin → BoltPlugin → CellsPlugin → ChipsPlugin → FxPlugin → RunPlugin → AudioPlugin →
UiPlugin → DebugPlugin

Note: InterpolatePlugin is registered BEFORE PhysicsPlugin.
Note: BehaviorsPlugin is a STANDALONE domain, registered between BreakerPlugin and BoltPlugin.
BreakerPlugin no longer contains any behavior sub-plugin.

---

## InterpolatePlugin

### `restore_authoritative` — FixedFirst [NO ordering constraints]
- Writes (query): mut Transform, mut PhysicsTranslation (With<InterpolateTransform>)
- Runs before ALL FixedUpdate systems by schedule position
- Effect: shifts PhysicsTranslation.previous = current; restores Transform.translation = current

### `store_authoritative` — FixedPostUpdate [NO ordering constraints]
- Reads (query): &Transform (With<InterpolateTransform>)
- Writes (query): mut PhysicsTranslation (With<InterpolateTransform>)
- Runs after ALL FixedUpdate systems complete
- Effect: captures post-physics Transform.translation into PhysicsTranslation.current

### `interpolate_transform` — PostUpdate [NO ordering constraints]
- Reads: Res<Time<Fixed>>
- Reads (query): &PhysicsTranslation (With<InterpolateTransform>)
- Writes (query): mut Transform (With<InterpolateTransform>)
- Effect: lerps Transform.translation between previous and current using overstep_fraction

Entities with interpolation: Bolt (baseline + ExtraBolt) — both get InterpolateTransform + PhysicsTranslation at spawn

---

## InputPlugin

### `read_input_actions` — PreUpdate, after(InputSystems)
- Reads: Res<ButtonInput<KeyCode>>, Res<InputConfig>, Res<Time<Real>>
- Writes: ResMut<InputActions>, ResMut<DoubleTapState>
- Receives: MessageReader<KeyboardInput> (Bevy built-in)

### `clear_input_actions` — FixedPostUpdate
- Writes: ResMut<InputActions>

---

## ScreenPlugin (sub-plugins: LoadingPlugin, MainMenuPlugin, RunEndPlugin, etc.)

### `spawn_loading_screen` — OnEnter(GameState::Loading)
- Commands (spawn UI)

### Seeding systems (x11) — Update, run_if(GameState::Loading), tracked as progress
- Each reads its own Res<Assets<*Defaults>> and Commands (insert_resource)
- Systems: seed_playfield_config, seed_bolt_config, seed_breaker_config, seed_cell_config,
  seed_input_config, seed_main_menu_config, seed_timer_ui_config, seed_archetype_registry,
  seed_cell_type_registry, seed_node_layout_registry, seed_upgrade_select_config

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

### PauseMenuPlugin
### `toggle_pause` — Update [NO condition]
- Reads: Res<InputActions> (checks GameAction::TogglePause); Writes: ResMut<NextState<PlayingState>>
- NOTE: Previously read ButtonInput<KeyCode> directly (Escape key). Now reads InputActions via GameAction::TogglePause — routed through InputPlugin's read_input_actions in PreUpdate.
### `spawn_pause_menu` — OnEnter(PlayingState::Paused)
### `handle_pause_input` — Update, run_if(PlayingState::Paused)
- Reads: Res<InputActions>; Writes: ResMut<NextState<PlayingState>>

### RunSetupPlugin
### `spawn_run_setup` — OnEnter(GameState::RunSetup)
### `handle_run_setup_input` + `update_run_setup_colors` — Update, run_if(RunSetup), chained

### UpgradeSelectPlugin
### `spawn_upgrade_select` — OnEnter(GameState::UpgradeSelect)
### `handle_upgrade_input` — Update, run_if(UpgradeSelect)
### `tick_upgrade_timer` — Update, run_if(UpgradeSelect)
### `update_upgrade_display` — Update, run_if(UpgradeSelect)

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
- Writes (query): mut Transform, BreakerState, BreakerVelocity, BreakerTilt, BreakerStateTimer, BumpState; read BreakerBaseY

### `update_bump` — FixedUpdate, run_if(PlayingState::Active) [first in chain]
- Reads: Res<InputActions>, Res<Time<Fixed>>
- Writes (query): BumpTimingQuery (mut BumpState; read BumpPerfectWindow, BumpEarlyWindow, BumpLateWindow, BumpPerfectCooldown, BumpWeakCooldown, Option<BumpPerfectMultiplier>, Option<BumpWeakMultiplier>)
- Reads (query): Query<(), With<BoltServing>> (serving guard)
- Sends: MessageWriter<BumpPerformed> (retroactive path only)

### `move_breaker` — FixedUpdate, after(update_bump), in_set(BreakerSystems::Move), run_if(PlayingState::Active)
- Reads: Res<InputActions>, Res<PlayfieldConfig>, Res<Time<Fixed>>
- Writes (query): BreakerMovementQuery (mut Transform, mut BreakerVelocity; read BreakerState, BreakerMaxSpeed, BreakerAcceleration, BreakerDeceleration, DecelEasing, BreakerWidth)

### `update_breaker_state` — FixedUpdate, after(move_breaker), run_if(PlayingState::Active)
- Reads: Res<InputActions>, Res<Time<Fixed>>
- Writes (query): mut BreakerState, mut BreakerVelocity, mut BreakerTilt, mut BreakerStateTimer

### `grade_bump` — FixedUpdate, after(update_bump), after(PhysicsSystems::BreakerCollision), run_if(PlayingState::Active)
- Receives: MessageReader<BoltHitBreaker>
- Sends: MessageWriter<BumpPerformed>, MessageWriter<BumpWhiffed>
- Writes (query): BumpGradingQuery (mut BumpState; read BumpPerfectWindow, BumpLateWindow, BumpPerfectCooldown, BumpWeakCooldown, Option<BumpPerfectMultiplier>, Option<BumpWeakMultiplier>)

### `perfect_bump_dash_cancel` — FixedUpdate, after(grade_bump), run_if(PlayingState::Active)
- Receives: MessageReader<BumpPerformed>
- Writes (query): mut BreakerState, mut BreakerStateTimer, read SettleDuration

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

## BehaviorsPlugin (src/behaviors/ — standalone domain)

Resources owned: ArchetypeRegistry, ActiveBehaviors
System set exported: BehaviorSystems::Bridge (FixedUpdate — bridge systems)

### `apply_archetype_config_overrides` — OnEnter(GameState::Playing), .before(init_breaker_params)
- Reads: Res<SelectedArchetype>, Res<ArchetypeRegistry>, Res<Assets<BreakerDefaults>>
- Writes: ResMut<BreakerConfig>
- Cross-domain write: touches BreakerConfig (BreakerPlugin resource)

### `init_archetype` — OnEnter(GameState::Playing), .after(init_breaker_params)
- Reads: Res<SelectedArchetype>, Res<ArchetypeRegistry>
- Reads (query): Query<Entity, (With<Breaker>, Without<LivesCount>)>
- Writes: ResMut<ActiveBehaviors>
- Commands: inserts LivesCount, BumpPerfectMultiplier, BumpWeakMultiplier on Breaker entity
- Cross-domain: stamps components from breaker domain (BumpPerfectMultiplier, BumpWeakMultiplier)

### `spawn_lives_display` — OnEnter(GameState::Playing), .after(init_archetype), .after(spawn_timer_hud)
- Reads (query): Query<&LivesCount>
- Reads (query): Query<Entity, With<StatusPanel>>
- Reads (query): Query<(), With<LivesDisplay>> (existence guard)
- Commands: spawns LivesDisplay as child of StatusPanel (UI entity)

### `bridge_bolt_lost` — FixedUpdate, .after(PhysicsSystems::BoltLost), .in_set(BehaviorSystems::Bridge)
- run_if: ActiveBehaviors.has_trigger(Trigger::BoltLost) AND PlayingState::Active
- Receives: MessageReader<BoltLost>
- Reads: Res<ActiveBehaviors>
- Commands: commands.trigger(ConsequenceFired(_)) for each matching consequence

### `bridge_bump` — FixedUpdate, .after(PhysicsSystems::BreakerCollision), .in_set(BehaviorSystems::Bridge)
- run_if: ActiveBehaviors.has_trigger_any_bump() AND PlayingState::Active
- Receives: MessageReader<BumpPerformed>
- Reads: Res<ActiveBehaviors>
- Commands: commands.trigger(ConsequenceFired(_)) for each matching consequence

### `handle_life_lost` — Observer on ConsequenceFired (immediate, runs in command flush)
- Pattern-matches: Consequence::LoseLife only, ignores others
- Writes (query): mut LivesCount (all entities with LivesCount)
- Sends: MessageWriter<RunLost> (only when lives.0 reaches 0)

### `handle_time_penalty` — Observer on ConsequenceFired (immediate, runs in command flush)
- Pattern-matches: Consequence::TimePenalty(seconds) only
- Sends: MessageWriter<ApplyTimePenalty>

### `handle_spawn_bolt` — Observer on ConsequenceFired (immediate, runs in command flush)
- Pattern-matches: Consequence::SpawnBolt only
- Sends: MessageWriter<SpawnAdditionalBolt>

### `update_lives_display` — Update, run_if(any_with_component::<LivesDisplay> AND PlayingState::Active)
- Reads (query): Query<&LivesCount>
- Writes (query): mut Text (With<LivesDisplay>)

---

## BoltPlugin

### `spawn_bolt` — OnEnter(GameState::Playing)
- Reads: Res<BoltConfig>, Res<BreakerConfig>, Res<RunState>
- Writes: ResMut<Assets<Mesh>>, ResMut<Assets<ColorMaterial>>
- Reads (query): Query<&Transform, With<Breaker>>
- Commands (spawn Bolt entity with CleanupOnNodeExit, InterpolateTransform, PhysicsTranslation)

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

### `apply_bump_velocity` — FixedUpdate, after(PhysicsSystems::BreakerCollision), before(PhysicsSystems::BoltLost), run_if(PlayingState::Active)
- Receives: MessageReader<BumpPerformed>
- Writes (query): mut BoltVelocity, read BoltBaseSpeed, BoltMaxSpeed (With<Bolt>)

### `spawn_additional_bolt` — FixedUpdate, after(BehaviorSystems::Bridge), run_if(PlayingState::Active)
- Receives: MessageReader<SpawnAdditionalBolt>
- Reads: Res<BoltConfig>, ResMut<GameRng>, ResMut<Assets<Mesh>>, ResMut<Assets<ColorMaterial>>
- Reads (query): Query<&Transform, With<Breaker>> (read-only)
- Commands (spawn Bolt+ExtraBolt entity with InterpolateTransform, PhysicsTranslation, CleanupOnNodeExit)
- NOTE: Orders .after(BehaviorSystems::Bridge) to ensure bridge_bump observer has run and SpawnAdditionalBolt message is available in the same tick

### `spawn_bolt_lost_text` — FixedUpdate, run_if(PlayingState::Active) [NO ordering]
- Receives: MessageReader<BoltLost>
- Commands (spawn FadeOut text)

---

## FxPlugin

### `animate_fade_out` — Update, run_if(PlayingState::Active)
- Reads: Res<Time>
- Writes (query): mut FadeOut, mut TextColor (Any entity with FadeOut component)
- Commands (despawn when FadeOut complete)

---

## PhysicsPlugin

### `bolt_cell_collision` — FixedUpdate, after(BoltSystems::PrepareVelocity), run_if(PlayingState::Active)
- Reads: Res<Time<Fixed>>
- Writes (query): mut Transform, mut BoltVelocity, read BoltRadius (ActiveBoltFilter)
- Reads (query): Entity+Transform+CellWidth+CellHeight (CellCollisionFilter)
- Reads (query): Entity+Transform+WallSize (WallCollisionFilter)
- Sends: MessageWriter<BoltHitCell>
- NOTE: BoltHitCell no longer carries a bolt Entity field (removed in feature/scenario-coverage-expansion)

### `bolt_breaker_collision` — FixedUpdate, after(bolt_cell_collision), in_set(PhysicsSystems::BreakerCollision), run_if(PlayingState::Active)
- Reads: Res<Time<Fixed>>
- Writes (query): mut Transform, mut BoltVelocity, read BoltBaseSpeed, BoltRadius
- Reads (query): Transform+BreakerTilt+BreakerWidth+BreakerHeight+MaxReflectionAngle+MinAngleFromHorizontal (BreakerCollisionFilter)
- Sends: MessageWriter<BoltHitBreaker>
- NOTE: New upward-bolt guard added: bolts moving upward (vel.y > 0) are now skipped for ALL face types (previously only top-hit path had this guard; side hits were unguarded). This means upward-moving bolts pass through the breaker entirely.

### `clamp_bolt_to_playfield` — FixedUpdate, after(bolt_breaker_collision), run_if(PlayingState::Active)
- Reads: Res<PlayfieldConfig>
- Writes (query): mut Transform, mut BoltVelocity; read BoltRadius (ActiveBoltFilter)
- NOTE: Safety clamp for bolts that escape through wall corner overlaps in CCD. Positioned AFTER bolt_breaker_collision and BEFORE bolt_lost in the chain: `bolt_cell_collision → bolt_breaker_collision → clamp_bolt_to_playfield → bolt_lost`.
- No bottom clamp — intentionally open for bolt_lost to handle.
- BoltLost set is now registered `.after(clamp_bolt_to_playfield)` (was previously `.after(bolt_breaker_collision)`).

### `bolt_lost` — FixedUpdate, after(clamp_bolt_to_playfield), in_set(PhysicsSystems::BoltLost), run_if(PlayingState::Active)
- Reads: Res<PlayfieldConfig>, ResMut<GameRng>
- Writes (query): mut Transform, mut BoltVelocity + inserts PhysicsTranslation on respawn (baseline bolt)
  OR despawns entity (ExtraBolt)
- Reads (query): Has<ExtraBolt> as part of bolt_query (ActiveBoltFilter)
- Reads (query): Query<&Transform, (With<Breaker>, Without<Bolt>)>
- Sends: MessageWriter<BoltLost> (once per lost bolt, including ExtraBolt)
- Commands (despawn ExtraBolt OR insert respawn components on baseline)

---

## CellsPlugin

### `handle_cell_hit` — FixedUpdate, run_if(PlayingState::Active) [NO ordering vs NodePlugin systems]
- Receives: MessageReader<BoltHitCell>
- Writes (query): mut CellHealth, read MeshMaterial2d<ColorMaterial>, CellDamageVisuals (With<Cell>)
- Writes: ResMut<Assets<ColorMaterial>>
- Sends: MessageWriter<CellDestroyed>
- Commands (despawn destroyed cells)

---

## RunPlugin (parent) + NodePlugin (sub-plugin)

### NodePlugin — OnEnter(GameState::Playing) setup chain (4 steps, chained):
1. `set_active_layout` — Reads: Res<RunState>, Res<NodeLayoutRegistry>; Commands (insert_resource ActiveNodeLayout)
2. `spawn_cells_from_layout` — in_set(NodeSystems::Spawn); Reads: Res<CellConfig>, Res<PlayfieldConfig>, Res<ActiveNodeLayout>, Res<CellTypeRegistry>; Writes: ResMut<Assets<Mesh>>, ResMut<Assets<ColorMaterial>>; Commands (spawn Cell entities)
3. `init_clear_remaining` — Reads (query): Query<(), With<RequiredToClear>>; Commands (insert_resource ClearRemainingCount)
4. `init_node_timer` — Reads: Res<ActiveNodeLayout>; Commands (insert_resource NodeTimer)

### `check_spawn_complete` — FixedUpdate [NO run_if condition — runs unconditionally]
- Receives: MessageReader<BoltSpawned>, MessageReader<BreakerSpawned>, MessageReader<CellsSpawned>, MessageReader<WallsSpawned>
- Sends: MessageWriter<SpawnNodeComplete>
- Uses: Local<SpawnChecklist> (bitfield, resets after firing)
- NOTE: Runs in FixedUpdate WITHOUT PlayingState::Active guard. Fires SpawnNodeComplete once when all 4 domain spawn signals have arrived. Resets automatically after firing for next node entry. Cross-frame accumulation supported.
- NOTE: SpawnNodeComplete consumed by check_no_entity_leaks (scenario runner) for baseline entity count sampling.

### `track_node_completion` — FixedUpdate, in_set(NodeSystems::TrackCompletion), run_if(PlayingState::Active)
- Receives: MessageReader<CellDestroyed>
- Writes: ResMut<ClearRemainingCount>
- Sends: MessageWriter<NodeCleared>

### `tick_node_timer` — FixedUpdate, in_set(NodeSystems::TickTimer), run_if(PlayingState::Active)
- Reads: Res<Time<Fixed>>
- Writes: ResMut<NodeTimer>
- Sends: MessageWriter<TimerExpired>

### `apply_time_penalty` — FixedUpdate, in_set(NodeSystems::ApplyTimePenalty), after(NodeSystems::TickTimer), run_if(PlayingState::Active)
- Receives: MessageReader<ApplyTimePenalty>
- Writes: ResMut<NodeTimer>
- Sends: MessageWriter<TimerExpired> (when penalty drives timer to zero)
- NOTE: Now in NodeSystems::ApplyTimePenalty set — handle_timer_expired orders .after(NodeSystems::ApplyTimePenalty) to guarantee same-tick propagation. Prior known-conflict (apply_time_penalty unordered vs handle_timer_expired) is NOW RESOLVED.

### `handle_node_cleared` — FixedUpdate, after(NodeSystems::TrackCompletion), run_if(PlayingState::Active)
- Receives: MessageReader<NodeCleared>
- Reads: Res<NodeLayoutRegistry>
- Writes: ResMut<RunState>, ResMut<NextState<GameState>>

### `handle_timer_expired` — FixedUpdate, after(NodeSystems::ApplyTimePenalty), after(handle_node_cleared), run_if(PlayingState::Active)
- Receives: MessageReader<TimerExpired>
- Writes: ResMut<RunState>, ResMut<NextState<GameState>>
- Guards: checks run_state.outcome != InProgress and run_state.transition_queued before acting
- NOTE: Now orders .after(NodeSystems::ApplyTimePenalty) — same-tick penalty-induced expiry is guaranteed. Previously known-conflict (1-tick delay) is RESOLVED.
- NOTE: Reads TimerExpired messages from BOTH tick_node_timer AND apply_time_penalty

### `handle_run_lost` — FixedUpdate, after(handle_node_cleared), after(handle_timer_expired), run_if(PlayingState::Active)
- Receives: MessageReader<RunLost>
- Writes: ResMut<RunState>, ResMut<NextState<GameState>>
- NOTE: Previously unordered vs handle_node_cleared/handle_timer_expired — NOW FIXED

### `advance_node` — OnEnter(GameState::NodeTransition)
- Writes: ResMut<RunState>, ResMut<NextState<GameState>>

### `reset_run_state` — OnExit(GameState::MainMenu)
- Writes: ResMut<RunState>

---

## UiPlugin

### OnEnter(GameState::Playing) — CHAINED: spawn_side_panels → ApplyDeferred → spawn_timer_hud
- `spawn_side_panels`: Reads (query): Query<(), With<SidePanels>> (existence check); Commands (spawn SidePanels with CleanupOnRunEnd)
- `spawn_timer_hud` — in_set(UiSystems::SpawnTimerHud): Reads: Res<TimerUiConfig>, Res<NodeTimer>, Res<AssetServer>; Reads (query): Query<(), With<NodeTimerDisplay>> (existence guard), Query<Entity, With<StatusPanel>>; Commands (spawn NodeTimerDisplay as child of StatusPanel, with CleanupOnNodeExit)
- NOTE: ApplyDeferred between them ensures SidePanels entity is committed before spawn_timer_hud queries for StatusPanel.
- NOTE: spawn_timer_hud guards are checked AFTER chaining: if no StatusPanel exists (single() fails), it returns early.

### `update_timer_display` — Update, run_if(in_state(PlayingState::Active))
- Reads: Res<NodeTimer>, Res<TimerUiConfig>
- Writes (query): mut Text, mut TextColor (With<NodeTimerDisplay>)

---

## AudioPlugin — no systems (stub)
## ChipsPlugin — no systems (stub beyond chip_select screen)
## UpgradesPlugin — no systems (stub)

---

## DebugPlugin (cfg(feature = "dev") only)

### `debug_ui_system` — EguiPrimaryContextPass, run_if(resource_exists::<DebugOverlays>)
- Reads: Res<State<GameState>>, Res<DiagnosticsStore>
- Writes: ResMut<DebugOverlays> (via flag_mut()), EguiContexts
- NOTE: DebugOverlays now uses enum-indexed bool array (Overlay enum) instead of individual bool fields. API change: `overlays.show_foo` → `overlays.is_active(Overlay::Foo)` and `overlays.flag_mut(Overlay::Foo)`.

### `bolt_info_ui` — EguiPrimaryContextPass, run_if(resource_exists::<DebugOverlays>)
- Reads: Res<DebugOverlays>, EguiContexts
- Reads (query): Transform+BoltVelocity (With<Bolt>)

### `breaker_state_ui` — EguiPrimaryContextPass, run_if(resource_exists::<DebugOverlays>)
- Reads: Res<DebugOverlays>, Res<LastBumpResult>, EguiContexts
- Reads (query): BreakerBumpTelemetryQuery

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

---

## SCENARIO RUNNER — ScenarioLifecycle FixedUpdate systems

These systems run in `FixedUpdate` alongside gameplay systems (but are in `breaker-scenario-runner`, not `breaker-game`).

### Lifecycle group (chained, .before(BreakerSystems::Move)):
```
tick_scenario_frame → inject_scenario_input → check_frame_limit   [.chain()]
```
- `tick_scenario_frame`: writes ResMut<ScenarioFrame>, Option<ResMut<ScenarioStats>>
- `inject_scenario_input`: reads Option<ResMut<ScenarioInputDriver>>, Res<ScenarioFrame>; writes ResMut<InputActions>, Option<ResMut<ScenarioStats>>
- `check_frame_limit`: reads Res<ScenarioFrame>, Res<ScenarioConfig>; sends MessageWriter<AppExit>

### 12 Invariant check systems (unordered, all read-only on game state):

| System | Reads | Writes |
|--------|-------|--------|
| `check_bolt_in_bounds` | Query(Entity+&Transform+ScenarioTagBolt), Res<PlayfieldConfig>, Res<ScenarioFrame> | ResMut<ViolationLog>, Option<ResMut<ScenarioStats>> |
| `check_bolt_speed_in_range` | Query(Entity+&BoltVelocity+&BoltMinSpeed+&BoltMaxSpeed+ScenarioTagBolt), Res<ScenarioFrame> | ResMut<ViolationLog> |
| `check_bolt_count_reasonable` | Query(Entity+ScenarioTagBolt), Res<ScenarioConfig>, Res<ScenarioFrame> | ResMut<ViolationLog> |
| `check_breaker_in_bounds` | Query(Entity+&Transform+ScenarioTagBreaker), Res<PlayfieldConfig>, Res<ScenarioFrame> | ResMut<ViolationLog> |
| `check_no_nan` | TaggedTransformQuery (Or<ScenarioTagBolt|ScenarioTagBreaker>), Res<ScenarioFrame> | ResMut<ViolationLog> |
| `check_timer_non_negative` | Option<Res<NodeTimer>>, Res<ScenarioFrame> | ResMut<ViolationLog> |
| `check_valid_state_transitions` | Res<State<GameState>>, Res<ScenarioFrame> | ResMut<ViolationLog>, ResMut<PreviousGameState> |
| `check_valid_breaker_state` | Query(&BreakerState+ScenarioTagBreaker), Local<Option<BreakerState>>, Res<ScenarioFrame> | ResMut<ViolationLog> |
| `check_timer_monotonically_decreasing` | Option<Res<NodeTimer>>, Local<Option<f32>>, Res<ScenarioFrame> | ResMut<ViolationLog> |
| `check_breaker_position_clamped` | Query(Entity+&Transform+&BreakerWidth+ScenarioTagBreaker), Res<PlayfieldConfig>, Res<ScenarioFrame> | ResMut<ViolationLog> |
| `check_physics_frozen_during_pause` | Query(Entity+&Transform+ScenarioTagBolt), Option<Res<State<PlayingState>>>, Local<HashMap>, Res<ScenarioFrame> | ResMut<ViolationLog> |
| `check_no_entity_leaks` | Query<Entity>, Res<ScenarioFrame> | ResMut<ViolationLog>, ResMut<EntityLeakBaseline> |

### 2 Mutator systems:
- `enforce_frozen_positions` — writes &mut Transform on With<ScenarioPhysicsFrozen>; NO ordering vs physics (see known-conflicts.md)
- `tag_game_entities` — reads untagged Bolt+Breaker queries (read-only); Commands (insert tag components); writes Option<ResMut<ScenarioStats>>

### OnEnter(GameState::Playing) chain (after init_bolt_params):
```
init_scenario_input → tag_game_entities → apply_debug_setup   [.chain()]
```
- `init_scenario_input`: reads Res<ScenarioConfig>; Commands (insert_resource ScenarioInputDriver)
- `apply_debug_setup`: reads Res<ScenarioConfig>; writes &mut Transform; Commands (insert ScenarioPhysicsFrozen)

### Also registered (OnEnter/OnEnter):
- `bypass_menu_to_playing` (OnEnter(MainMenu)): writes ResMut<SelectedArchetype>, ResMut<ScenarioLayoutOverride>, ResMut<NextState<GameState>>
- `auto_skip_chip_select` (OnEnter(ChipSelect)): writes ResMut<NextState<GameState>>
- `exit_on_run_end` (OnEnter(RunEnd)): sends MessageWriter<AppExit>

### tag_game_entities runs in BOTH OnEnter(Playing) AND FixedUpdate — idempotent (Without<ScenarioTagBolt> filter prevents double-tagging).
