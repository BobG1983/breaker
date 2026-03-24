use bevy::state::app::StatesPlugin;
use breaker::{
    bolt::components::Bolt,
    breaker::components::{Breaker, BreakerState},
    run::node::resources::NodeTimer,
    shared::{GameState, PlayfieldConfig, PlayingState},
};
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use super::*;
use crate::{
    invariants::{
        PreviousGameState, ScenarioPhysicsFrozen, ScenarioStats, ScenarioTagBolt,
        ScenarioTagBreaker,
    },
    types::{
        ChaosParams, DebugSetup, ForcedGameState, FrameMutation, InputStrategy, InvariantKind,
        InvariantParams, MutationKind, ScenarioBreakerState, ScenarioDefinition, ScriptedParams,
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
        stress: None,
        seed: None,
        initial_overclocks: None,
        frame_mutations: None,
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
        stress: None,
        seed: None,
        initial_overclocks: None,
        frame_mutations: None,
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
            zone_fraction: 0.667,
        });
    // Resources required by bypass_menu_to_playing
    app.insert_resource(breaker::shared::SelectedArchetype("Aegis".to_owned()))
        .insert_resource(breaker::run::node::ScenarioLayoutOverride(None))
        .init_resource::<breaker::shared::RunSeed>();
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
        zone_fraction: 0.667,
    });

    // Set entered_playing so invariant checkers are active
    app.world_mut()
        .resource_mut::<ScenarioStats>()
        .entered_playing = true;

    // Spawn bolt well above the top bound
    app.world_mut()
        .spawn((ScenarioTagBolt, Position2D(Vec2::new(0.0, 500.0))));

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
/// [`ScenarioLifecycle`]. A bolt entity with `f32::NAN` in its x position
/// must produce a [`ViolationEntry`] with [`InvariantKind::NoNaN`] after one tick.
///
/// This test FAILS until `check_no_nan` is added to `ScenarioLifecycle::build()`.
#[test]
fn check_no_nan_is_registered_in_scenario_lifecycle() {
    let mut app = lifecycle_test_app();

    // Set entered_playing so invariant checkers are active
    app.world_mut()
        .resource_mut::<ScenarioStats>()
        .entered_playing = true;

    app.world_mut()
        .spawn((ScenarioTagBolt, Position2D(Vec2::new(f32::NAN, 0.0))));

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
// apply_debug_setup — teleport to bolt_position
// -------------------------------------------------------------------------

/// When `debug_setup` has `bolt_position: Some((0.0, -500.0))` and
/// `disable_physics: false`, `apply_debug_setup` must move the
/// [`ScenarioTagBolt`] entity's `Position2D` to `(0.0, -500.0)`.
#[test]
fn apply_debug_setup_teleports_bolt_to_bolt_position() {
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
            ..Default::default()
        }),
        invariant_params: InvariantParams::default(),
        allow_early_end: true,
        stress: None,
        seed: None,
        initial_overclocks: None,
        frame_mutations: None,
    };

    let mut app = debug_setup_app(definition);
    app.add_systems(Update, apply_debug_setup);

    let entity = app
        .world_mut()
        .spawn((ScenarioTagBolt, Position2D(Vec2::new(0.0, 0.0)), Velocity2D(Vec2::ZERO)))
        .id();

    // First update: system runs and enqueues commands
    app.update();
    // Second update: commands are flushed
    app.update();

    let position = app
        .world()
        .entity(entity)
        .get::<Position2D>()
        .expect("entity must still have Position2D");

    assert!(
        (position.0.y - (-500.0_f32)).abs() < f32::EPSILON,
        "expected y = -500.0 after teleport, got {}",
        position.0.y
    );
}

// -------------------------------------------------------------------------
// apply_debug_setup — teleport breaker to breaker_position
// -------------------------------------------------------------------------

/// When `debug_setup` has `breaker_position: Some((100.0, -50.0))`,
/// `apply_debug_setup` must move the [`ScenarioTagBreaker`] entity's
/// `Position2D` to `(100.0, -50.0)`.
#[test]
fn apply_debug_setup_teleports_breaker_to_breaker_position() {
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
            ..Default::default()
        }),
        invariant_params: InvariantParams::default(),
        allow_early_end: true,
        stress: None,
        seed: None,
        initial_overclocks: None,
        frame_mutations: None,
    };

    let mut app = debug_setup_app(definition);
    app.add_systems(Update, apply_debug_setup);

    let entity = app
        .world_mut()
        .spawn((ScenarioTagBreaker, Position2D(Vec2::new(0.0, 0.0))))
        .id();

    // First update: system runs and mutates position directly (no commands needed)
    app.update();
    // Second update: flush any pending commands
    app.update();

    let position = app
        .world()
        .entity(entity)
        .get::<Position2D>()
        .expect("breaker entity must still have Position2D");

    assert!(
        (position.0.x - 100.0_f32).abs() < f32::EPSILON,
        "expected x = 100.0 after breaker_position teleport, got {}",
        position.0.x
    );
    assert!(
        (position.0.y - (-50.0_f32)).abs() < f32::EPSILON,
        "expected y = -50.0 after breaker_position teleport, got {}",
        position.0.y
    );
}

// -------------------------------------------------------------------------
// apply_debug_setup — inserts ScenarioPhysicsFrozen + disables physics
// -------------------------------------------------------------------------

/// When `disable_physics: true`, `apply_debug_setup` must insert
/// [`ScenarioPhysicsFrozen`] with `target = Vec2::new(0.0, -400.0)`.
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
            ..Default::default()
        }),
        invariant_params: InvariantParams::default(),
        allow_early_end: true,
        stress: None,
        seed: None,
        initial_overclocks: None,
        frame_mutations: None,
    };

    let mut app = debug_setup_app(definition);
    app.add_systems(Update, apply_debug_setup);

    let entity = app
        .world_mut()
        .spawn((ScenarioTagBolt, Position2D(Vec2::new(0.0, 0.0)), Velocity2D(Vec2::ZERO)))
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
        Vec2::new(0.0, -400.0),
        "ScenarioPhysicsFrozen.target must be (0.0, -400.0)"
    );
}

// -------------------------------------------------------------------------
// enforce_frozen_positions — resets entity to frozen target each tick
// -------------------------------------------------------------------------

