//! Scenario lifecycle — state navigation, frame counter, and exit logic.
//!
//! [`ScenarioLifecycle`] is a Bevy plugin that:
//! - Bypasses menus: `OnEnter(GameState::MainMenu)` → immediately enters `Playing`
//! - Auto-skips chip selection: `OnEnter(GameState::ChipSelect)` → `NodeTransition`
//! - Counts fixed-update frames via [`ScenarioFrame`]
//! - Exits when `max_frames` is reached; optionally exits when the run ends
//!   naturally (controlled by [`ScenarioDefinition::allow_early_end`])

use bevy::prelude::*;
use breaker::{
    bolt::components::Bolt,
    breaker::components::Breaker,
    input::resources::InputActions,
    shared::{GameState, ScenarioLayoutOverride, SelectedArchetype},
};

use crate::{
    input::InputDriver,
    invariants::{
        EntityLeakBaseline, PreviousGameState, ScenarioFrame, ScenarioPhysicsFrozen, ScenarioStats,
        ScenarioTagBolt, ScenarioTagBreaker, ViolationLog, check_bolt_count_reasonable,
        check_bolt_in_bounds, check_bolt_speed_in_range, check_breaker_in_bounds,
        check_breaker_position_clamped, check_no_entity_leaks, check_no_nan,
        check_physics_frozen_during_pause, check_timer_monotonically_decreasing,
        check_timer_non_negative, check_valid_breaker_state, check_valid_state_transitions,
    },
    types::{GameAction as ScenarioGameAction, ScenarioDefinition},
};

/// Query alias for breaker entities in [`apply_debug_setup`].
type BreakerDebugQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static mut Transform),
    (With<ScenarioTagBreaker>, Without<ScenarioTagBolt>),
>;

/// Loaded scenario configuration, inserted before the app runs.
#[derive(Resource)]
pub struct ScenarioConfig {
    /// The full scenario definition loaded from RON.
    pub definition: ScenarioDefinition,
}

/// Bevy resource wrapping an [`InputDriver`] for the current scenario run.
///
/// Inserted by [`init_scenario_input`] when the scenario starts.
/// Queried each fixed-update tick by [`inject_scenario_input`].
#[derive(Resource)]
pub struct ScenarioInputDriver(pub InputDriver);

/// Reads [`ScenarioConfig`] and inserts a [`ScenarioInputDriver`] into the world.
///
/// Runs once at scenario startup.
pub fn init_scenario_input(config: Res<ScenarioConfig>, mut commands: Commands) {
    let driver = InputDriver::from_strategy(&config.definition.input);
    commands.insert_resource(ScenarioInputDriver(driver));
}

/// Maps a scenario-crate [`ScenarioGameAction`] to the game-crate
/// [`breaker::input::resources::GameAction`].
const fn map_action(action: ScenarioGameAction) -> breaker::input::resources::GameAction {
    match action {
        ScenarioGameAction::MoveLeft => breaker::input::resources::GameAction::MoveLeft,
        ScenarioGameAction::MoveRight => breaker::input::resources::GameAction::MoveRight,
        ScenarioGameAction::Bump => breaker::input::resources::GameAction::Bump,
        ScenarioGameAction::DashLeft => breaker::input::resources::GameAction::DashLeft,
        ScenarioGameAction::DashRight => breaker::input::resources::GameAction::DashRight,
        ScenarioGameAction::MenuUp => breaker::input::resources::GameAction::MenuUp,
        ScenarioGameAction::MenuDown => breaker::input::resources::GameAction::MenuDown,
        ScenarioGameAction::MenuLeft => breaker::input::resources::GameAction::MenuLeft,
        ScenarioGameAction::MenuRight => breaker::input::resources::GameAction::MenuRight,
        ScenarioGameAction::MenuConfirm => breaker::input::resources::GameAction::MenuConfirm,
        ScenarioGameAction::TogglePause => breaker::input::resources::GameAction::TogglePause,
    }
}

/// Injects scenario-controlled actions into [`InputActions`] each fixed-update tick.
///
/// Reads [`ScenarioInputDriver`], queries the current [`ScenarioFrame`], maps the
/// scenario-crate [`crate::types::GameAction`] values to the game crate's
/// [`breaker::input::resources::GameAction`], and writes to [`InputActions`].
///
/// Uses `Option<ResMut<ScenarioInputDriver>>` so it does not panic if the resource
/// has not yet been inserted.
pub fn inject_scenario_input(
    mut driver: Option<ResMut<ScenarioInputDriver>>,
    frame: Res<ScenarioFrame>,
    mut actions: ResMut<InputActions>,
    mut stats: Option<ResMut<ScenarioStats>>,
) {
    let Some(ref mut driver) = driver else {
        return;
    };

    let scenario_actions = driver.0.actions_for_frame(frame.0, true);
    let action_count = u32::try_from(scenario_actions.len()).unwrap_or(u32::MAX);

    for action in scenario_actions {
        actions.0.push(map_action(action));
    }

    if let Some(ref mut s) = stats {
        s.actions_injected += action_count;
    }
}

