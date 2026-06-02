// Wave 2B Group B — bolt_lost uses BoltRng

use bevy::prelude::*;
use rand::Rng;
use rantzsoft_spatial2d::components::Spatial2D;

use super::super::helpers::*;
use crate::{
    bolt::resources::DEFAULT_BOLT_SPAWN_OFFSET_Y,
    prelude::*,
    shared::{
        GameDrawLayer,
        rng::{BoltRng, GameRng},
    },
};

const SENTINEL: u64 = 0xDEAD_BEEF_CAFE_1234;

fn spawn_breaker_at(app: &mut App, x: f32, y: f32) {
    app.world_mut().spawn((
        Breaker,
        Position2D(Vec2::new(x, y)),
        Spatial2D,
        GameDrawLayer::Breaker,
    ));
}

// Behavior 4: bolt_lost reads ResMut<BoltRng> — harness has only BoltRng
#[test]
fn bolt_lost_uses_bolt_rng_not_game_rng() {
    // test_app() already uses BoltRng (helpers.rs updated). No GameRng registered.
    let mut app = test_app();
    let playfield = PlayfieldConfig::default();
    spawn_breaker_at(&mut app, 0.0, -250.0);

    spawn_bolt(
        &mut app,
        Vec2::new(0.0, playfield.bottom() - 100.0),
        Vec2::new(0.0, -400.0),
    );
    tick(&mut app);

    let vel_y = app
        .world_mut()
        .query_filtered::<&Velocity2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .unwrap()
        .0
        .y;
    let pos_y = app
        .world_mut()
        .query_filtered::<&Position2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .unwrap()
        .0
        .y;

    let expected_y = -250.0 + DEFAULT_BOLT_SPAWN_OFFSET_Y;
    assert!(
        (pos_y - expected_y).abs() < f32::EPSILON,
        "respawn position y should be {expected_y} (bolt_lost ran without panic), got {pos_y}"
    );
    assert!(vel_y > 0.0, "bolt should be respawned upward");
}

// Behavior 4 edge case: BoltRng AND GameRng present — system must NOT advance GameRng
#[test]
fn bolt_lost_does_not_advance_game_rng_when_both_resources_present() {
    let mut app = test_app();
    app.insert_resource(GameRng::from_seed(SENTINEL));

    let playfield = PlayfieldConfig::default();
    spawn_breaker_at(&mut app, 0.0, -250.0);
    spawn_bolt(
        &mut app,
        Vec2::new(0.0, playfield.bottom() - 100.0),
        Vec2::new(0.0, -400.0),
    );
    tick(&mut app);

    // Sentinel: respawn happened, proving system executed
    let vel = app
        .world_mut()
        .query_filtered::<&Velocity2D, With<Bolt>>()
        .iter(app.world())
        .next()
        .unwrap();
    assert!(vel.0.y > 0.0, "bolt respawned — system executed");

    // GameRng must be un-advanced
    let actual: u64 = app.world_mut().resource_mut::<GameRng>().0.random();
    let expected: u64 = GameRng::from_seed(SENTINEL).0.random();
    assert_eq!(
        actual, expected,
        "bolt_lost must NOT advance GameRng when BoltRng is the param"
    );
}

// Behavior 5: same BoltRng seed yields same respawn angle in two independent apps
#[test]
fn bolt_lost_same_seed_produces_identical_respawn_velocity() {
    let seed = 987_654_321_u64;
    let playfield = PlayfieldConfig::default();

    let vel_a = {
        let mut app = test_app();
        app.insert_resource(BoltRng::from_seed(seed));
        spawn_breaker_at(&mut app, 0.0, -250.0);
        spawn_bolt(
            &mut app,
            Vec2::new(0.0, playfield.bottom() - 100.0),
            Vec2::new(0.0, -400.0),
        );
        tick(&mut app);
        *app.world_mut()
            .query_filtered::<&Velocity2D, With<Bolt>>()
            .iter(app.world())
            .next()
            .unwrap()
    };

    let vel_b = {
        let mut app = test_app();
        app.insert_resource(BoltRng::from_seed(seed));
        spawn_breaker_at(&mut app, 0.0, -250.0);
        spawn_bolt(
            &mut app,
            Vec2::new(0.0, playfield.bottom() - 100.0),
            Vec2::new(0.0, -400.0),
        );
        tick(&mut app);
        *app.world_mut()
            .query_filtered::<&Velocity2D, With<Bolt>>()
            .iter(app.world())
            .next()
            .unwrap()
    };

    assert!(
        (vel_a.0.x - vel_b.0.x).abs() < f32::EPSILON,
        "respawn vx must be identical for same seed: a={}, b={}",
        vel_a.0.x,
        vel_b.0.x
    );
    assert!(
        (vel_a.0.y - vel_b.0.y).abs() < f32::EPSILON,
        "respawn vy must be identical for same seed: a={}, b={}",
        vel_a.0.y,
        vel_b.0.y
    );
}

