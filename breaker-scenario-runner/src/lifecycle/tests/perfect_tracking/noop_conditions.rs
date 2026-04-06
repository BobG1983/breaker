use crate::lifecycle::tests::helpers::*;

/// Perfect tracking does not run when `InputDriver` is not `Perfect`.
#[test]
fn perfect_tracking_noop_when_driver_is_not_perfect() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<InputActions>()
        .init_resource::<PlayfieldConfig>();

    let driver = InputDriver::Chaos(crate::input::ChaosDriver::new(
        42,
        &ChaosParams { action_prob: 0.3 },
    ));
    app.insert_resource(ScenarioInputDriver(driver));
    app.insert_resource(ForceBumpGrade::default());

    app.add_systems(Update, apply_perfect_tracking);

    app.world_mut().spawn((
        ScenarioTagBolt,
        Position2D(Vec2::new(100.0, 50.0)),
        Velocity2D(Vec2::new(0.0, -400.0)),
    ));

    let breaker = app
        .world_mut()
        .spawn((
            ScenarioTagBreaker,
            Position2D(Vec2::new(0.0, -250.0)),
            BaseWidth(120.0),
        ))
        .id();

    app.update();

    let pos = app.world().entity(breaker).get::<Position2D>().unwrap();
    assert!(
        pos.0.x.abs() < f32::EPSILON,
        "expected breaker x unchanged when driver is Chaos, got {}",
        pos.0.x
    );
}

/// Perfect tracking is no-op when `ScenarioInputDriver` is absent.
#[test]
fn perfect_tracking_noop_when_driver_absent() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<InputActions>()
        .init_resource::<PlayfieldConfig>();
    // No ScenarioInputDriver inserted
    app.insert_resource(ForceBumpGrade::default());

    app.add_systems(Update, apply_perfect_tracking);

    app.world_mut().spawn((
        ScenarioTagBolt,
        Position2D(Vec2::new(100.0, 50.0)),
        Velocity2D(Vec2::new(0.0, -400.0)),
    ));

    let breaker = app
        .world_mut()
        .spawn((
            ScenarioTagBreaker,
            Position2D(Vec2::new(0.0, -250.0)),
            BaseWidth(120.0),
        ))
        .id();

    // Must not panic
    app.update();

    let pos = app.world().entity(breaker).get::<Position2D>().unwrap();
    assert!(
        pos.0.x.abs() < f32::EPSILON,
        "expected breaker x unchanged when driver absent, got {}",
        pos.0.x
    );
}

/// Perfect tracking with no bolt entities is a no-op.
#[test]
fn perfect_tracking_noop_with_no_bolt_entities() {
    let mut app = perfect_tracking_app(42, BumpMode::AlwaysPerfect);

    app.add_systems(Update, apply_perfect_tracking);

    // No bolt entities
    let breaker = app
        .world_mut()
        .spawn((
            ScenarioTagBreaker,
            Position2D(Vec2::new(0.0, -250.0)),
            BaseWidth(120.0),
        ))
        .id();

    app.update();

    let pos = app.world().entity(breaker).get::<Position2D>().unwrap();
    assert!(
        pos.0.x.abs() < f32::EPSILON,
        "expected breaker x unchanged when no bolts, got {}",
        pos.0.x
    );
}
