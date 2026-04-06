use crate::lifecycle::tests::helpers::*;

/// Perfect tracking moves breaker x toward bolt x with random offset when
/// bolt moves downward (negative y velocity).
#[test]
fn perfect_tracking_moves_breaker_toward_bolt_when_bolt_descends() {
    let mut app = perfect_tracking_app(42, BumpMode::AlwaysPerfect);

    app.add_systems(Update, apply_perfect_tracking);

    // Bolt moving downward
    app.world_mut().spawn((
        ScenarioTagBolt,
        Position2D(Vec2::new(100.0, 50.0)),
        Velocity2D(Vec2::new(0.0, -400.0)),
    ));

    // Breaker at x=0
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
    let half_width = 60.0;
    let low = (-PERFECT_TRACKING_WIDTH_FACTOR).mul_add(half_width, 100.0);
    let high = PERFECT_TRACKING_WIDTH_FACTOR.mul_add(half_width, 100.0);
    assert!(
        pos.0.x >= low && pos.0.x <= high,
        "expected breaker x in [{low}, {high}], got {}",
        pos.0.x
    );
    assert!(
        (pos.0.y - (-250.0)).abs() < f32::EPSILON,
        "expected breaker y unchanged at -250.0, got {}",
        pos.0.y
    );
}

/// Perfect tracking repositions breaker under bolt even when bolt moves
/// upward — the breaker always tracks the bolt regardless of direction.
#[test]
fn perfect_tracking_moves_breaker_when_bolt_ascends() {
    let mut app = perfect_tracking_app(42, BumpMode::AlwaysPerfect);

    app.add_systems(Update, apply_perfect_tracking);

    app.world_mut().spawn((
        ScenarioTagBolt,
        Position2D(Vec2::new(100.0, 50.0)),
        Velocity2D(Vec2::new(0.0, 400.0)),
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
    let half_width = 60.0;
    let low = (-PERFECT_TRACKING_WIDTH_FACTOR).mul_add(half_width, 100.0);
    let high = PERFECT_TRACKING_WIDTH_FACTOR.mul_add(half_width, 100.0);
    assert!(
        pos.0.x >= low && pos.0.x <= high,
        "expected breaker x in [{low}, {high}], got {}",
        pos.0.x
    );
    assert!(
        (pos.0.y - (-250.0)).abs() < f32::EPSILON,
        "expected breaker y unchanged at -250.0, got {}",
        pos.0.y
    );
}

/// Edge case: bolt with zero y velocity (but non-zero x) still has breaker
/// tracked — the breaker always positions under the bolt regardless of direction.
#[test]
fn perfect_tracking_moves_breaker_when_bolt_y_velocity_zero() {
    let mut app = perfect_tracking_app(42, BumpMode::AlwaysPerfect);

    app.add_systems(Update, apply_perfect_tracking);

    app.world_mut().spawn((
        ScenarioTagBolt,
        Position2D(Vec2::new(100.0, 50.0)),
        Velocity2D(Vec2::new(50.0, 0.0)),
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
    let half_width = 60.0;
    let low = (-PERFECT_TRACKING_WIDTH_FACTOR).mul_add(half_width, 100.0);
    let high = PERFECT_TRACKING_WIDTH_FACTOR.mul_add(half_width, 100.0);
    assert!(
        pos.0.x >= low && pos.0.x <= high,
        "expected breaker x in [{low}, {high}], got {}",
        pos.0.x
    );
    assert!(
        (pos.0.y - (-250.0)).abs() < f32::EPSILON,
        "expected breaker y unchanged at -250.0, got {}",
        pos.0.y
    );
}