/// Plugin that drives the scenario lifecycle.
pub struct ScenarioLifecycle;

impl Plugin for ScenarioLifecycle {
    fn build(&self, app: &mut App) {
        let allow_early_end = app
            .world()
            .resource::<ScenarioConfig>()
            .definition
            .allow_early_end;

        app.init_resource::<ScenarioFrame>()
            .init_resource::<ViolationLog>()
            .init_resource::<PreviousGameState>()
            .init_resource::<EntityLeakBaseline>()
            .init_resource::<ScenarioStats>()
            .add_systems(OnEnter(GameState::MainMenu), bypass_menu_to_playing)
            .add_systems(OnEnter(GameState::ChipSelect), auto_skip_chip_select)
            .add_systems(
                OnEnter(GameState::Playing),
                (init_scenario_input, tag_game_entities, apply_debug_setup)
                    .chain()
                    .after(breaker::bolt::systems::init_bolt_params),
            )
            // Input injection runs in FixedPreUpdate so it executes after
            // clear_input_actions (FixedPostUpdate of previous tick) and before
            // all FixedUpdate game systems that read InputActions.
            .add_systems(FixedPreUpdate, inject_scenario_input)
            .add_systems(
                FixedUpdate,
                (
                    (tick_scenario_frame, check_frame_limit)
                        .chain()
                        .before(breaker::breaker::sets::BreakerSystems::Move),
                    // Invariant checkers and frozen position enforcement must run
                    // BEFORE physics systems. Otherwise bolt_lost respawns OOB
                    // bolts before invariants can detect them.
                    (
                        enforce_frozen_positions,
                        check_bolt_in_bounds,
                        check_bolt_speed_in_range,
                        check_bolt_count_reasonable,
                        check_breaker_in_bounds,
                        check_no_nan,
                        check_timer_non_negative,
                        check_valid_state_transitions,
                        check_valid_breaker_state,
                        check_timer_monotonically_decreasing,
                        check_breaker_position_clamped,
                        check_physics_frozen_during_pause,
                        check_no_entity_leaks,
                    )
                        .after(tag_game_entities)
                        .before(breaker::physics::PhysicsSystems::BoltLost),
                    tag_game_entities,
                ),
            );

        if allow_early_end {
            // Normal: exit when run ends naturally.
            // Runs every frame while in RunEnd to avoid one-shot timing issues
            // where Winit misses a single AppExit message.
            app.add_systems(Update, exit_on_run_end.run_if(in_state(GameState::RunEnd)));
        } else {
            // Stress: restart when run ends, only max_frames triggers exit.
            app.add_systems(OnEnter(GameState::RunEnd), restart_run_on_end);
        }
    }
}

/// Sets the archetype and layout override, then immediately enters `Playing`.
///
/// This bypasses `RunSetup` entirely — the scenario controls which archetype
/// and layout are used without any user interaction.
fn bypass_menu_to_playing(
    config: Res<ScenarioConfig>,
    mut selected: ResMut<SelectedArchetype>,
    mut layout_override: ResMut<ScenarioLayoutOverride>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    selected.0.clone_from(&config.definition.breaker);
    layout_override.0 = Some(config.definition.layout.clone());
    next_state.set(GameState::Playing);
}

/// Transitions immediately to `NodeTransition`, skipping chip selection UI.
///
/// No chip is applied — the scenario runner does not model chip effects.
fn auto_skip_chip_select(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::NodeTransition);
}

/// Increments [`ScenarioFrame`] by 1 each fixed-update tick.
///
/// Also updates [`ScenarioStats::max_frame`] when the stats resource is present.
fn tick_scenario_frame(mut frame: ResMut<ScenarioFrame>, mut stats: Option<ResMut<ScenarioStats>>) {
    frame.0 += 1;
    if let Some(ref mut s) = stats {
        s.max_frame = frame.0;
    }
}

/// Sends [`AppExit::Success`] when [`ScenarioFrame`] reaches `max_frames`.
fn check_frame_limit(
    frame: Res<ScenarioFrame>,
    config: Res<ScenarioConfig>,
    mut exits: MessageWriter<AppExit>,
) {
    if frame.0 >= config.definition.max_frames {
        exits.write(AppExit::Success);
    }
}

/// Sends [`AppExit::Success`] when the run ends naturally.
///
/// Runs every frame while in `RunEnd` (not as a one-shot `OnEnter`) so that
/// the Winit event loop reliably sees the exit message on macOS.
fn exit_on_run_end(mut exits: MessageWriter<AppExit>) {
    exits.write(AppExit::Success);
}

/// Redirects `RunEnd` back to `MainMenu` (which `bypass_menu_to_playing`
/// sends to `Playing`). Used when `allow_early_end` is false so the
/// scenario runs for the full `max_frames` frame budget.
fn restart_run_on_end(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::MainMenu);
}

