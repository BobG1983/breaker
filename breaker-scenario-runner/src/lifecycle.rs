//! Scenario lifecycle — state navigation, frame counter, and exit logic.
//!
//! [`ScenarioLifecycle`] is a Bevy plugin that:
//! - Bypasses menus: `OnEnter(GameState::MainMenu)` → immediately enters `Playing`
//! - Auto-skips chip selection: `OnEnter(GameState::ChipSelect)` → `NodeTransition`
//! - Counts fixed-update frames via [`ScenarioFrame`]
//! - Exits when `max_frames` is reached or the run ends naturally

use bevy::prelude::*;
use breaker::{
    bolt::components::Bolt,
    breaker::components::Breaker,
    shared::{GameState, ScenarioLayoutOverride, SelectedArchetype},
};

use crate::{
    invariants::{
        EntityLeakBaseline, PreviousGameState, ScenarioFrame, ScenarioPhysicsFrozen,
        ScenarioTagBolt, ScenarioTagBreaker, ViolationLog, check_bolt_count_reasonable,
        check_bolt_in_bounds, check_bolt_speed_in_range, check_breaker_in_bounds,
        check_no_entity_leaks, check_no_nan, check_timer_non_negative,
        check_valid_state_transitions,
    },
    types::ScenarioDefinition,
};

/// Loaded scenario configuration, inserted before the app runs.
#[derive(Resource)]
pub struct ScenarioConfig {
    /// The full scenario definition loaded from RON.
    pub definition: ScenarioDefinition,
}

/// Plugin that drives the scenario lifecycle.
pub struct ScenarioLifecycle;

impl Plugin for ScenarioLifecycle {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScenarioFrame>()
            .init_resource::<ViolationLog>()
            .init_resource::<PreviousGameState>()
            .init_resource::<EntityLeakBaseline>()
            .add_systems(OnEnter(GameState::MainMenu), bypass_menu_to_playing)
            .add_systems(OnEnter(GameState::ChipSelect), auto_skip_chip_select)
            .add_systems(
                OnEnter(GameState::Playing),
                (tag_game_entities, apply_debug_setup)
                    .chain()
                    .after(breaker::bolt::systems::init_bolt_params),
            )
            .add_systems(
                FixedUpdate,
                (
                    (tick_scenario_frame, check_frame_limit).chain(),
                    check_bolt_in_bounds,
                    check_bolt_speed_in_range,
                    check_bolt_count_reasonable,
                    check_breaker_in_bounds,
                    check_no_nan,
                    check_timer_non_negative,
                    check_valid_state_transitions,
                    check_no_entity_leaks,
                    enforce_frozen_positions,
                ),
            )
            .add_systems(OnEnter(GameState::RunEnd), exit_on_run_end);
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
fn tick_scenario_frame(mut frame: ResMut<ScenarioFrame>) {
    frame.0 += 1;
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
fn exit_on_run_end(mut exits: MessageWriter<AppExit>) {
    exits.write(AppExit::Success);
}

/// Applies debug overrides from [`ScenarioConfig`] to tagged bolt and breaker entities.
///
/// For each entity tagged with [`ScenarioTagBolt`], applies the `bolt_position`
/// teleport from [`crate::types::DebugSetup`] (z coordinate is preserved). When
/// `disable_physics` is true, also inserts [`ScenarioPhysicsFrozen`] with the
/// post-teleport position as the frozen target.
pub fn apply_debug_setup(
    config: Res<ScenarioConfig>,
    mut bolt_query: Query<(Entity, &mut Transform), With<ScenarioTagBolt>>,
    mut breaker_query: Query<&mut Transform, (With<ScenarioTagBreaker>, Without<ScenarioTagBolt>)>,
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

    if let Some((x, y)) = setup.breaker_position {
        for mut transform in &mut breaker_query {
            transform.translation.x = x;
            transform.translation.y = y;
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
pub fn tag_game_entities(
    bolt_query: Query<Entity, (With<Bolt>, Without<ScenarioTagBolt>)>,
    breaker_query: Query<Entity, (With<Breaker>, Without<ScenarioTagBreaker>)>,
    mut commands: Commands,
) {
    for entity in &bolt_query {
        commands.entity(entity).insert(ScenarioTagBolt);
    }
    for entity in &breaker_query {
        commands.entity(entity).insert(ScenarioTagBreaker);
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
            ChaosParams, DebugSetup, InputStrategy, InvariantKind, ScenarioDefinition,
            ScriptedParams,
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
            invariant_params: Default::default(),
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
            invariant_params: Default::default(),
        }
    }

    /// Builds a test app that uses [`ScenarioLifecycle`] as a plugin, with the
    /// minimal state wiring needed to exercise invariant registration.
    fn lifecycle_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(StatesPlugin);
        app.init_state::<GameState>();
        app.insert_resource(ScenarioConfig {
            definition: make_lifecycle_test_scenario(),
        });
        app.insert_resource(PlayfieldConfig {
            width: 800.0,
            height: 700.0,
            background_color_rgb: [0.0, 0.0, 0.0],
            wall_thickness: 180.0,
        });
        // Resources required by bypass_menu_to_playing
        app.insert_resource(breaker::shared::SelectedArchetype("Aegis".to_owned()));
        app.insert_resource(breaker::shared::ScenarioLayoutOverride(None));
        app.add_plugins(ScenarioLifecycle);
        app
    }

    /// Build a minimal app for testing `apply_debug_setup` in isolation.
    fn debug_setup_app(definition: ScenarioDefinition) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(ScenarioConfig { definition });
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
        app.add_plugins(MinimalPlugins);
        app.insert_resource(ScenarioFrame(0));
        app.add_systems(FixedUpdate, tick_scenario_frame);

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
        app.add_plugins(MinimalPlugins);
        app.add_message::<AppExit>();
        app.insert_resource(ScenarioFrame(current_frame));
        app.insert_resource(ScenarioConfig {
            definition: make_scenario(max_frames),
        });
        app.init_resource::<ExitReceived>();
        app.add_systems(FixedUpdate, (check_frame_limit, capture_exit).chain());
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
    /// by [`ScenarioLifecycle`]. A bolt entity at y = -500.0 is below the bottom
    /// bound of a 700-unit-tall playfield (bottom = -350.0). After one tick the
    /// [`ViolationLog`] must contain exactly one entry with
    /// [`InvariantKind::BoltInBounds`].
    ///
    /// This test FAILS until `check_bolt_in_bounds` is added to
    /// `ScenarioLifecycle::build()`.
    #[test]
    fn check_bolt_in_bounds_is_registered_in_scenario_lifecycle() {
        let mut app = lifecycle_test_app();

        // Override playfield so bottom() = -350.0
        app.world_mut().insert_resource(PlayfieldConfig {
            width: 800.0,
            height: 700.0,
            background_color_rgb: [0.0, 0.0, 0.0],
            wall_thickness: 180.0,
        });

        // Spawn bolt well below the bottom bound
        app.world_mut().spawn((
            ScenarioTagBolt,
            Transform::from_translation(Vec3::new(0.0, -500.0, 0.0)),
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
            invariant_params: Default::default(),
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
            invariant_params: Default::default(),
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
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, enforce_frozen_positions);

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
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, tag_game_entities);

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
        app.add_plugins(MinimalPlugins);
        app.add_systems(Update, tag_game_entities);

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
}
