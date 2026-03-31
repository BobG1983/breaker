//! Tests for bolt velocity override via `apply_debug_setup`.

use super::super::helpers::*;

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
        disallowed_failures: vec![],
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
        disallowed_failures: vec![],
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
        disallowed_failures: vec![],
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