/// Each fixed-update tick, `enforce_frozen_positions` must set the entity's
/// `Position2D` exactly to `ScenarioPhysicsFrozen.target`, regardless
/// of where physics moved it.
///
/// Given target = `(0.0, -500.0)` and current position `(100.0, 200.0)`,
/// after one tick the position must be exactly `(0.0, -500.0)`.
#[test]
fn enforce_frozen_positions_resets_entity_to_frozen_target_each_tick() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(FixedUpdate, enforce_frozen_positions);

    let entity = app
        .world_mut()
        .spawn((
            ScenarioPhysicsFrozen {
                target: Vec2::new(0.0, -500.0),
            },
            Position2D(Vec2::new(100.0, 200.0)),
        ))
        .id();

    tick(&mut app);

    let position = app
        .world()
        .entity(entity)
        .get::<Position2D>()
        .expect("entity must still have Position2D");

    assert_eq!(
        position.0,
        Vec2::new(0.0, -500.0),
        "expected position to be reset to frozen target (0.0, -500.0), got {:?}",
        position.0
    );
}

// -------------------------------------------------------------------------
// tag_game_entities — tags Bolt entities with ScenarioTagBolt
// -------------------------------------------------------------------------

/// `tag_game_entities` must find all [`Bolt`] entities that lack
/// [`ScenarioTagBolt`] and insert the marker. After two updates (system
/// runs + commands flush), the entity must have [`ScenarioTagBolt`] and its
/// position must be unchanged.
#[test]
fn tag_game_entities_tags_bolt_entity_with_scenario_tag_bolt() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Update, tag_game_entities);

    let entity = app
        .world_mut()
        .spawn((Bolt, Position2D(Vec2::new(50.0, 50.0))))
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

    // Position must be unchanged — tagging should not move the entity.
    let position = app
        .world()
        .entity(entity)
        .get::<Position2D>()
        .expect("entity must still have Position2D");
    assert_eq!(
        position.0,
        Vec2::new(50.0, 50.0),
        "expected position unchanged after tagging, got {:?}",
        position.0
    );
}

// -------------------------------------------------------------------------
// tag_game_entities — tags Breaker entities with ScenarioTagBreaker
// -------------------------------------------------------------------------

/// `tag_game_entities` must find all [`Breaker`] entities that lack
/// [`ScenarioTagBreaker`] and insert the marker. After two updates the
/// entity must have [`ScenarioTagBreaker`].
#[test]
fn tag_game_entities_tags_breaker_entity_with_scenario_tag_breaker() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_systems(Update, tag_game_entities);

    let entity = app
        .world_mut()
        .spawn((Breaker, Position2D(Vec2::new(0.0, -250.0))))
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
            stress: None,
            seed: None,
            initial_overclocks: None,
            frame_mutations: None,
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
        .insert_resource(ScenarioStats {
            entered_playing: true,
            ..Default::default()
        })
        .add_systems(FixedUpdate, check_bolt_in_bounds);

    app.world_mut()
        .spawn((ScenarioTagBolt, Position2D(Vec2::new(0.0, 0.0))));

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
// ScenarioStats — entered_playing set by mark_entered_playing_on_spawn_complete
// -------------------------------------------------------------------------

/// When `SpawnNodeComplete` fires, `mark_entered_playing_on_spawn_complete`
/// sets `ScenarioStats::entered_playing` to `true`.
#[test]
fn scenario_stats_entered_playing_set_on_spawn_node_complete() {
    use breaker::run::node::messages::SpawnNodeComplete;

    use crate::invariants::ScenarioStats;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<ScenarioStats>()
        .add_message::<SpawnNodeComplete>()
        .add_systems(Update, mark_entered_playing_on_spawn_complete);

    // Send SpawnNodeComplete message
    app.world_mut()
        .resource_mut::<Messages<SpawnNodeComplete>>()
        .write(SpawnNodeComplete);

    app.update();

    let stats = app.world().resource::<ScenarioStats>();
    assert!(
        stats.entered_playing,
        "expected entered_playing == true after SpawnNodeComplete"
    );
}

// -------------------------------------------------------------------------
// bypass_menu_to_playing — sets RunSeed from scenario config
// -------------------------------------------------------------------------

/// `bypass_menu_to_playing` must set `RunSeed` to `Some(0)` when the
/// scenario definition has `seed: None` (default 0 for determinism).
#[test]
fn bypass_menu_to_playing_sets_run_seed_default_zero() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioConfig {
            definition: make_scenario(100),
        })
        .insert_resource(breaker::shared::SelectedArchetype::default())
        .insert_resource(breaker::run::node::ScenarioLayoutOverride(None))
        .init_resource::<breaker::shared::RunSeed>()
        .add_plugins(StatesPlugin)
        .init_state::<GameState>()
        .add_systems(Update, bypass_menu_to_playing);

    app.update();

    let seed = app.world().resource::<breaker::shared::RunSeed>();
    assert_eq!(
        seed.0,
        Some(0),
        "expected RunSeed(Some(0)) when scenario seed is None, got {:?}",
        seed.0
    );
}

/// `bypass_menu_to_playing` must set `RunSeed` to `Some(42)` when the
/// scenario definition has `seed: Some(42)`.
#[test]
fn bypass_menu_to_playing_sets_run_seed_from_scenario() {
    let mut definition = make_scenario(100);
    definition.seed = Some(42);

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioConfig { definition })
        .insert_resource(breaker::shared::SelectedArchetype::default())
        .insert_resource(breaker::run::node::ScenarioLayoutOverride(None))
        .init_resource::<breaker::shared::RunSeed>()
        .add_plugins(StatesPlugin)
        .init_state::<GameState>()
        .add_systems(Update, bypass_menu_to_playing);

    app.update();

    let seed = app.world().resource::<breaker::shared::RunSeed>();
    assert_eq!(
        seed.0,
        Some(42),
        "expected RunSeed(Some(42)) from scenario seed, got {:?}",
        seed.0
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

// -------------------------------------------------------------------------
// bypass_menu_to_playing — populates ActiveChains from initial_overclocks
// -------------------------------------------------------------------------

/// When `initial_overclocks` is `Some` with one chain, `bypass_menu_to_playing`
/// must populate `ActiveChains` with that chain.
#[test]
fn bypass_menu_to_playing_inserts_active_overclocks_when_some() {
    use breaker::{behaviors::ActiveChains, chips::TriggerChain};

    let mut definition = make_scenario(100);
    definition.initial_overclocks = Some(vec![TriggerChain::Shockwave {
        base_range: 64.0,
        range_per_level: 0.0,
        stacks: 1,
        speed: 400.0,
    }]);

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioConfig { definition })
        .insert_resource(breaker::shared::SelectedArchetype::default())
        .insert_resource(breaker::run::node::ScenarioLayoutOverride(None))
        .init_resource::<breaker::shared::RunSeed>()
        .init_resource::<ActiveChains>()
        .add_plugins(StatesPlugin)
        .init_state::<GameState>()
        .add_systems(Update, bypass_menu_to_playing);

    app.update();

    let active = app.world().resource::<ActiveChains>();
    assert_eq!(
        active.0.len(),
        1,
        "expected ActiveChains to contain 1 chain when initial_overclocks is Some, got {}",
        active.0.len()
    );
    assert_eq!(
        active.0[0],
        TriggerChain::Shockwave {
            base_range: 64.0,
            range_per_level: 0.0,
            stacks: 1,
            speed: 400.0,
        },
        "expected ActiveChains[0] to be Shockwave"
    );
}

