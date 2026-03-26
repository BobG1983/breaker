use super::helpers::*;

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
        debug_setup: Some(DebugSetup {
            bolt_position: Some((0.0, -500.0)),
            breaker_position: None,
            disable_physics: false,
            ..Default::default()
        }),
        ..Default::default()
    };

    let mut app = debug_setup_app(definition);
    app.add_systems(Update, apply_debug_setup);

    let entity = app
        .world_mut()
        .spawn((
            ScenarioTagBolt,
            Position2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::ZERO),
        ))
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
        debug_setup: Some(DebugSetup {
            bolt_position: None,
            breaker_position: Some((100.0, -50.0)),
            disable_physics: false,
            ..Default::default()
        }),
        ..Default::default()
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
        debug_setup: Some(DebugSetup {
            bolt_position: Some((0.0, -400.0)),
            breaker_position: None,
            disable_physics: true,
            ..Default::default()
        }),
        ..Default::default()
    };

    let mut app = debug_setup_app(definition);
    app.add_systems(Update, apply_debug_setup);

    let entity = app
        .world_mut()
        .spawn((
            ScenarioTagBolt,
            Position2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::ZERO),
        ))
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
        debug_setup: Some(DebugSetup {
            bolt_velocity: Some((0.0, 2000.0)),
            ..Default::default()
        }),
        ..Default::default()
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
        debug_setup: Some(DebugSetup {
            bolt_velocity: None,
            ..Default::default()
        }),
        ..Default::default()
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
        debug_setup: Some(DebugSetup {
            bolt_velocity: Some((100.0, 200.0)),
            ..Default::default()
        }),
        ..Default::default()
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
        debug_setup: Some(DebugSetup {
            extra_tagged_bolts: Some(5),
            ..Default::default()
        }),
        ..Default::default()
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
        debug_setup: Some(DebugSetup {
            extra_tagged_bolts: Some(0),
            ..Default::default()
        }),
        ..Default::default()
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
        debug_setup: Some(DebugSetup {
            node_timer_remaining: Some(-1.0),
            ..Default::default()
        }),
        ..Default::default()
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
        debug_setup: Some(DebugSetup {
            node_timer_remaining: Some(-1.0),
            ..Default::default()
        }),
        ..Default::default()
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
        debug_setup: Some(DebugSetup {
            force_previous_game_state: Some(ForcedGameState::Loading),
            ..Default::default()
        }),
        ..Default::default()
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
