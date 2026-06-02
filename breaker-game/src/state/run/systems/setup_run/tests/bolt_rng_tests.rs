// Wave 2B Groups C and F — setup_run uses BoltRng

use bevy::prelude::*;
use rand::Rng;

use super::helpers::test_app;
use crate::{
    prelude::*,
    shared::rng::{BoltRng, GameRng, derive_seed, derive_seed_named},
    state::run::{NodeOutcome, resources::RunStats},
};

const SENTINEL: u64 = 0xDEAD_BEEF_CAFE_1234;

// ── Group C ───────────────────────────────────────────────────────────────────

// Behavior 7: setup_run reads ResMut<BoltRng> — harness has only BoltRng
#[test]
fn setup_run_uses_bolt_rng_not_game_rng_on_subsequent_node() {
    // test_app() helper now registers BoltRng instead of GameRng.
    // If setup_run still names GameRng, the app panics here.
    let mut app = test_app();
    app.world_mut().resource_mut::<NodeOutcome>().node_index = 1;
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("exactly one bolt should be spawned");

    let vel = app.world().get::<Velocity2D>(entity).unwrap();
    assert!(
        (vel.0.length() - 720.0).abs() < 2.0,
        "bolt speed should be ~720.0, got {}",
        vel.0.length()
    );
    assert!(vel.0.y > 0.0, "bolt should launch upward");
    assert!(
        app.world().get::<BoltServing>(entity).is_none(),
        "subsequent-node bolt must NOT carry BoltServing"
    );
}

// Behavior 7 edge case: BoltRng AND GameRng present — system must NOT advance GameRng
#[test]
fn setup_run_does_not_advance_game_rng_on_subsequent_node() {
    let mut app = test_app();
    app.insert_resource(GameRng::from_seed(SENTINEL));
    app.world_mut().resource_mut::<NodeOutcome>().node_index = 1;
    app.update();

    // Sentinel: bolt spawned proves system ran
    let entity_count = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .count();
    assert_eq!(entity_count, 1, "bolt should be spawned — system executed");

    let actual: u64 = app.world_mut().resource_mut::<GameRng>().0.random();
    let expected: u64 = GameRng::from_seed(SENTINEL).0.random();
    assert_eq!(
        actual, expected,
        "setup_run must NOT advance GameRng on subsequent-node path"
    );
}

// Behavior 8: same BoltRng seed yields same subsequent-node launch angle
#[test]
fn setup_run_same_seed_produces_identical_velocity_subsequent_node() {
    let seed = 0xCAFE_F00D_u64;

    let vel_a = {
        let mut app = test_app();
        app.insert_resource(BoltRng::from_seed(seed));
        app.world_mut().resource_mut::<NodeOutcome>().node_index = 1;
        app.update();
        let e = app
            .world_mut()
            .query_filtered::<Entity, With<Bolt>>()
            .iter(app.world())
            .next()
            .unwrap();
        *app.world().get::<Velocity2D>(e).unwrap()
    };

    let vel_b = {
        let mut app = test_app();
        app.insert_resource(BoltRng::from_seed(seed));
        app.world_mut().resource_mut::<NodeOutcome>().node_index = 1;
        app.update();
        let e = app
            .world_mut()
            .query_filtered::<Entity, With<Bolt>>()
            .iter(app.world())
            .next()
            .unwrap();
        *app.world().get::<Velocity2D>(e).unwrap()
    };

    assert!(
        (vel_a.0.x - vel_b.0.x).abs() < f32::EPSILON,
        "setup_run vx must be identical for same seed: a={}, b={}",
        vel_a.0.x,
        vel_b.0.x
    );
    assert!(
        (vel_a.0.y - vel_b.0.y).abs() < f32::EPSILON,
        "setup_run vy must be identical for same seed: a={}, b={}",
        vel_a.0.y,
        vel_b.0.y
    );
}

// Behavior 8 edge case: seed 0 vs seed 7 must produce different angles
#[test]
fn setup_run_different_seeds_produce_different_velocity_subsequent_node() {
    let launch_vel = |seed: u64| -> Velocity2D {
        let mut app = test_app();
        app.insert_resource(BoltRng::from_seed(seed));
        app.world_mut().resource_mut::<NodeOutcome>().node_index = 1;
        app.update();
        let e = app
            .world_mut()
            .query_filtered::<Entity, With<Bolt>>()
            .iter(app.world())
            .next()
            .unwrap();
        *app.world().get::<Velocity2D>(e).unwrap()
    };

    let vel_0 = launch_vel(0);
    let vel_7 = launch_vel(7);
    let diff_x = (vel_0.0.x - vel_7.0.x).abs();
    let diff_y = (vel_0.0.y - vel_7.0.y).abs();
    assert!(
        diff_x > 0.01 || diff_y > 0.01,
        "seeds 0 and 7 must produce different velocities (diff_x={diff_x:.4}, diff_y={diff_y:.4})"
    );
}