// Behavior 5 edge case: seed 0 vs seed 1 must produce different angles
#[test]
fn bolt_lost_different_seeds_produce_different_respawn_velocity() {
    let playfield = PlayfieldConfig::default();

    let respawn_vel = |seed: u64| -> Velocity2D {
        let mut app = test_app();
        app.insert_resource(BoltRng::from_seed(seed));
        spawn_breaker_at(&mut app, 0.0, -250.0);
        spawn_bolt(
            &mut app,
            Vec2::new(0.0, playfield.bottom() - 100.0),
            Vec2::new(0.0, -400.0),
        );
        tick(&mut app);
        *app.world_mut()
            .query_filtered::<&Velocity2D, With<Bolt>>()
            .iter(app.world())
            .next()
            .unwrap()
    };

    let vel_0 = respawn_vel(0);
    let vel_1 = respawn_vel(1);
    let diff_x = (vel_0.0.x - vel_1.0.x).abs();
    let diff_y = (vel_0.0.y - vel_1.0.y).abs();
    assert!(
        diff_x > 0.01 || diff_y > 0.01,
        "seeds 0 and 1 must produce different respawn velocities (diff_x={diff_x:.4}, diff_y={diff_y:.4})"
    );
}

// Behavior 6: BoltRng does NOT advance when no bolt is lost (bolt is inside playfield)
#[test]
fn bolt_lost_does_not_advance_bolt_rng_when_no_bolt_lost() {
    let mut app = test_app();
    app.insert_resource(BoltRng::from_seed(42));
    spawn_breaker_at(&mut app, 0.0, -250.0);
    // Bolt is well inside the playfield — NOT below the floor
    spawn_bolt(&mut app, Vec2::new(0.0, 0.0), Vec2::new(0.0, 0.0));
    tick(&mut app);

    let actual: u64 = app.world_mut().resource_mut::<BoltRng>().0.random();
    let fresh_first: u64 = BoltRng::from_seed(42).0.random();
    assert_eq!(
        actual, fresh_first,
        "BoltRng must NOT advance when no bolt is below the floor"
    );
}

// Behavior 6 edge case: ExtraBolt below floor writes KillYourself but does NOT draw from BoltRng
#[test]
fn bolt_lost_does_not_advance_bolt_rng_for_extra_bolt_below_floor() {
    let mut app = test_app();
    app.insert_resource(BoltRng::from_seed(42));

    let playfield = PlayfieldConfig::default();
    spawn_breaker_at(&mut app, 0.0, -250.0);

    // Only an ExtraBolt is below the floor — the baseline respawn path is not taken
    let world = app.world_mut();
    let def = make_default_bolt_definition();
    Bolt::builder()
        .at_position(Vec2::new(0.0, playfield.bottom() - 100.0))
        .definition(&def)
        .with_velocity(Velocity2D(Vec2::new(0.0, -400.0)))
        .extra()
        .headless()
        .spawn(&mut world.commands());
    world.flush();

    tick(&mut app);

    let actual: u64 = app.world_mut().resource_mut::<BoltRng>().0.random();
    let fresh_first: u64 = BoltRng::from_seed(42).0.random();
    assert_eq!(
        actual, fresh_first,
        "BoltRng must NOT advance when only an ExtraBolt is below the floor (extra-bolt branch)"
    );
}
