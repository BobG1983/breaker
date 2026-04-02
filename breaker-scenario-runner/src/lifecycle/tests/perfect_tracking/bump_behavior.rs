use super::super::helpers::*;

/// Perfect tracking writes Bump when bolt is within threshold of breaker.
#[test]
fn perfect_tracking_writes_bump_when_bolt_within_threshold() {
    let mut app = perfect_tracking_app(42, BumpMode::AlwaysPerfect);

    app.add_systems(Update, apply_perfect_tracking);

    // Bolt at y=-240, breaker at y=-250 => distance=10 <= 20 threshold
    app.world_mut().spawn((
        ScenarioTagBolt,
        Position2D(Vec2::new(0.0, -240.0)),
        Velocity2D(Vec2::new(0.0, -400.0)),
    ));
    app.world_mut().spawn((
        ScenarioTagBreaker,
        Position2D(Vec2::new(0.0, -250.0)),
        BaseWidth(120.0),
    ));

    app.update();

    let actions = app.world().resource::<InputActions>();
    assert!(
        actions.active(breaker::input::resources::GameAction::Bump),
        "expected Bump in InputActions when bolt within threshold"
    );
}

/// Edge case: bolt at distance 20.1 (just beyond threshold) does NOT write Bump.
#[test]
fn perfect_tracking_does_not_write_bump_when_bolt_beyond_threshold() {
    let mut app = perfect_tracking_app(42, BumpMode::AlwaysPerfect);

    app.add_systems(Update, apply_perfect_tracking);

    // Bolt at y=-229.9, breaker at y=-250 => distance=20.1 > 20.0
    app.world_mut().spawn((
        ScenarioTagBolt,
        Position2D(Vec2::new(0.0, -229.9)),
        Velocity2D(Vec2::new(0.0, -400.0)),
    ));
    app.world_mut().spawn((
        ScenarioTagBreaker,
        Position2D(Vec2::new(0.0, -250.0)),
        BaseWidth(120.0),
    ));

    app.update();

    let actions = app.world().resource::<InputActions>();
    assert!(
        !actions.active(breaker::input::resources::GameAction::Bump),
        "expected no Bump when bolt is beyond threshold (distance=20.1)"
    );
}

/// `NeverBump` mode does NOT write Bump even when bolt is within threshold,
/// but breaker IS still repositioned under the bolt.
#[test]
fn perfect_tracking_never_bump_mode_does_not_write_bump() {
    let mut app = perfect_tracking_app(42, BumpMode::NeverBump);

    app.add_systems(Update, apply_perfect_tracking);

    app.world_mut().spawn((
        ScenarioTagBolt,
        Position2D(Vec2::new(0.0, -245.0)),
        Velocity2D(Vec2::new(0.0, -400.0)),
    ));
    // Initial x = 200.0 is well outside the expected repositioning range
    // [-48.0, 48.0] for bolt at x=0.0 with BaseWidth(120.0) and
    // PERFECT_TRACKING_WIDTH_FACTOR = 0.8.
    let breaker = app
        .world_mut()
        .spawn((
            ScenarioTagBreaker,
            Position2D(Vec2::new(200.0, -250.0)),
            BaseWidth(120.0),
        ))
        .id();

    app.update();

    let actions = app.world().resource::<InputActions>();
    assert!(
        !actions.active(breaker::input::resources::GameAction::Bump),
        "expected no Bump in NeverBump mode"
    );

    // Breaker must still be repositioned even though no bump is triggered.
    // Expected range: bolt.x +/- (0.8 * half_width) = 0.0 +/- 48.0.
    let pos = app.world().entity(breaker).get::<Position2D>().unwrap();
    assert!(
        pos.0.x >= -48.0 && pos.0.x <= 48.0,
        "expected breaker repositioned into [-48.0, 48.0] in NeverBump mode, got {}",
        pos.0.x
    );
}

/// `AlwaysWhiff` mode writes Bump when bolt is within threshold (breaker
/// attempts to bump), but `ForceBumpGrade` is set to `None` so the bump is
/// graded as a whiff. The breaker IS repositioned under the bolt.
#[test]
fn perfect_tracking_always_whiff_mode_writes_bump() {
    let mut app = perfect_tracking_app(42, BumpMode::AlwaysWhiff);

    app.add_systems(Update, apply_perfect_tracking);

    app.world_mut().spawn((
        ScenarioTagBolt,
        Position2D(Vec2::new(0.0, -245.0)),
        Velocity2D(Vec2::new(0.0, -400.0)),
    ));
    // Initial x = 200.0 is well outside the expected repositioning range
    // [-48.0, 48.0] for bolt at x=0.0 with BaseWidth(120.0) and
    // PERFECT_TRACKING_WIDTH_FACTOR = 0.8.
    let breaker = app
        .world_mut()
        .spawn((
            ScenarioTagBreaker,
            Position2D(Vec2::new(200.0, -250.0)),
            BaseWidth(120.0),
        ))
        .id();

    app.update();

    let actions = app.world().resource::<InputActions>();
    assert!(
        actions.active(breaker::input::resources::GameAction::Bump),
        "expected Bump in AlwaysWhiff mode — breaker attempts to bump, grade is whiff"
    );

    // Breaker must be repositioned under the bolt.
    // Expected range: bolt.x +/- (0.8 * half_width) = 0.0 +/- 48.0.
    let pos = app.world().entity(breaker).get::<Position2D>().unwrap();
    assert!(
        pos.0.x >= -48.0 && pos.0.x <= 48.0,
        "expected breaker repositioned into [-48.0, 48.0] in AlwaysWhiff mode, got {}",
        pos.0.x
    );
}