/// When `initial_overclocks` is `None`, `bypass_menu_to_playing` must leave
/// `ActiveChains` at its default (empty vec).
#[test]
fn bypass_menu_to_playing_leaves_active_overclocks_empty_when_none() {
    use breaker::behaviors::ActiveChains;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioConfig {
            definition: make_scenario(100),
        })
        .insert_resource(breaker::shared::SelectedArchetype::default())
        .insert_resource(breaker::run::node::ScenarioLayoutOverride(None))
        .init_resource::<breaker::shared::RunSeed>()
        .init_resource::<ActiveChains>()
        .add_plugins(StatesPlugin)
        .init_state::<GameState>()
        .add_systems(Update, bypass_menu_to_playing);

    app.update();

    let active = app.world().resource::<ActiveChains>();
    assert!(
        active.0.is_empty(),
        "expected ActiveChains to be empty when initial_overclocks is None, got {} entries",
        active.0.len()
    );
}

// -------------------------------------------------------------------------
// Invariant checker gating — entered_playing
// -------------------------------------------------------------------------

/// Invariant checkers must NOT produce violations when
/// `ScenarioStats::entered_playing` is `false`. This simulates the
/// `GameState::Loading` phase where entities may not be fully initialized.
///
/// Given: `entered_playing = false`, bolt at (0.0, 999.0) — well above
/// the top bound (350.0 for a 700.0-height playfield). Despite the bolt
/// being clearly out of bounds, the checker must NOT fire because the
/// game has not yet entered `Playing`.
#[test]
fn invariant_checkers_do_not_fire_when_entered_playing_is_false() {
    use crate::invariants::{ScenarioStats, ScenarioTagBolt, check_bolt_in_bounds};

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ViolationLog::default())
        .insert_resource(ScenarioFrame(1))
        .insert_resource(breaker::shared::PlayfieldConfig {
            width: 800.0,
            height: 700.0,
            background_color_rgb: [0.0, 0.0, 0.0],
            wall_thickness: 180.0,
            zone_fraction: 0.667,
        })
        .insert_resource(ScenarioStats {
            entered_playing: false,
            ..Default::default()
        })
        .add_systems(FixedUpdate, check_bolt_in_bounds);

    // Bolt at y = 999.0 is well above top bound (350.0). Without the
    // entered_playing gate this would fire a BoltInBounds violation.
    app.world_mut()
        .spawn((ScenarioTagBolt, Position2D(Vec2::new(0.0, 999.0))));

    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert!(
        log.0.is_empty(),
        "expected no violations when entered_playing is false, but got {}: {:?}",
        log.0.len(),
        log.0.iter().map(|e| &e.message).collect::<Vec<_>>()
    );
}

/// Invariant checkers MUST produce violations when
/// `ScenarioStats::entered_playing` is `true` and a bolt is out of bounds.
///
/// Given: `entered_playing = true`, bolt at (0.0, 999.0) — above the top
/// bound (350.0 for a 700.0-height playfield).
///
/// This is the control test that confirms the checker fires normally
/// when the gate condition is met.
#[test]
fn invariant_checkers_fire_when_entered_playing_is_true() {
    use crate::invariants::{ScenarioStats, ScenarioTagBolt, check_bolt_in_bounds};

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ViolationLog::default())
        .insert_resource(ScenarioFrame(1))
        .insert_resource(breaker::shared::PlayfieldConfig {
            width: 800.0,
            height: 700.0,
            background_color_rgb: [0.0, 0.0, 0.0],
            wall_thickness: 180.0,
            zone_fraction: 0.667,
        })
        .insert_resource(ScenarioStats {
            entered_playing: true,
            ..Default::default()
        })
        .add_systems(FixedUpdate, check_bolt_in_bounds);

    // Bolt at y = 999.0 is above top bound (350.0) — should produce a violation.
    app.world_mut()
        .spawn((ScenarioTagBolt, Position2D(Vec2::new(0.0, 999.0))));

    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert!(
        !log.0.is_empty(),
        "expected at least one BoltInBounds violation when entered_playing is true and bolt is OOB"
    );
    assert_eq!(
        log.0[0].invariant,
        InvariantKind::BoltInBounds,
        "expected BoltInBounds invariant kind"
    );
}

/// Invariant checkers must remain gated across multiple frames while
/// `entered_playing` is `false`. Even after 5 ticks with an OOB bolt,
/// the `ViolationLog` must stay empty.
#[test]
fn invariant_checkers_remain_gated_across_multiple_frames_while_not_playing() {
    use crate::invariants::{ScenarioStats, ScenarioTagBolt, check_bolt_in_bounds};

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ViolationLog::default())
        .insert_resource(ScenarioFrame(1))
        .insert_resource(breaker::shared::PlayfieldConfig {
            width: 800.0,
            height: 700.0,
            background_color_rgb: [0.0, 0.0, 0.0],
            wall_thickness: 180.0,
            zone_fraction: 0.667,
        })
        .insert_resource(ScenarioStats {
            entered_playing: false,
            ..Default::default()
        })
        .add_systems(FixedUpdate, check_bolt_in_bounds);

    // Bolt far above top bound — would fire if not gated
    app.world_mut()
        .spawn((ScenarioTagBolt, Position2D(Vec2::new(0.0, 999.0))));

    for _ in 0..5 {
        tick(&mut app);
    }

    let log = app.world().resource::<ViolationLog>();
    assert!(
        log.0.is_empty(),
        "expected no violations after 5 ticks with entered_playing=false, but got {}: {:?}",
        log.0.len(),
        log.0.iter().map(|e| &e.message).collect::<Vec<_>>()
    );
}

// -------------------------------------------------------------------------
// apply_debug_setup — sets BoltVelocity when bolt_velocity is Some
// -------------------------------------------------------------------------