/// Applies debug overrides from [`ScenarioConfig`] to tagged bolt and breaker entities.
///
/// For each entity tagged with [`ScenarioTagBolt`] or [`ScenarioTagBreaker`],
/// applies position teleports from [`crate::types::DebugSetup`] (z coordinate is
/// preserved). When `disable_physics` is true, inserts [`ScenarioPhysicsFrozen`]
/// on both bolts and breakers with the post-teleport position as the frozen target.
pub fn apply_debug_setup(
    config: Res<ScenarioConfig>,
    mut bolt_query: Query<(Entity, &mut Transform), With<ScenarioTagBolt>>,
    mut breaker_query: BreakerDebugQuery,
    mut commands: Commands,
) {
    let Some(setup) = config.definition.debug_setup.as_ref() else {
        return;
    };

    for (entity, mut transform) in &mut bolt_query {
        if let Some((x, y)) = setup.bolt_position {
            transform.translation.x = x;
            transform.translation.y = y;
        }

        if setup.disable_physics {
            commands.entity(entity).insert(ScenarioPhysicsFrozen {
                target: transform.translation,
            });
        }
    }

    for (entity, mut transform) in &mut breaker_query {
        if let Some((x, y)) = setup.breaker_position {
            transform.translation.x = x;
            transform.translation.y = y;
        }

        if setup.disable_physics {
            commands.entity(entity).insert(ScenarioPhysicsFrozen {
                target: transform.translation,
            });
        }
    }
}

/// Resets each entity with [`ScenarioPhysicsFrozen`] back to its pinned `target` every tick.
///
/// Prevents physics systems from moving entities that should be stationary during
/// a self-test scenario. Runs after physics in `FixedUpdate`.
pub fn enforce_frozen_positions(mut frozen: Query<(&ScenarioPhysicsFrozen, &mut Transform)>) {
    for (pinned, mut transform) in &mut frozen {
        transform.translation = pinned.target;
    }
}