// Behavior 9: setup_run does NOT draw from BoltRng on first-node serving path
#[test]
fn setup_run_does_not_advance_bolt_rng_on_first_node() {
    let mut app = test_app();
    app.insert_resource(BoltRng::from_seed(42));
    // node_index defaults to 0
    app.update();

    let e = app
        .world_mut()
        .query_filtered::<Entity, With<Bolt>>()
        .iter(app.world())
        .next()
        .expect("bolt should be spawned on node 0");

    assert!(
        app.world().get::<BoltServing>(e).is_some(),
        "first-node bolt must carry BoltServing"
    );

    let vel = app.world().get::<Velocity2D>(e).unwrap();
    assert!(
        vel.0 == Vec2::ZERO,
        "first-node bolt velocity should be zero, got {:?}",
        vel.0
    );

    let actual: u64 = app.world_mut().resource_mut::<BoltRng>().0.random();
    let fresh_first: u64 = BoltRng::from_seed(42).0.random();
    assert_eq!(
        actual, fresh_first,
        "BoltRng must NOT advance on first-node serving path (node_index=0)"
    );
}

// Behavior 9 edge case: Breaker already present — guard fires, BoltRng un-advanced
#[test]
fn setup_run_does_not_advance_bolt_rng_when_breaker_already_present() {
    let mut app = test_app();
    app.insert_resource(BoltRng::from_seed(42));
    // Pre-spawn a breaker so the guard branch fires
    app.world_mut()
        .spawn((Breaker, Position2D(Vec2::new(0.0, -250.0))));
    app.update();

    let actual: u64 = app.world_mut().resource_mut::<BoltRng>().0.random();
    let fresh_first: u64 = BoltRng::from_seed(42).0.random();
    assert_eq!(
        actual, fresh_first,
        "BoltRng must NOT advance when the guard branch fires (Breaker already present)"
    );
}

// ── Group F (Behavior 15) ─────────────────────────────────────────────────────

// Behavior 15: setup_run with production formula seed produces stable velocity
#[test]
fn setup_run_with_production_bolt_rng_seed_is_deterministic() {
    let bolt_rng_seed = derive_seed_named(derive_seed(42, 1u64), "bolt");

    let vel_a = {
        let mut app = test_app();
        app.insert_resource(BoltRng::from_seed(bolt_rng_seed));
        app.world_mut().resource_mut::<NodeOutcome>().node_index = 1;
        app.update();
        let e = app
            .world_mut()
            .query_filtered::<Entity, With<Bolt>>()
            .iter(app.world())
            .next()
            .unwrap();
        *app.world().get::<Velocity2D>(e).unwrap()
    };

    let vel_b = {
        let mut app = test_app();
        app.insert_resource(BoltRng::from_seed(bolt_rng_seed));
        app.world_mut().resource_mut::<NodeOutcome>().node_index = 1;
        app.update();
        let e = app
            .world_mut()
            .query_filtered::<Entity, With<Bolt>>()
            .iter(app.world())
            .next()
            .unwrap();
        *app.world().get::<Velocity2D>(e).unwrap()
    };

    assert!(
        (vel_a.0.x - vel_b.0.x).abs() < f32::EPSILON,
        "setup_run with production formula seed must be deterministic (vx mismatch)"
    );
    assert!(
        (vel_a.0.y - vel_b.0.y).abs() < f32::EPSILON,
        "setup_run with production formula seed must be deterministic (vy mismatch)"
    );
}

// Behavior 15 edge case A: different node_index yields different velocity
#[test]
fn setup_run_different_node_index_yields_different_velocity_with_production_seed() {
    let vel_for_node = |node_index: u32| -> Velocity2D {
        let bolt_rng_seed = derive_seed_named(derive_seed(42, u64::from(node_index)), "bolt");
        let mut app = test_app();
        app.insert_resource(BoltRng::from_seed(bolt_rng_seed));
        app.world_mut().resource_mut::<NodeOutcome>().node_index = node_index;
        app.update();
        let e = app
            .world_mut()
            .query_filtered::<Entity, With<Bolt>>()
            .iter(app.world())
            .next()
            .unwrap();
        *app.world().get::<Velocity2D>(e).unwrap()
    };

    let vel_1 = vel_for_node(1);
    let vel_2 = vel_for_node(2);
    let diff_x = (vel_1.0.x - vel_2.0.x).abs();
    let diff_y = (vel_1.0.y - vel_2.0.y).abs();
    assert!(
        diff_x > 0.01 || diff_y > 0.01,
        "node_index=1 and node_index=2 must produce different velocities with production formula \
         (diff_x={diff_x:.4}, diff_y={diff_y:.4})"
    );
}

// Behavior 15 edge case B: different run seed yields different velocity
#[test]
fn setup_run_different_run_seed_yields_different_velocity_with_production_formula() {
    let vel_for_run_seed = |run_seed: u64| -> Velocity2D {
        let bolt_rng_seed = derive_seed_named(derive_seed(run_seed, 1u64), "bolt");
        let mut app = test_app();
        app.insert_resource(BoltRng::from_seed(bolt_rng_seed));
        app.world_mut().resource_mut::<NodeOutcome>().node_index = 1;
        app.update();
        let e = app
            .world_mut()
            .query_filtered::<Entity, With<Bolt>>()
            .iter(app.world())
            .next()
            .unwrap();
        *app.world().get::<Velocity2D>(e).unwrap()
    };

    let vel_42 = vel_for_run_seed(42);
    let vel_100 = vel_for_run_seed(100);
    let diff_x = (vel_42.0.x - vel_100.0.x).abs();
    let diff_y = (vel_42.0.y - vel_100.0.y).abs();
    assert!(
        diff_x > 0.01 || diff_y > 0.01,
        "run_seed=42 and run_seed=100 must produce different velocities \
         (diff_x={diff_x:.4}, diff_y={diff_y:.4})"
    );
}