/// When `debug_setup` has `bolt_velocity: Some((0.0, 2000.0))`, `apply_debug_setup`
/// must set `BoltVelocity.value` to `Vec2::new(0.0, 2000.0)` on the tagged bolt.
#[test]
fn apply_debug_setup_sets_bolt_velocity_when_some() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        invariants: vec![],
        expected_violations: None,
        debug_setup: Some(DebugSetup {
            bolt_velocity: Some((0.0, 2000.0)),
            ..Default::default()
        }),
        invariant_params: InvariantParams::default(),
        allow_early_end: true,
        stress: None,
        seed: None,
        initial_overclocks: None,
        frame_mutations: None,
    };

    let mut app = debug_setup_app(definition);
    app.add_systems(Update, apply_debug_setup);

    let entity = app
        .world_mut()
        .spawn((
            ScenarioTagBolt,
            Position2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    app.update();
    app.update();

    let vel = app
        .world()
        .entity(entity)
        .get::<Velocity2D>()
        .expect("entity must still have Velocity2D");
    assert_eq!(
        vel.0,
        Vec2::new(0.0, 2000.0),
        "expected Velocity2D.0 == (0.0, 2000.0), got {:?}",
        vel.0
    );
}

/// When `debug_setup` has `bolt_velocity: None`, `BoltVelocity` must remain unchanged.
#[test]
fn apply_debug_setup_leaves_bolt_velocity_unchanged_when_none() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        invariants: vec![],
        expected_violations: None,
        debug_setup: Some(DebugSetup {
            bolt_velocity: None,
            ..Default::default()
        }),
        invariant_params: InvariantParams::default(),
        allow_early_end: true,
        stress: None,
        seed: None,
        initial_overclocks: None,
        frame_mutations: None,
    };

    let mut app = debug_setup_app(definition);
    app.add_systems(Update, apply_debug_setup);

    let entity = app
        .world_mut()
        .spawn((
            ScenarioTagBolt,
            Position2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    app.update();
    app.update();

    let vel = app
        .world()
        .entity(entity)
        .get::<Velocity2D>()
        .expect("entity must still have Velocity2D");
    assert_eq!(
        vel.0,
        Vec2::new(0.0, 400.0),
        "expected Velocity2D unchanged at (0.0, 400.0), got {:?}",
        vel.0
    );
}

// -------------------------------------------------------------------------
// apply_debug_setup — sets BoltVelocity on ALL tagged bolts
// -------------------------------------------------------------------------

/// When `bolt_velocity: Some((100.0, 200.0))`, ALL tagged bolts must get
/// the overridden velocity, not just the first one.
#[test]
fn apply_debug_setup_sets_bolt_velocity_on_all_tagged_bolts() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        invariants: vec![],
        expected_violations: None,
        debug_setup: Some(DebugSetup {
            bolt_velocity: Some((100.0, 200.0)),
            ..Default::default()
        }),
        invariant_params: InvariantParams::default(),
        allow_early_end: true,
        stress: None,
        seed: None,
        initial_overclocks: None,
        frame_mutations: None,
    };

    let mut app = debug_setup_app(definition);
    app.add_systems(Update, apply_debug_setup);

    let e1 = app
        .world_mut()
        .spawn((
            ScenarioTagBolt,
            Position2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();
    let e2 = app
        .world_mut()
        .spawn((
            ScenarioTagBolt,
            Position2D(Vec2::new(10.0, 10.0)),
            Velocity2D(Vec2::new(300.0, 0.0)),
        ))
        .id();
    let e3 = app
        .world_mut()
        .spawn((
            ScenarioTagBolt,
            Position2D(Vec2::new(20.0, 20.0)),
            Velocity2D(Vec2::new(-100.0, -100.0)),
        ))
        .id();

    app.update();
    app.update();

    let expected = Vec2::new(100.0, 200.0);
    for (label, entity) in [("bolt1", e1), ("bolt2", e2), ("bolt3", e3)] {
        let vel = app
            .world()
            .entity(entity)
            .get::<Velocity2D>()
            .unwrap_or_else(|| panic!("{label} must still have BoltVelocity"));
        assert_eq!(
            vel.0, expected,
            "{label}: expected BoltVelocity.value == {expected:?}, got {:?}",
            vel.0
        );
    }
}

// -------------------------------------------------------------------------
// apply_debug_setup — spawns extra tagged bolts
// -------------------------------------------------------------------------

/// When `extra_tagged_bolts: Some(5)`, `apply_debug_setup` must spawn 5 extra
/// `ScenarioTagBolt` entities. Combined with the 1 existing tagged bolt, the
/// total must be 6. The extra entities must NOT have `Bolt`, `BoltVelocity`,
/// `BoltMinSpeed`, or `BoltMaxSpeed` components.
#[test]
fn apply_debug_setup_spawns_extra_tagged_bolts() {
    use breaker::bolt::components::{BoltMaxSpeed, BoltMinSpeed};

    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        invariants: vec![],
        expected_violations: None,
        debug_setup: Some(DebugSetup {
            extra_tagged_bolts: Some(5),
            ..Default::default()
        }),
        invariant_params: InvariantParams::default(),
        allow_early_end: true,
        stress: None,
        seed: None,
        initial_overclocks: None,
        frame_mutations: None,
    };

    let mut app = debug_setup_app(definition);
    app.add_systems(Update, apply_debug_setup);

    // Spawn one existing tagged bolt
    app.world_mut()
        .spawn((ScenarioTagBolt, Position2D(Vec2::new(0.0, 0.0))));

    // Single update: apply_debug_setup runs as OnEnter in production (fires once).
    // Two updates would run the system twice, doubling the spawned count.
    app.update();

    // Count all ScenarioTagBolt entities
    let tagged_count = app
        .world_mut()
        .query_filtered::<Entity, With<ScenarioTagBolt>>()
        .iter(app.world())
        .count();
    assert_eq!(
        tagged_count, 6,
        "expected 6 total ScenarioTagBolt entities (1 original + 5 extra), got {tagged_count}"
    );

    // Verify extra bolts do NOT have physics components
    let bolts_with_bolt_component = app
        .world_mut()
        .query_filtered::<Entity, (With<ScenarioTagBolt>, With<Bolt>)>()
        .iter(app.world())
        .count();
    assert_eq!(
        bolts_with_bolt_component, 0,
        "extra tagged bolts must NOT have Bolt component, found {bolts_with_bolt_component}"
    );

    let bolts_with_velocity = app
        .world_mut()
        .query_filtered::<Entity, (With<ScenarioTagBolt>, With<Velocity2D>)>()
        .iter(app.world())
        .count();
    assert_eq!(
        bolts_with_velocity, 0,
        "extra tagged bolts must NOT have Velocity2D component, found {bolts_with_velocity}"
    );

    let bolts_with_min_speed = app
        .world_mut()
        .query_filtered::<Entity, (With<ScenarioTagBolt>, With<BoltMinSpeed>)>()
        .iter(app.world())
        .count();
    assert_eq!(
        bolts_with_min_speed, 0,
        "extra tagged bolts must NOT have BoltMinSpeed component, found {bolts_with_min_speed}"
    );

    let bolts_with_max_speed = app
        .world_mut()
        .query_filtered::<Entity, (With<ScenarioTagBolt>, With<BoltMaxSpeed>)>()
        .iter(app.world())
        .count();
    assert_eq!(
        bolts_with_max_speed, 0,
        "extra tagged bolts must NOT have BoltMaxSpeed component, found {bolts_with_max_speed}"
    );
}