/// Tags game entities with scenario marker components for invariant checking.
///
/// Finds all untagged [`Bolt`] entities and inserts [`ScenarioTagBolt`].
/// Finds all untagged [`Breaker`] entities and inserts [`ScenarioTagBreaker`].
/// Runs in `OnEnter(GameState::Playing)` before [`apply_debug_setup`].
///
/// Also sets [`ScenarioStats::entered_playing`] to `true` when the stats resource is present.
pub fn tag_game_entities(
    bolt_query: Query<Entity, (With<Bolt>, Without<ScenarioTagBolt>)>,
    breaker_query: Query<Entity, (With<Breaker>, Without<ScenarioTagBreaker>)>,
    mut commands: Commands,
    mut stats: Option<ResMut<ScenarioStats>>,
) {
    let mut bolts_tagged = 0u32;
    let mut breakers_tagged = 0u32;

    for entity in &bolt_query {
        commands.entity(entity).insert(ScenarioTagBolt);
        bolts_tagged += 1;
    }
    for entity in &breaker_query {
        commands.entity(entity).insert(ScenarioTagBreaker);
        breakers_tagged += 1;
    }

    if let Some(ref mut s) = stats {
        s.entered_playing = true;
        s.bolts_tagged += bolts_tagged;
        s.breakers_tagged += breakers_tagged;
    }
}

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;
    use breaker::{bolt::components::Bolt, breaker::components::Breaker, shared::PlayfieldConfig};

    use super::*;
    use crate::{
        invariants::{ScenarioPhysicsFrozen, ScenarioTagBolt, ScenarioTagBreaker},
        types::{
            ChaosParams, DebugSetup, InputStrategy, InvariantKind, InvariantParams,
            ScenarioDefinition, ScriptedParams,
        },
    };

    fn make_scenario(max_frames: u32) -> ScenarioDefinition {
        ScenarioDefinition {
            breaker: "aegis".to_owned(),
            layout: "corridor".to_owned(),
            input: InputStrategy::Chaos(ChaosParams {
                seed: 0,
                action_prob: 0.3,
            }),
            max_frames,
            invariants: vec![InvariantKind::BoltInBounds],
            expected_violations: None,
            debug_setup: None,
            invariant_params: InvariantParams::default(),
            allow_early_end: true,
        }
    }

    /// Scenario for lifecycle plugin integration tests — uses `Scripted` input
    /// so no randomisation is involved.
    fn make_lifecycle_test_scenario() -> ScenarioDefinition {
        ScenarioDefinition {
            breaker: "Aegis".to_owned(),
            layout: "Corridor".to_owned(),
            input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
            max_frames: 1000,
            invariants: vec![],
            expected_violations: None,
            debug_setup: None,
            invariant_params: InvariantParams::default(),
            allow_early_end: true,
        }
    }

    /// Builds a test app that uses [`ScenarioLifecycle`] as a plugin, with the
    /// minimal state wiring needed to exercise invariant registration.
    fn lifecycle_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(StatesPlugin)
            .init_state::<GameState>()
            .insert_resource(ScenarioConfig {
                definition: make_lifecycle_test_scenario(),
            })
            .insert_resource(PlayfieldConfig {
                width: 800.0,
                height: 700.0,
                background_color_rgb: [0.0, 0.0, 0.0],
                wall_thickness: 180.0,
            });
        // Resources required by bypass_menu_to_playing
        app.insert_resource(breaker::shared::SelectedArchetype("Aegis".to_owned()))
            .insert_resource(breaker::shared::ScenarioLayoutOverride(None));
        // Resources required by inject_scenario_input
        app.init_resource::<InputActions>()
            .add_plugins(ScenarioLifecycle);
        app
    }

    /// Build a minimal app for testing `apply_debug_setup` in isolation.
    fn debug_setup_app(definition: ScenarioDefinition) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ScenarioConfig { definition });
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    // -------------------------------------------------------------------------
    // tick_scenario_frame
    // -------------------------------------------------------------------------

    /// Each fixed-update tick increments [`ScenarioFrame`] by 1.
    #[test]
    fn tick_scenario_frame_increments_by_one_per_tick() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ScenarioFrame(0))
            .add_systems(FixedUpdate, tick_scenario_frame);

        tick(&mut app);
        assert_eq!(app.world().resource::<ScenarioFrame>().0, 1);

        tick(&mut app);
        assert_eq!(app.world().resource::<ScenarioFrame>().0, 2);
    }

    // -------------------------------------------------------------------------
    // check_frame_limit
    // -------------------------------------------------------------------------

    #[derive(Resource, Default)]
    struct ExitReceived(bool);

    fn capture_exit(mut reader: MessageReader<AppExit>, mut received: ResMut<ExitReceived>) {
        for _ in reader.read() {
            received.0 = true;
        }
    }

    fn exit_test_app(current_frame: u32, max_frames: u32) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<AppExit>()
            .insert_resource(ScenarioFrame(current_frame))
            .insert_resource(ScenarioConfig {
                definition: make_scenario(max_frames),
            })
            .init_resource::<ExitReceived>()
            .add_systems(FixedUpdate, (check_frame_limit, capture_exit).chain());
        app
    }

    /// When frame equals `max_frames`, `AppExit` is sent.
    #[test]
    fn check_frame_limit_sends_exit_at_max_frames() {
        let mut app = exit_test_app(100, 100);
        tick(&mut app);
        assert!(
            app.world().resource::<ExitReceived>().0,
            "expected AppExit when frame == max_frames"
        );
    }

    /// When frame exceeds `max_frames`, `AppExit` is still sent.
    #[test]
    fn check_frame_limit_sends_exit_when_frame_exceeds_max() {
        let mut app = exit_test_app(150, 100);
        tick(&mut app);
        assert!(
            app.world().resource::<ExitReceived>().0,
            "expected AppExit when frame > max_frames"
        );
    }

    /// When frame is below `max_frames`, no `AppExit` is sent.
    #[test]
    fn check_frame_limit_does_not_exit_before_max_frames() {
        let mut app = exit_test_app(99, 100);
        tick(&mut app);
        assert!(
            !app.world().resource::<ExitReceived>().0,
            "expected no AppExit when frame < max_frames"
        );
    }

    // -------------------------------------------------------------------------
    // ScenarioLifecycle — invariant system registration
    // -------------------------------------------------------------------------

    /// `check_bolt_in_bounds` is defined in `invariants.rs` but must be registered
    /// by [`ScenarioLifecycle`]. A bolt entity at y = 500.0 is above the top
    /// bound of a 700-unit-tall playfield (top = 350.0). After one tick the
    /// [`ViolationLog`] must contain exactly one entry with
    /// [`InvariantKind::BoltInBounds`].
    #[test]
    fn check_bolt_in_bounds_is_registered_in_scenario_lifecycle() {
        let mut app = lifecycle_test_app();

        // Override playfield so top() = 350.0
        app.world_mut().insert_resource(PlayfieldConfig {
            width: 800.0,
            height: 700.0,
            background_color_rgb: [0.0, 0.0, 0.0],
            wall_thickness: 180.0,
        });

        // Spawn bolt well above the top bound
        app.world_mut().spawn((
            ScenarioTagBolt,
            Transform::from_translation(Vec3::new(0.0, 500.0, 0.0)),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected 1 BoltInBounds violation from ScenarioLifecycle, got {}",
            log.0.len()
        );
        assert_eq!(
            log.0[0].invariant,
            InvariantKind::BoltInBounds,
            "expected BoltInBounds invariant kind"
        );
    }

    /// `check_no_nan` is defined in `invariants.rs` but must be registered by
    /// [`ScenarioLifecycle`]. A bolt entity with `f32::NAN` in its x translation
    /// must produce a [`ViolationEntry`] with [`InvariantKind::NoNaN`] after one tick.
    ///
    /// This test FAILS until `check_no_nan` is added to `ScenarioLifecycle::build()`.
    #[test]
    fn check_no_nan_is_registered_in_scenario_lifecycle() {
        let mut app = lifecycle_test_app();

        app.world_mut().spawn((
            ScenarioTagBolt,
            Transform::from_translation(Vec3::new(f32::NAN, 0.0, 0.0)),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            !log.0.is_empty(),
            "expected at least one NoNaN violation from ScenarioLifecycle, got none"
        );
        assert_eq!(
            log.0[0].invariant,
            InvariantKind::NoNaN,
            "expected NoNaN invariant kind"
        );
    }

    // -------------------------------------------------------------------------
    // apply_debug_setup — teleport to bolt_position (z preserved)
    // -------------------------------------------------------------------------

    /// When `debug_setup` has `bolt_position: Some((0.0, -500.0))` and
    /// `disable_physics: false`, `apply_debug_setup` must move the
    /// [`ScenarioTagBolt`] entity to `(0.0, -500.0)` while preserving z = 1.0.
    ///
    /// This test FAILS until `apply_debug_setup` is implemented.
    #[test]
    fn apply_debug_setup_teleports_bolt_to_bolt_position_preserving_z() {
        let definition = ScenarioDefinition {
            breaker: "Aegis".to_owned(),
            layout: "Corridor".to_owned(),
            input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
            max_frames: 1000,
            invariants: vec![],
            expected_violations: None,
            debug_setup: Some(DebugSetup {
                bolt_position: Some((0.0, -500.0)),
                breaker_position: None,
                disable_physics: false,
            }),
            invariant_params: InvariantParams::default(),
            allow_early_end: true,
        };

        let mut app = debug_setup_app(definition);
        app.add_systems(Update, apply_debug_setup);

        let entity = app
            .world_mut()
            .spawn((ScenarioTagBolt, Transform::from_xyz(0.0, 0.0, 1.0)))
            .id();

        // First update: system runs and enqueues commands
        app.update();
        // Second update: commands are flushed
        app.update();

        let transform = app
            .world()
            .entity(entity)
            .get::<Transform>()
            .expect("entity must still have Transform");

        assert!(
            (transform.translation.y - (-500.0_f32)).abs() < f32::EPSILON,
            "expected y = -500.0 after teleport, got {}",
            transform.translation.y
        );
        assert!(
            (transform.translation.z - 1.0_f32).abs() < f32::EPSILON,
            "expected z = 1.0 preserved, got {}",
            transform.translation.z
        );
    }

    // -------------------------------------------------------------------------
    // apply_debug_setup — teleport breaker to breaker_position (z preserved)
    // -------------------------------------------------------------------------

    /// When `debug_setup` has `breaker_position: Some((100.0, -50.0))`,
    /// `apply_debug_setup` must move the [`ScenarioTagBreaker`] entity to
    /// `(100.0, -50.0)` while preserving the original z coordinate.
    ///
    /// This test covers the `breaker_position` code path, which is distinct from
    /// the existing `bolt_position` tests.
    #[test]
    fn apply_debug_setup_teleports_breaker_to_breaker_position_preserving_z() {
        let definition = ScenarioDefinition {
            breaker: "Aegis".to_owned(),
            layout: "Corridor".to_owned(),
            input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
            max_frames: 1000,
            invariants: vec![],
            expected_violations: None,
            debug_setup: Some(DebugSetup {
                bolt_position: None,
                breaker_position: Some((100.0, -50.0)),
                disable_physics: false,
            }),
            invariant_params: InvariantParams::default(),
            allow_early_end: true,
        };

        let mut app = debug_setup_app(definition);
        app.add_systems(Update, apply_debug_setup);

        let entity = app
            .world_mut()
            .spawn((ScenarioTagBreaker, Transform::from_xyz(0.0, 0.0, 2.0)))
            .id();

        // First update: system runs and mutates transform directly (no commands needed)
        app.update();
        // Second update: flush any pending commands
        app.update();

        let transform = app
            .world()
            .entity(entity)
            .get::<Transform>()
            .expect("breaker entity must still have Transform");

        assert!(
            (transform.translation.x - 100.0_f32).abs() < f32::EPSILON,
            "expected x = 100.0 after breaker_position teleport, got {}",
            transform.translation.x
        );
        assert!(
            (transform.translation.y - (-50.0_f32)).abs() < f32::EPSILON,
            "expected y = -50.0 after breaker_position teleport, got {}",
            transform.translation.y
        );
        assert!(
            (transform.translation.z - 2.0_f32).abs() < f32::EPSILON,
            "expected z = 2.0 preserved, got {}",
            transform.translation.z
        );
    }

    // -------------------------------------------------------------------------
    // apply_debug_setup — inserts ScenarioPhysicsFrozen + disables physics
    // -------------------------------------------------------------------------

    /// When `disable_physics: true`, `apply_debug_setup` must insert
    /// [`ScenarioPhysicsFrozen`] with `target = Vec3::new(0.0, -400.0, 1.0)`.
    ///
    /// The entity's z coordinate (1.0) is baked into the frozen target.
    ///
    /// This test FAILS until `apply_debug_setup` is implemented.
    #[test]
    fn apply_debug_setup_inserts_scenario_physics_frozen_when_disable_physics_true() {
        let definition = ScenarioDefinition {
            breaker: "Aegis".to_owned(),
            layout: "Corridor".to_owned(),
            input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
            max_frames: 1000,
            invariants: vec![],
            expected_violations: None,
            debug_setup: Some(DebugSetup {
                bolt_position: Some((0.0, -400.0)),
                breaker_position: None,
                disable_physics: true,
            }),
            invariant_params: InvariantParams::default(),
            allow_early_end: true,
        };

        let mut app = debug_setup_app(definition);
        app.add_systems(Update, apply_debug_setup);

        let entity = app
            .world_mut()
            .spawn((ScenarioTagBolt, Transform::from_xyz(0.0, 0.0, 1.0)))
            .id();

        // First update: system runs
        app.update();
        // Second update: commands are flushed
        app.update();

        let frozen = app
            .world()
            .entity(entity)
            .get::<ScenarioPhysicsFrozen>()
            .expect("entity must have ScenarioPhysicsFrozen when disable_physics is true");

        assert_eq!(
            frozen.target,
            Vec3::new(0.0, -400.0, 1.0),
            "ScenarioPhysicsFrozen.target must be (0.0, -400.0, 1.0)"
        );
    }

    // -------------------------------------------------------------------------
    // enforce_frozen_positions — resets entity to frozen target each tick
    // -------------------------------------------------------------------------

    /// Each fixed-update tick, `enforce_frozen_positions` must set the entity's
    /// `Transform.translation` exactly to `ScenarioPhysicsFrozen.target`, regardless
    /// of where physics moved it.
    ///
    /// Given target = `(0.0, -500.0, 0.0)` and current position `(100.0, 200.0, 0.0)`,
    /// after one tick the position must be exactly `(0.0, -500.0, 0.0)`.
    ///
    /// This test FAILS until `enforce_frozen_positions` is implemented.
    #[test]
    fn enforce_frozen_positions_resets_entity_to_frozen_target_each_tick() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(FixedUpdate, enforce_frozen_positions);

        let entity = app
            .world_mut()
            .spawn((
                ScenarioPhysicsFrozen {
                    target: Vec3::new(0.0, -500.0, 0.0),
                },
                Transform::from_xyz(100.0, 200.0, 0.0),
            ))
            .id();

        tick(&mut app);

        let transform = app
            .world()
            .entity(entity)
            .get::<Transform>()
            .expect("entity must still have Transform");

        assert_eq!(
            transform.translation,
            Vec3::new(0.0, -500.0, 0.0),
            "expected position to be reset to frozen target (0.0, -500.0, 0.0), got {:?}",
            transform.translation
        );
    }

    // -------------------------------------------------------------------------
    // tag_game_entities — tags Bolt entities with ScenarioTagBolt
    // -------------------------------------------------------------------------

    /// `tag_game_entities` must find all [`Bolt`] entities that lack
    /// [`ScenarioTagBolt`] and insert the marker. After two updates (system
    /// runs + commands flush), the entity must have [`ScenarioTagBolt`] and its
    /// transform must be unchanged.
    ///
    /// This test FAILS until `tag_game_entities` is implemented.
    #[test]
    fn tag_game_entities_tags_bolt_entity_with_scenario_tag_bolt() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, tag_game_entities);

        let entity = app
            .world_mut()
            .spawn((Bolt, Transform::from_xyz(50.0, 50.0, 1.0)))
            .id();

        // First update: system runs and enqueues insert(ScenarioTagBolt)
        app.update();
        // Second update: commands are flushed
        app.update();

        assert!(
            app.world()
                .entity(entity)
                .get::<ScenarioTagBolt>()
                .is_some(),
            "expected ScenarioTagBolt to be added to Bolt entity"
        );

        // Transform must be unchanged — tagging should not move the entity.
        let transform = app
            .world()
            .entity(entity)
            .get::<Transform>()
            .expect("entity must still have Transform");
        assert_eq!(
            transform.translation,
            Vec3::new(50.0, 50.0, 1.0),
            "expected transform unchanged after tagging, got {:?}",
            transform.translation
        );
    }

    // -------------------------------------------------------------------------
    // tag_game_entities — tags Breaker entities with ScenarioTagBreaker
    // -------------------------------------------------------------------------

    /// `tag_game_entities` must find all [`Breaker`] entities that lack
    /// [`ScenarioTagBreaker`] and insert the marker. After two updates the
    /// entity must have [`ScenarioTagBreaker`].
    ///
    /// This test FAILS until `tag_game_entities` is implemented.
    #[test]
    fn tag_game_entities_tags_breaker_entity_with_scenario_tag_breaker() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(Update, tag_game_entities);

        let entity = app
            .world_mut()
            .spawn((Breaker, Transform::from_xyz(0.0, -250.0, 0.0)))
            .id();

        app.update();
        app.update();

        assert!(
            app.world()
                .entity(entity)
                .get::<ScenarioTagBreaker>()
                .is_some(),
            "expected ScenarioTagBreaker to be added to Breaker entity"
        );
    }

    // -------------------------------------------------------------------------
    // inject_scenario_input — writes Bump for scripted frame
    // -------------------------------------------------------------------------

    /// `inject_scenario_input` must translate `crate::types::GameAction::Bump` from
    /// the scripted driver to `breaker::input::resources::GameAction::Bump` and write
    /// it into [`InputActions`] when the current frame matches.
    ///
    /// Given: [`ScenarioInputDriver`] with `Scripted` input that has `Bump` at frame 10,
    /// [`ScenarioFrame`] = 10, and empty [`InputActions`].
    ///
    /// After the system runs, `InputActions` must contain `GameAction::Bump`.
    #[test]
    fn inject_scenario_input_writes_bump_for_scripted_frame() {
        use breaker::input::resources::{GameAction as BreakerGameAction, InputActions};

        use crate::{
            input::{InputDriver, ScriptedInput},
            types::{GameAction as ScenarioGameAction, ScriptedFrame, ScriptedParams},
        };

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let scripted = ScriptedInput::new(&ScriptedParams {
            actions: vec![ScriptedFrame {
                frame: 10,
                actions: vec![ScenarioGameAction::Bump],
            }],
        });
        app.insert_resource(ScenarioInputDriver(InputDriver::Scripted(scripted)))
            .insert_resource(ScenarioFrame(10))
            .insert_resource(InputActions::default())
            .add_systems(Update, inject_scenario_input);

        app.update();

        let actions = app.world().resource::<InputActions>();
        assert!(
            actions.active(BreakerGameAction::Bump),
            "expected InputActions to contain Bump after inject_scenario_input at frame 10"
        );
    }

    // -------------------------------------------------------------------------
    // inject_scenario_input — empty for unmatched frame
    // -------------------------------------------------------------------------

    /// `inject_scenario_input` must leave [`InputActions`] empty when the current
    /// frame does not match any scripted entry.
    ///
    /// Given: Scripted driver with `Bump` at frame 10, [`ScenarioFrame`] = 5.
    ///
    /// After the system runs, `InputActions` must remain empty.
    #[test]
    fn inject_scenario_input_empty_for_unmatched_frame() {
        use breaker::input::resources::{GameAction as BreakerGameAction, InputActions};

        use crate::{
            input::{InputDriver, ScriptedInput},
            types::{GameAction as ScenarioGameAction, ScriptedFrame, ScriptedParams},
        };

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let scripted = ScriptedInput::new(&ScriptedParams {
            actions: vec![ScriptedFrame {
                frame: 10,
                actions: vec![ScenarioGameAction::Bump],
            }],
        });
        app.insert_resource(ScenarioInputDriver(InputDriver::Scripted(scripted)))
            .insert_resource(ScenarioFrame(5))
            .insert_resource(InputActions::default())
            .add_systems(Update, inject_scenario_input);

        app.update();

        let actions = app.world().resource::<InputActions>();
        assert!(
            !actions.active(BreakerGameAction::Bump),
            "expected InputActions to NOT contain Bump at frame 5 (no scripted entry)"
        );
        assert!(
            actions.0.is_empty(),
            "expected InputActions to be empty at unmatched frame 5, got {:?}",
            actions.0
        );
    }

    // -------------------------------------------------------------------------
    // init_scenario_input — creates driver resource
    // -------------------------------------------------------------------------

    /// `init_scenario_input` must read [`ScenarioConfig`] and insert a
    /// [`ScenarioInputDriver`] resource into the world.
    ///
    /// Given: A Bevy app with [`ScenarioConfig`] containing `Chaos` input strategy.
    /// After the system runs, the world must contain [`ScenarioInputDriver`].
    #[test]
    fn init_scenario_input_creates_driver_resource() {
        use crate::types::{ChaosParams, InputStrategy};

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        app.insert_resource(ScenarioConfig {
            definition: ScenarioDefinition {
                breaker: "aegis".to_owned(),
                layout: "corridor".to_owned(),
                input: InputStrategy::Chaos(ChaosParams {
                    seed: 42,
                    action_prob: 0.3,
                }),
                max_frames: 1000,
                invariants: vec![],
                expected_violations: None,
                debug_setup: None,
                invariant_params: InvariantParams::default(),
                allow_early_end: true,
            },
        });
        app.add_systems(Update, init_scenario_input);

        app.update();

        assert!(
            app.world().get_resource::<ScenarioInputDriver>().is_some(),
            "expected ScenarioInputDriver resource to exist after init_scenario_input ran"
        );
    }

    // -------------------------------------------------------------------------
    // ScenarioStats — actions_injected incremented by inject_scenario_input
    // -------------------------------------------------------------------------

    /// When `inject_scenario_input` writes an action, `ScenarioStats::actions_injected`
    /// must be incremented.
    ///
    /// Given: Scripted driver with `Bump` at frame 5, [`ScenarioFrame`] = 5,
    /// and [`ScenarioStats`] with `actions_injected = 0`.
    /// After the system runs, `stats.actions_injected == 1`.
    #[test]
    fn scenario_stats_actions_injected_incremented_by_inject_scenario_input() {
        use breaker::input::resources::InputActions;

        use crate::{
            input::{InputDriver, ScriptedInput},
            invariants::ScenarioStats,
            types::{GameAction as ScenarioGameAction, ScriptedFrame, ScriptedParams},
        };

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);

        let scripted = ScriptedInput::new(&ScriptedParams {
            actions: vec![ScriptedFrame {
                frame: 5,
                actions: vec![ScenarioGameAction::Bump],
            }],
        });
        app.insert_resource(ScenarioInputDriver(InputDriver::Scripted(scripted)))
            .insert_resource(ScenarioFrame(5))
            .insert_resource(InputActions::default())
            .init_resource::<ScenarioStats>()
            .add_systems(Update, inject_scenario_input);

        app.update();

        let stats = app.world().resource::<ScenarioStats>();
        assert_eq!(
            stats.actions_injected, 1,
            "expected actions_injected == 1 after one action was injected, got {}",
            stats.actions_injected
        );
    }

    // -------------------------------------------------------------------------
    // ScenarioStats — invariant_checks incremented by invariant system
    // -------------------------------------------------------------------------

    /// After one tick with a tagged bolt present, `ScenarioStats::invariant_checks`
    /// must be greater than zero. The `check_bolt_in_bounds` system must increment
    /// the counter when it runs.
    #[test]
    fn scenario_stats_invariant_checks_incremented_after_one_tick() {
        use crate::invariants::{ScenarioStats, ScenarioTagBolt, check_bolt_in_bounds};

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ViolationLog::default())
            .insert_resource(ScenarioFrame::default())
            .insert_resource(breaker::shared::PlayfieldConfig::default())
            .init_resource::<ScenarioStats>()
            .add_systems(FixedUpdate, check_bolt_in_bounds);

        app.world_mut().spawn((
            ScenarioTagBolt,
            Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
        ));

        tick(&mut app);

        let stats = app.world().resource::<ScenarioStats>();
        assert!(
            stats.invariant_checks > 0,
            "expected invariant_checks > 0 after one tick with bolt entity, got {}",
            stats.invariant_checks
        );
    }

    // -------------------------------------------------------------------------
    // ScenarioStats — max_frame tracked by tick_scenario_frame
    // -------------------------------------------------------------------------

    /// After 10 ticks, `ScenarioStats::max_frame` must equal 10.
    /// `tick_scenario_frame` must update both [`ScenarioFrame`] and `stats.max_frame`.
    #[test]
    fn scenario_stats_max_frame_tracked_by_tick_scenario_frame() {
        use crate::invariants::ScenarioStats;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ScenarioFrame(0))
            .init_resource::<ScenarioStats>()
            .add_systems(FixedUpdate, tick_scenario_frame);

        for _ in 0..10 {
            tick(&mut app);
        }

        let stats = app.world().resource::<ScenarioStats>();
        assert_eq!(
            stats.max_frame, 10,
            "expected max_frame == 10 after 10 ticks, got {}",
            stats.max_frame
        );
    }

    // -------------------------------------------------------------------------
    // ScenarioStats — entered_playing set by tag_game_entities
    // -------------------------------------------------------------------------

    /// When `tag_game_entities` runs (which happens in `OnEnter(GameState::Playing)`),
    /// `ScenarioStats::entered_playing` must be set to `true`.
    #[test]
    fn scenario_stats_entered_playing_set_by_tag_game_entities() {
        use crate::invariants::ScenarioStats;

        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .init_resource::<ScenarioStats>()
            .add_systems(Update, tag_game_entities);

        // Run the system (simulates entering Playing)
        app.update();

        let stats = app.world().resource::<ScenarioStats>();
        assert!(
            stats.entered_playing,
            "expected entered_playing == true after tag_game_entities ran"
        );
    }

    // -------------------------------------------------------------------------
    // restart_run_on_end — transitions from RunEnd to MainMenu
    // -------------------------------------------------------------------------

    /// `restart_run_on_end` must set the next state to `MainMenu` so
    /// `bypass_menu_to_playing` can restart the run.
    #[test]
    fn restart_run_on_end_transitions_to_main_menu() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_plugins(StatesPlugin)
            .init_state::<GameState>()
            .add_systems(OnEnter(GameState::RunEnd), restart_run_on_end);

        // Drive into RunEnd
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::RunEnd);
        app.update();

        // OnEnter(RunEnd) fires and sets next state to MainMenu.
        // One more update applies the transition.
        app.update();

        let state = app.world().resource::<State<GameState>>();
        assert_eq!(
            **state,
            GameState::MainMenu,
            "expected restart_run_on_end to transition to MainMenu"
        );
    }
}
