//! Tests for bolt teleport and `ScenarioPhysicsFrozen` insertion via `apply_debug_setup`.

use super::super::helpers::*;

// -------------------------------------------------------------------------
// apply_debug_setup — teleport to bolt_position
// -------------------------------------------------------------------------

/// When `debug_setup` has `bolt_position: Some((0.0, -500.0))` and
/// `disable_physics: false`, `apply_debug_setup` must move the
/// `ScenarioTagBolt` entity's `Position2D` to `(0.0, -500.0)`.
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
// apply_debug_setup — inserts ScenarioPhysicsFrozen + disables physics
// -------------------------------------------------------------------------

/// When `disable_physics: true`, `apply_debug_setup` must insert
/// `ScenarioPhysicsFrozen` with `target = Vec2::new(0.0, -400.0)`.
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