// -------------------------------------------------------------------------
// apply_debug_setup — spawns zero extra bolts when Some(0)
// -------------------------------------------------------------------------

/// When `extra_tagged_bolts: Some(0)`, no extra entities should be spawned.
/// Total `ScenarioTagBolt` count remains 1.
#[test]
fn apply_debug_setup_spawns_zero_extra_bolts_when_some_zero() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        invariants: vec![],
        expected_violations: None,
        debug_setup: Some(DebugSetup {
            extra_tagged_bolts: Some(0),
            ..Default::default()
        }),
        invariant_params: InvariantParams::default(),
        allow_early_end: true,
        stress: None,
        seed: None,
        initial_overclocks: None,
        frame_mutations: None,
    };

    let mut app = debug_setup_app(definition);
    app.add_systems(Update, apply_debug_setup);

    app.world_mut()
        .spawn((ScenarioTagBolt, Position2D(Vec2::new(0.0, 0.0))));

    app.update();
    app.update();

    let tagged_count = app
        .world_mut()
        .query_filtered::<Entity, With<ScenarioTagBolt>>()
        .iter(app.world())
        .count();
    assert_eq!(
        tagged_count, 1,
        "expected 1 ScenarioTagBolt entity (no extras from Some(0)), got {tagged_count}"
    );
}

// -------------------------------------------------------------------------
// apply_debug_setup — sets NodeTimer.remaining
// -------------------------------------------------------------------------

/// When `node_timer_remaining: Some(-1.0)`, `apply_debug_setup` must set
/// `NodeTimer.remaining` to -1.0.
#[test]
fn apply_debug_setup_sets_node_timer_remaining() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        invariants: vec![],
        expected_violations: None,
        debug_setup: Some(DebugSetup {
            node_timer_remaining: Some(-1.0),
            ..Default::default()
        }),
        invariant_params: InvariantParams::default(),
        allow_early_end: true,
        stress: None,
        seed: None,
        initial_overclocks: None,
        frame_mutations: None,
    };

    let mut app = debug_setup_app(definition);
    app.add_systems(Update, apply_debug_setup);
    app.world_mut().insert_resource(NodeTimer {
        remaining: 60.0,
        total: 60.0,
    });

    app.update();
    app.update();

    let timer = app.world().resource::<NodeTimer>();
    assert!(
        (timer.remaining - (-1.0_f32)).abs() < f32::EPSILON,
        "expected NodeTimer.remaining == -1.0, got {}",
        timer.remaining
    );
}

// -------------------------------------------------------------------------
// apply_debug_setup — ignores node_timer_remaining when no NodeTimer
// -------------------------------------------------------------------------

/// When `node_timer_remaining: Some(-1.0)` but no `NodeTimer` resource is
/// present, `apply_debug_setup` must not panic.
#[test]
fn apply_debug_setup_ignores_node_timer_remaining_when_no_resource() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        invariants: vec![],
        expected_violations: None,
        debug_setup: Some(DebugSetup {
            node_timer_remaining: Some(-1.0),
            ..Default::default()
        }),
        invariant_params: InvariantParams::default(),
        allow_early_end: true,
        stress: None,
        seed: None,
        initial_overclocks: None,
        frame_mutations: None,
    };

    let mut app = debug_setup_app(definition);
    app.add_systems(Update, apply_debug_setup);
    // Deliberately do NOT insert NodeTimer

    // Must not panic
    app.update();
    app.update();
}

// -------------------------------------------------------------------------
// apply_debug_setup — sets PreviousGameState from force_previous_game_state
// -------------------------------------------------------------------------

/// When `force_previous_game_state: Some(ForcedGameState::Loading)`,
/// `apply_debug_setup` must set `PreviousGameState.0` to `Some(GameState::Loading)`.
#[test]
fn apply_debug_setup_sets_previous_game_state_from_forced() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        invariants: vec![],
        expected_violations: None,
        debug_setup: Some(DebugSetup {
            force_previous_game_state: Some(ForcedGameState::Loading),
            ..Default::default()
        }),
        invariant_params: InvariantParams::default(),
        allow_early_end: true,
        stress: None,
        seed: None,
        initial_overclocks: None,
        frame_mutations: None,
    };

    let mut app = debug_setup_app(definition);
    app.add_systems(Update, apply_debug_setup);
    app.init_resource::<PreviousGameState>();

    app.update();
    app.update();

    let prev = app.world().resource::<PreviousGameState>();
    assert_eq!(
        prev.0,
        Some(GameState::Loading),
        "expected PreviousGameState.0 == Some(GameState::Loading), got {:?}",
        prev.0
    );
}

// -------------------------------------------------------------------------
// map_forced_game_state — maps all variants correctly
// -------------------------------------------------------------------------

/// Each `ForcedGameState` variant must map 1:1 to the corresponding `GameState` variant.
#[test]
fn map_forced_game_state_maps_all_variants_correctly() {
    let mappings = [
        (ForcedGameState::Loading, GameState::Loading),
        (ForcedGameState::MainMenu, GameState::MainMenu),
        (ForcedGameState::RunSetup, GameState::RunSetup),
        (ForcedGameState::Playing, GameState::Playing),
        (ForcedGameState::TransitionOut, GameState::TransitionOut),
        (ForcedGameState::TransitionIn, GameState::TransitionIn),
        (ForcedGameState::ChipSelect, GameState::ChipSelect),
        (ForcedGameState::RunEnd, GameState::RunEnd),
        (ForcedGameState::MetaProgression, GameState::MetaProgression),
    ];
    for (forced, expected) in &mappings {
        let result = map_forced_game_state(*forced);
        assert_eq!(
            result, *expected,
            "map_forced_game_state({forced:?}) must return {expected:?}, got {result:?}"
        );
    }
}

