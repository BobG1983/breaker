use super::helpers::*;

// =========================================================================
// apply_perfect_tracking
// =========================================================================

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
            BreakerWidth(120.0),
        ))
        .id();

    app.update();

    let pos = app.world().entity(breaker).get::<Position2D>().unwrap();
    let half_width = 60.0;
    let low = 100.0 - PERFECT_TRACKING_WIDTH_FACTOR * half_width;
    let high = 100.0 + PERFECT_TRACKING_WIDTH_FACTOR * half_width;
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
            BreakerWidth(120.0),
        ))
        .id();

    app.update();

    let pos = app.world().entity(breaker).get::<Position2D>().unwrap();
    let half_width = 60.0;
    let low = 100.0 - PERFECT_TRACKING_WIDTH_FACTOR * half_width;
    let high = 100.0 + PERFECT_TRACKING_WIDTH_FACTOR * half_width;
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
            BreakerWidth(120.0),
        ))
        .id();

    app.update();

    let pos = app.world().entity(breaker).get::<Position2D>().unwrap();
    let half_width = 60.0;
    let low = 100.0 - PERFECT_TRACKING_WIDTH_FACTOR * half_width;
    let high = 100.0 + PERFECT_TRACKING_WIDTH_FACTOR * half_width;
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
        BreakerWidth(120.0),
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
        BreakerWidth(120.0),
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
    // [-48.0, 48.0] for bolt at x=0.0 with BreakerWidth(120.0) and
    // PERFECT_TRACKING_WIDTH_FACTOR = 0.8.
    let breaker = app
        .world_mut()
        .spawn((
            ScenarioTagBreaker,
            Position2D(Vec2::new(200.0, -250.0)),
            BreakerWidth(120.0),
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
    // [-48.0, 48.0] for bolt at x=0.0 with BreakerWidth(120.0) and
    // PERFECT_TRACKING_WIDTH_FACTOR = 0.8.
    let breaker = app
        .world_mut()
        .spawn((
            ScenarioTagBreaker,
            Position2D(Vec2::new(200.0, -250.0)),
            BreakerWidth(120.0),
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

/// Perfect tracking does not run when `InputDriver` is not `Perfect`.
#[test]
fn perfect_tracking_noop_when_driver_is_not_perfect() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<InputActions>();

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
            BreakerWidth(120.0),
        ))
        .id();

    app.update();

    let pos = app.world().entity(breaker).get::<Position2D>().unwrap();
    assert!(
        (pos.0.x - 0.0).abs() < f32::EPSILON,
        "expected breaker x unchanged when driver is Chaos, got {}",
        pos.0.x
    );
}

/// Perfect tracking is no-op when `ScenarioInputDriver` is absent.
#[test]
fn perfect_tracking_noop_when_driver_absent() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .init_resource::<InputActions>();
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
            BreakerWidth(120.0),
        ))
        .id();

    // Must not panic
    app.update();

    let pos = app.world().entity(breaker).get::<Position2D>().unwrap();
    assert!(
        (pos.0.x - 0.0).abs() < f32::EPSILON,
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
            BreakerWidth(120.0),
        ))
        .id();

    app.update();

    let pos = app.world().entity(breaker).get::<Position2D>().unwrap();
    assert!(
        (pos.0.x - 0.0).abs() < f32::EPSILON,
        "expected breaker x unchanged when no bolts, got {}",
        pos.0.x
    );
}

/// Perfect tracking with multiple bolts tracks the first bolt found.
#[test]
fn perfect_tracking_with_multiple_bolts_tracks_first() {
    let mut app = perfect_tracking_app(42, BumpMode::AlwaysPerfect);

    app.add_systems(Update, apply_perfect_tracking);

    // Two bolts, both descending
    app.world_mut().spawn((
        ScenarioTagBolt,
        Position2D(Vec2::new(100.0, 50.0)),
        Velocity2D(Vec2::new(0.0, -400.0)),
    ));
    app.world_mut().spawn((
        ScenarioTagBolt,
        Position2D(Vec2::new(-100.0, 50.0)),
        Velocity2D(Vec2::new(0.0, -400.0)),
    ));

    let breaker = app
        .world_mut()
        .spawn((
            ScenarioTagBreaker,
            Position2D(Vec2::new(0.0, -250.0)),
            BreakerWidth(120.0),
        ))
        .id();

    app.update();

    let pos = app.world().entity(breaker).get::<Position2D>().unwrap();
    let half_width = 60.0;
    // Must be in range of one of the bolts
    let tracks_bolt_a =
        pos.0.x >= (100.0 - 0.8 * half_width) && pos.0.x <= (100.0 + 0.8 * half_width);
    let tracks_bolt_b =
        pos.0.x >= (-100.0 - 0.8 * half_width) && pos.0.x <= (-100.0 + 0.8 * half_width);
    assert!(
        tracks_bolt_a || tracks_bolt_b,
        "expected breaker x in range of bolt A or B, got {}",
        pos.0.x
    );
}

/// Two apps with identical seed and entity layout must produce the exact
/// same breaker position, proving deterministic bolt selection.
#[test]
fn perfect_tracking_with_multiple_bolts_is_deterministic() {
    /// Build an app with two bolts and one breaker, all at the same positions
    /// and using the same seed. Returns the app and the breaker entity id.
    fn build_multi_bolt_app(seed: u64) -> (App, Entity) {
        let mut app = perfect_tracking_app(seed, BumpMode::AlwaysPerfect);

        app.add_systems(Update, apply_perfect_tracking);

        // Two bolts, both descending — same positions in both apps
        app.world_mut().spawn((
            ScenarioTagBolt,
            Position2D(Vec2::new(100.0, 50.0)),
            Velocity2D(Vec2::new(0.0, -400.0)),
        ));
        app.world_mut().spawn((
            ScenarioTagBolt,
            Position2D(Vec2::new(-100.0, 50.0)),
            Velocity2D(Vec2::new(0.0, -400.0)),
        ));

        // Initial x = 300.0 is well outside both bolts' repositioning ranges,
        // so the system MUST move the breaker for the test to pass.
        let breaker = app
            .world_mut()
            .spawn((
                ScenarioTagBreaker,
                Position2D(Vec2::new(300.0, -250.0)),
                BreakerWidth(120.0),
            ))
            .id();

        (app, breaker)
    }

    let (mut app_a, breaker_a) = build_multi_bolt_app(42);
    let (mut app_b, breaker_b) = build_multi_bolt_app(42);

    app_a.update();
    app_b.update();

    let pos_a = app_a
        .world()
        .entity(breaker_a)
        .get::<Position2D>()
        .unwrap()
        .0
        .x;
    let pos_b = app_b
        .world()
        .entity(breaker_b)
        .get::<Position2D>()
        .unwrap()
        .0
        .x;

    // First verify the system actually repositioned the breaker (not still at initial 300.0).
    let half_width = 60.0;
    let tracks_bolt_a = pos_a >= (100.0 - 0.8 * half_width) && pos_a <= (100.0 + 0.8 * half_width);
    let tracks_bolt_b =
        pos_a >= (-100.0 - 0.8 * half_width) && pos_a <= (-100.0 + 0.8 * half_width);
    assert!(
        tracks_bolt_a || tracks_bolt_b,
        "expected breaker repositioned into bolt A or B range, got {pos_a}"
    );

    // Then verify both runs produced the exact same position (determinism).
    assert!(
        (pos_a - pos_b).abs() < f32::EPSILON,
        "expected deterministic breaker position, got {pos_a} vs {pos_b}"
    );
}
