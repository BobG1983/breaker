use crate::lifecycle::tests::helpers::*;

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
            BaseWidth(120.0),
        ))
        .id();

    app.update();

    let pos = app.world().entity(breaker).get::<Position2D>().unwrap();
    let half_width = 60.0;
    // Must be in range of one of the bolts
    let tracks_bolt_a = pos.0.x >= (-0.8f32).mul_add(half_width, 100.0)
        && pos.0.x <= 0.8f32.mul_add(half_width, 100.0);
    let tracks_bolt_b = pos.0.x >= (-0.8f32).mul_add(half_width, -100.0)
        && pos.0.x <= 0.8f32.mul_add(half_width, -100.0);
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
                BaseWidth(120.0),
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
    let tracks_bolt_a =
        pos_a >= (-0.8f32).mul_add(half_width, 100.0) && pos_a <= 0.8f32.mul_add(half_width, 100.0);
    let tracks_bolt_b = pos_a >= (-0.8f32).mul_add(half_width, -100.0)
        && pos_a <= 0.8f32.mul_add(half_width, -100.0);
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