// -------------------------------------------------------------------------
// map_scenario_breaker_state — maps all variants 1:1
// -------------------------------------------------------------------------

/// Each `ScenarioBreakerState` variant must map 1:1 to the corresponding
/// `BreakerState` variant.
#[test]
fn map_scenario_breaker_state_maps_all_variants() {
    let mappings = [
        (ScenarioBreakerState::Idle, BreakerState::Idle),
        (ScenarioBreakerState::Dashing, BreakerState::Dashing),
        (ScenarioBreakerState::Braking, BreakerState::Braking),
        (ScenarioBreakerState::Settling, BreakerState::Settling),
    ];
    for (scenario, expected) in &mappings {
        let result = map_scenario_breaker_state(*scenario);
        assert_eq!(
            result, *expected,
            "map_scenario_breaker_state({scenario:?}) must return {expected:?}, got {result:?}"
        );
    }
}

// -------------------------------------------------------------------------
// apply_debug_frame_mutations — no-op when frame_mutations is None
// -------------------------------------------------------------------------

/// When `frame_mutations` is `None`, `apply_debug_frame_mutations` must
/// do nothing and not panic at any frame.
#[test]
fn apply_debug_frame_mutations_noop_when_none() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        invariants: vec![],
        expected_violations: None,
        debug_setup: None,
        invariant_params: InvariantParams::default(),
        allow_early_end: true,
        stress: None,
        seed: None,
        initial_overclocks: None,
        frame_mutations: None,
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioConfig { definition })
        .insert_resource(ScenarioFrame(5))
        .add_systems(Update, apply_debug_frame_mutations);

    // Must not panic
    app.update();
}

// -------------------------------------------------------------------------
// apply_debug_frame_mutations — SetBreakerState at matching frame
// -------------------------------------------------------------------------

/// When `frame_mutations` has `SetBreakerState(Braking)` at frame 3 and
/// the current frame is 3, the breaker entity's `BreakerState` must
/// become `BreakerState::Braking`.
#[test]
fn apply_debug_frame_mutations_set_breaker_state_at_matching_frame() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        invariants: vec![],
        expected_violations: None,
        debug_setup: None,
        invariant_params: InvariantParams::default(),
        allow_early_end: true,
        stress: None,
        seed: None,
        initial_overclocks: None,
        frame_mutations: Some(vec![FrameMutation {
            frame: 3,
            mutation: MutationKind::SetBreakerState(ScenarioBreakerState::Braking),
        }]),
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioConfig { definition })
        .insert_resource(ScenarioFrame(3))
        .add_systems(Update, apply_debug_frame_mutations);

    let entity = app
        .world_mut()
        .spawn((ScenarioTagBreaker, BreakerState::Idle))
        .id();

    app.update();

    let state = app
        .world()
        .entity(entity)
        .get::<BreakerState>()
        .expect("entity must still have BreakerState");
    assert_eq!(
        *state,
        BreakerState::Braking,
        "expected BreakerState::Braking at frame 3, got {state:?}"
    );
}

// -------------------------------------------------------------------------
// apply_debug_frame_mutations — SetBreakerState does NOT apply at non-matching frame
// -------------------------------------------------------------------------

/// When `frame_mutations` has `SetBreakerState(Braking)` at frame 3 but
/// the current frame is 2, the breaker must remain `Idle`.
#[test]
fn apply_debug_frame_mutations_set_breaker_state_skips_non_matching_frame() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        invariants: vec![],
        expected_violations: None,
        debug_setup: None,
        invariant_params: InvariantParams::default(),
        allow_early_end: true,
        stress: None,
        seed: None,
        initial_overclocks: None,
        frame_mutations: Some(vec![FrameMutation {
            frame: 3,
            mutation: MutationKind::SetBreakerState(ScenarioBreakerState::Braking),
        }]),
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioConfig { definition })
        .insert_resource(ScenarioFrame(2))
        .add_systems(Update, apply_debug_frame_mutations);

    let entity = app
        .world_mut()
        .spawn((ScenarioTagBreaker, BreakerState::Idle))
        .id();

    app.update();

    let state = app
        .world()
        .entity(entity)
        .get::<BreakerState>()
        .expect("entity must still have BreakerState");
    assert_eq!(
        *state,
        BreakerState::Idle,
        "expected BreakerState::Idle at frame 2 (mutation at frame 3), got {state:?}"
    );
}

// -------------------------------------------------------------------------
// apply_debug_frame_mutations — SetTimerRemaining at matching frame
// -------------------------------------------------------------------------

/// When `frame_mutations` has `SetTimerRemaining(61.0)` at frame 5 and
/// the current frame is 5, `NodeTimer.remaining` must be set to 61.0.
#[test]
fn apply_debug_frame_mutations_set_timer_remaining_at_matching_frame() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        invariants: vec![],
        expected_violations: None,
        debug_setup: None,
        invariant_params: InvariantParams::default(),
        allow_early_end: true,
        stress: None,
        seed: None,
        initial_overclocks: None,
        frame_mutations: Some(vec![FrameMutation {
            frame: 5,
            mutation: MutationKind::SetTimerRemaining(61.0),
        }]),
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioConfig { definition })
        .insert_resource(ScenarioFrame(5))
        .insert_resource(NodeTimer {
            remaining: 55.0,
            total: 60.0,
        })
        .add_systems(Update, apply_debug_frame_mutations);

    app.update();

    let timer = app.world().resource::<NodeTimer>();
    assert!(
        (timer.remaining - 61.0_f32).abs() < f32::EPSILON,
        "expected NodeTimer.remaining == 61.0, got {}",
        timer.remaining
    );
}

// -------------------------------------------------------------------------
// apply_debug_frame_mutations — SetTimerRemaining no-op when no NodeTimer
// -------------------------------------------------------------------------

/// When `frame_mutations` has `SetTimerRemaining(61.0)` at frame 5 but
/// no `NodeTimer` resource exists, the system must not panic.
#[test]
fn apply_debug_frame_mutations_set_timer_remaining_noop_when_no_timer() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        invariants: vec![],
        expected_violations: None,
        debug_setup: None,
        invariant_params: InvariantParams::default(),
        allow_early_end: true,
        stress: None,
        seed: None,
        initial_overclocks: None,
        frame_mutations: Some(vec![FrameMutation {
            frame: 5,
            mutation: MutationKind::SetTimerRemaining(61.0),
        }]),
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioConfig { definition })
        .insert_resource(ScenarioFrame(5))
        .add_systems(Update, apply_debug_frame_mutations);

    // Deliberately do NOT insert NodeTimer — must not panic
    app.update();
}

// -------------------------------------------------------------------------
// apply_debug_frame_mutations — SpawnExtraEntities at matching frame
// -------------------------------------------------------------------------

/// When `frame_mutations` has `SpawnExtraEntities(5)` at frame 10 and
/// the current frame is 10, 5 new entities with `Transform` must be spawned.
#[test]
fn apply_debug_frame_mutations_spawn_extra_entities_at_matching_frame() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        invariants: vec![],
        expected_violations: None,
        debug_setup: None,
        invariant_params: InvariantParams::default(),
        allow_early_end: true,
        stress: None,
        seed: None,
        initial_overclocks: None,
        frame_mutations: Some(vec![FrameMutation {
            frame: 10,
            mutation: MutationKind::SpawnExtraEntities(5),
        }]),
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioConfig { definition })
        .insert_resource(ScenarioFrame(10))
        .add_systems(Update, apply_debug_frame_mutations);

    // Single update only — avoid double-spawn from running the system twice
    app.update();

    let count = app
        .world_mut()
        .query_filtered::<Entity, With<Transform>>()
        .iter(app.world())
        .count();
    assert_eq!(
        count, 5,
        "expected 5 entities with Transform from SpawnExtraEntities(5), got {count}"
    );
}

// -------------------------------------------------------------------------
// apply_debug_frame_mutations — MoveBolt at matching frame
// -------------------------------------------------------------------------

/// When `frame_mutations` has `MoveBolt(999.0, 999.0)` at frame 5 and
/// the current frame is 5, the tagged bolt must be moved to (999.0, 999.0).
#[test]
fn apply_debug_frame_mutations_move_bolt_at_matching_frame() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        invariants: vec![],
        expected_violations: None,
        debug_setup: None,
        invariant_params: InvariantParams::default(),
        allow_early_end: true,
        stress: None,
        seed: None,
        initial_overclocks: None,
        frame_mutations: Some(vec![FrameMutation {
            frame: 5,
            mutation: MutationKind::MoveBolt(999.0, 999.0),
        }]),
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioConfig { definition })
        .insert_resource(ScenarioFrame(5))
        .add_systems(Update, apply_debug_frame_mutations);

    let entity = app
        .world_mut()
        .spawn((ScenarioTagBolt, Position2D(Vec2::new(0.0, 0.0))))
        .id();

    app.update();

    let position = app
        .world()
        .entity(entity)
        .get::<Position2D>()
        .expect("entity must still have Position2D");
    assert_eq!(
        position.0,
        Vec2::new(999.0, 999.0),
        "expected bolt at (999.0, 999.0), got {:?}",
        position.0
    );
}

// -------------------------------------------------------------------------
// apply_debug_frame_mutations — TogglePause sets NextState to Paused
// -------------------------------------------------------------------------

/// When `frame_mutations` has `TogglePause` at frame 3, the current frame
/// is 3, and the game is in `PlayingState::Active`, the system must set
/// `NextState<PlayingState>` to `Paused`.
#[test]
fn apply_debug_frame_mutations_toggle_pause_sets_paused() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        invariants: vec![],
        expected_violations: None,
        debug_setup: None,
        invariant_params: InvariantParams::default(),
        allow_early_end: true,
        stress: None,
        seed: None,
        initial_overclocks: None,
        frame_mutations: Some(vec![FrameMutation {
            frame: 3,
            mutation: MutationKind::TogglePause,
        }]),
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(StatesPlugin)
        .init_state::<GameState>()
        .add_sub_state::<PlayingState>()
        .insert_resource(ScenarioConfig { definition })
        .insert_resource(ScenarioFrame(3))
        .add_systems(Update, apply_debug_frame_mutations);

    // Drive into GameState::Playing so PlayingState becomes active.
    // Single update: state transition activates PlayingState, then the mutation
    // system runs (frame=3 matches) and sets NextState<PlayingState> to Paused.
    // A second update would process that transition, then the system would toggle
    // back to Active (because TogglePause runs again at the same frame).
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Playing);
    app.update();

    // Check that NextState<PlayingState> is set to Paused
    let next = app.world().resource::<NextState<PlayingState>>();
    assert!(
        format!("{next:?}").contains("Paused"),
        "expected NextState<PlayingState> to contain Paused after TogglePause, got: {next:?}"
    );
}

// -------------------------------------------------------------------------
// apply_debug_frame_mutations — multiple mutations on same frame
// -------------------------------------------------------------------------

/// When two mutations are scheduled for the same frame, both must apply.
/// Given `SetBreakerState(Braking)` and `MoveBolt(100.0, 200.0)` both at
/// frame 5, the breaker state must change AND the bolt must move.
#[test]
fn apply_debug_frame_mutations_multiple_mutations_on_same_frame() {
    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        invariants: vec![],
        expected_violations: None,
        debug_setup: None,
        invariant_params: InvariantParams::default(),
        allow_early_end: true,
        stress: None,
        seed: None,
        initial_overclocks: None,
        frame_mutations: Some(vec![
            FrameMutation {
                frame: 5,
                mutation: MutationKind::SetBreakerState(ScenarioBreakerState::Braking),
            },
            FrameMutation {
                frame: 5,
                mutation: MutationKind::MoveBolt(100.0, 200.0),
            },
        ]),
    };

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioConfig { definition })
        .insert_resource(ScenarioFrame(5))
        .add_systems(Update, apply_debug_frame_mutations);

    let breaker_entity = app
        .world_mut()
        .spawn((ScenarioTagBreaker, BreakerState::Idle))
        .id();
    let bolt_entity = app
        .world_mut()
        .spawn((ScenarioTagBolt, Position2D(Vec2::new(0.0, 0.0))))
        .id();

    app.update();

    let state = app
        .world()
        .entity(breaker_entity)
        .get::<BreakerState>()
        .expect("breaker entity must still have BreakerState");
    assert_eq!(
        *state,
        BreakerState::Braking,
        "expected BreakerState::Braking from first mutation, got {state:?}"
    );

    let position = app
        .world()
        .entity(bolt_entity)
        .get::<Position2D>()
        .expect("bolt entity must still have Position2D");
    assert_eq!(
        position.0,
        Vec2::new(100.0, 200.0),
        "expected bolt at (100.0, 200.0) from second mutation, got {:?}",
        position.0
    );
}

// -------------------------------------------------------------------------
// tick_scenario_frame — gated on entered_playing
// -------------------------------------------------------------------------

/// `tick_scenario_frame` must NOT increment `ScenarioFrame` when
/// `ScenarioStats::entered_playing` is `false`.
///
/// Given: `ScenarioStats { entered_playing: false }`, `ScenarioFrame(0)`,
///        `tick_scenario_frame` registered with `run_if(entered_playing)`.
/// When:  5 fixed-update ticks run.
/// Then:  `ScenarioFrame` is still 0.
///
/// Edge case: when `ScenarioStats` is absent (`Option` is `None`), the
/// guard must also block execution — frame should not tick.
#[test]
fn tick_scenario_frame_gated_before_playing_entered() {
    use crate::invariants::ScenarioStats;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioFrame(0))
        .insert_resource(ScenarioStats {
            entered_playing: false,
            ..Default::default()
        })
        .add_systems(FixedUpdate, tick_scenario_frame.run_if(entered_playing));

    for _ in 0..5 {
        tick(&mut app);
    }

    let frame = app.world().resource::<ScenarioFrame>();
    assert_eq!(
        frame.0, 0,
        "expected ScenarioFrame == 0 after 5 ticks with entered_playing=false, got {}",
        frame.0
    );
}

/// When `ScenarioStats` is absent (not inserted as a resource), the
/// `entered_playing` guard must return `false` — `tick_scenario_frame`
/// must not run.
#[test]
fn tick_scenario_frame_gated_when_scenario_stats_absent() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioFrame(0))
        // Intentionally do NOT insert ScenarioStats
        .add_systems(FixedUpdate, tick_scenario_frame.run_if(entered_playing));

    for _ in 0..5 {
        tick(&mut app);
    }

    let frame = app.world().resource::<ScenarioFrame>();
    assert_eq!(
        frame.0, 0,
        "expected ScenarioFrame == 0 after 5 ticks with no ScenarioStats, got {}",
        frame.0
    );
}

// -------------------------------------------------------------------------
// tick_scenario_frame — ticks normally after Playing entered
// -------------------------------------------------------------------------

/// `tick_scenario_frame` must increment normally when
/// `ScenarioStats::entered_playing` is `true`.
///
/// Given: `ScenarioStats { entered_playing: true }`, `ScenarioFrame(0)`.
/// When:  3 fixed-update ticks run.
/// Then:  `ScenarioFrame` is 3.
#[test]
fn tick_scenario_frame_ticks_after_playing_entered() {
    use crate::invariants::ScenarioStats;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(ScenarioFrame(0))
        .insert_resource(ScenarioStats {
            entered_playing: true,
            ..Default::default()
        })
        .add_systems(FixedUpdate, tick_scenario_frame.run_if(entered_playing));

    for _ in 0..3 {
        tick(&mut app);
    }

    let frame = app.world().resource::<ScenarioFrame>();
    assert_eq!(
        frame.0, 3,
        "expected ScenarioFrame == 3 after 3 ticks with entered_playing=true, got {}",
        frame.0
    );
}

// -------------------------------------------------------------------------
// check_frame_limit — gated on entered_playing
// -------------------------------------------------------------------------

/// `check_frame_limit` must NOT send `AppExit` when
/// `ScenarioStats::entered_playing` is `false`, even when
/// `ScenarioFrame` exceeds `max_frames`.
///
/// Given: `ScenarioStats { entered_playing: false }`, `ScenarioFrame(0)`,
///        `max_frames: 5`, both `tick_scenario_frame` and
///        `check_frame_limit` registered with the `run_if(entered_playing)`
///        guard.
/// When:  10 fixed-update ticks run.
/// Then:  No `AppExit` message sent (frame never reached 5 because
///        `tick_scenario_frame` was also gated).
#[test]
fn check_frame_limit_gated_before_playing_entered() {
    use crate::invariants::ScenarioStats;

    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<AppExit>()
        .insert_resource(ScenarioFrame(0))
        .insert_resource(ScenarioConfig {
            definition: make_scenario(5),
        })
        .insert_resource(ScenarioStats {
            entered_playing: false,
            ..Default::default()
        })
        .init_resource::<ExitReceived>()
        .add_systems(
            FixedUpdate,
            (
                tick_scenario_frame.run_if(entered_playing),
                check_frame_limit.run_if(entered_playing),
                capture_exit,
            )
                .chain(),
        );

    for _ in 0..10 {
        tick(&mut app);
    }

    assert!(
        !app.world().resource::<ExitReceived>().0,
        "expected no AppExit when entered_playing=false, even after 10 ticks with max_frames=5"
    );

    let frame = app.world().resource::<ScenarioFrame>();
    assert_eq!(
        frame.0, 0,
        "expected ScenarioFrame == 0 (gated), got {}",
        frame.0
    );
}

// -------------------------------------------------------------------------
// Velocity2D migration — apply_debug_setup writes Velocity2D
// -------------------------------------------------------------------------

/// After migration, `apply_debug_setup` must write `Velocity2D` (not `BoltVelocity`)
/// when `bolt_velocity` is `Some(...)`.
///
/// Given: tagged bolt with `Velocity2D(0.0, 400.0)`, `debug_setup` `bolt_velocity: Some((0.0, 2000.0))`
/// When: `apply_debug_setup` runs
/// Then: `Velocity2D.0` == `Vec2::new(0.0, 2000.0)`
///
/// This test will FAIL until `apply_debug_setup` is updated to write `Velocity2D`.
#[test]
fn apply_debug_setup_writes_velocity2d_when_bolt_velocity_some() {
    use rantzsoft_spatial2d::components::Velocity2D;

    let definition = ScenarioDefinition {
        breaker: "Aegis".to_owned(),
        layout: "Corridor".to_owned(),
        input: InputStrategy::Scripted(ScriptedParams { actions: vec![] }),
        max_frames: 1000,
        invariants: vec![],
        expected_violations: None,
        debug_setup: Some(DebugSetup {
            bolt_velocity: Some((0.0, 2000.0)),
            ..Default::default()
        }),
        invariant_params: InvariantParams::default(),
        allow_early_end: true,
        stress: None,
        seed: None,
        initial_overclocks: None,
        frame_mutations: None,
    };

    let mut app = debug_setup_app(definition);
    app.add_systems(Update, apply_debug_setup);

    let entity = app
        .world_mut()
        .spawn((
            ScenarioTagBolt,
            Position2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ))
        .id();

    app.update();
    app.update();

    let vel = app
        .world()
        .entity(entity)
        .get::<Velocity2D>()
        .expect("entity must have Velocity2D");
    assert_eq!(
        vel.0,
        Vec2::new(0.0, 2000.0),
        "expected Velocity2D == (0.0, 2000.0), got {:?}",
        vel.0
    );
}
