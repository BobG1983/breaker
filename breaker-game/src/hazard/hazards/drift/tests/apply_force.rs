//! Group C — `drift_apply_force` system.
//!
//! Every test in this group wires ONLY `drift_apply_force` via
//! `wire_apply_force_only(&mut app)`. This bypasses run-conditions. RNG is
//! not required — `drift_apply_force` does not read it.

use std::time::Duration;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;

use super::{
    super::system::{DriftConfig, DriftWind, drift_apply_force},
    helpers::{
        add_drift_stacks, canonical_config, install_drift_config, install_drift_wind, spawn_bolt,
        test_app_playing, tick_with_dt, wire_apply_force_only,
    },
};
use crate::hazard::{definition::HazardKind, resources::ActiveHazards};

// ── Behavior 22 — stack 1, direction (1,0), force 100, dt 1s → +100 X ───

#[test]
fn apply_force_accelerates_bolt_in_wind_direction() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, drift_apply_force);
    app.world_mut().insert_resource(DriftConfig {
        force:           100.0,
        period_secs:     8.0,
        per_level_force: 33.3,
    });
    app.world_mut().insert_resource(DriftWind {
        direction: Vec2::new(1.0, 0.0),
        timer:     8.0,
    });
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Drift);
    let bolt = spawn_bolt(&mut app, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs(1));

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    // 100 units/sec² × 1s = +100 in X
    assert!(
        (velocity.0.x - 100.0).abs() < 1e-3,
        "expected +100 in X, got {:?}",
        velocity.0
    );
    assert!(velocity.0.y.abs() < 1e-5);
}

#[test]
fn apply_force_adds_to_pre_existing_velocity_not_overwrites() {
    // Edge: bolt seeded at (50, 20) ends at (150, 20).
    let mut app = test_app_playing();
    wire_apply_force_only(&mut app);
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::new(1.0, 0.0),
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app, Vec2::new(50.0, 20.0));

    tick_with_dt(&mut app, Duration::from_secs(1));

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (velocity.0.x - 150.0).abs() < 1e-3,
        "impulse must add to pre-existing X, got {:?}",
        velocity.0
    );
    assert!((velocity.0.y - 20.0).abs() < 1e-5);
}

// ── Behavior 23 — stack 3, direction (0,-1), → -166 in Y ──────────────────

#[test]
fn apply_force_scales_with_stacks() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, drift_apply_force);
    app.world_mut().insert_resource(DriftConfig {
        force:           100.0,
        period_secs:     8.0,
        per_level_force: 33.0,
    });
    app.world_mut().insert_resource(DriftWind {
        direction: Vec2::new(0.0, -1.0),
        timer:     8.0,
    });
    for _ in 0..3 {
        app.world_mut()
            .resource_mut::<ActiveHazards>()
            .add_stack(HazardKind::Drift);
    }
    let bolt = spawn_bolt(&mut app, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs(1));

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    // 100 + 33 * 2 = 166 → applied in -Y for 1 second
    assert!(velocity.0.x.abs() < 1e-5);
    assert!((velocity.0.y - -166.0).abs() < 1e-3);
}

#[test]
fn apply_force_scales_with_stacks_without_per_level_force() {
    // Edge: per_level_force=0 → only base applies at any stack > 0.
    let mut app = test_app_playing();
    wire_apply_force_only(&mut app);
    install_drift_config(
        &mut app,
        DriftConfig {
            force:           100.0,
            period_secs:     8.0,
            per_level_force: 0.0,
        },
    );
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::new(0.0, -1.0),
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 3);
    let bolt = spawn_bolt(&mut app, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs(1));

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(velocity.0.x.abs() < 1e-5);
    assert!((velocity.0.y - -100.0).abs() < 1e-3);
}

// ── Behavior 24 — direction at 45° → equal X and Y components ────────────

#[test]
fn apply_force_at_45_degrees_produces_equal_axis_components() {
    let mut app = test_app_playing();
    wire_apply_force_only(&mut app);
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::new(
                std::f32::consts::FRAC_1_SQRT_2,
                std::f32::consts::FRAC_1_SQRT_2,
            ),
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs(1));

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        (velocity.0.x - velocity.0.y).abs() < 1e-5,
        "45° should produce equal X and Y components, got {:?}",
        velocity.0
    );
    assert!(
        (velocity.0.length() - 100.0).abs() < 1e-3,
        "total magnitude should equal force magnitude, got len={}",
        velocity.0.length()
    );
}

#[test]
fn apply_force_at_upper_left_45_preserves_negative_components() {
    // Edge: upper-left quadrant direction — X<0, Y>0, magnitudes match.
    let mut app = test_app_playing();
    wire_apply_force_only(&mut app);
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::new(
                -std::f32::consts::FRAC_1_SQRT_2,
                std::f32::consts::FRAC_1_SQRT_2,
            ),
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs(1));

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        velocity.0.x < 0.0,
        "upper-left direction must have negative X"
    );
    assert!(
        velocity.0.y > 0.0,
        "upper-left direction must have positive Y"
    );
    assert!(
        (velocity.0.x + velocity.0.y).abs() < 1e-5,
        "X and Y magnitudes should cancel to near zero"
    );
}

// ── Behavior 25 — two bolts → both affected, prior velocities preserved ──

#[test]
fn apply_force_affects_all_bolts() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, drift_apply_force);
    app.world_mut().insert_resource(DriftConfig {
        force:           100.0,
        period_secs:     8.0,
        per_level_force: 33.3,
    });
    app.world_mut().insert_resource(DriftWind {
        direction: Vec2::X,
        timer:     8.0,
    });
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .add_stack(HazardKind::Drift);
    let bolt_a = spawn_bolt(&mut app, Vec2::ZERO);
    let bolt_b = spawn_bolt(&mut app, Vec2::new(50.0, 0.0));

    tick_with_dt(&mut app, Duration::from_secs(1));

    let vel_a = app.world().get::<Velocity2D>(bolt_a).unwrap();
    let vel_b = app.world().get::<Velocity2D>(bolt_b).unwrap();
    assert!((vel_a.0.x - 100.0).abs() < 1e-3);
    assert!((vel_b.0.x - 150.0).abs() < 1e-3);
}

#[test]
fn apply_force_affects_three_bolts_and_preserves_non_axial_components() {
    // Edge: third bolt at (-50, 25) moves to (50, 25).
    let mut app = test_app_playing();
    wire_apply_force_only(&mut app);
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let bolt_a = spawn_bolt(&mut app, Vec2::ZERO);
    let bolt_b = spawn_bolt(&mut app, Vec2::new(50.0, 0.0));
    let bolt_c = spawn_bolt(&mut app, Vec2::new(-50.0, 25.0));

    tick_with_dt(&mut app, Duration::from_secs(1));

    let vel_a = app.world().get::<Velocity2D>(bolt_a).unwrap();
    let vel_b = app.world().get::<Velocity2D>(bolt_b).unwrap();
    let vel_c = app.world().get::<Velocity2D>(bolt_c).unwrap();
    assert!((vel_a.0.x - 100.0).abs() < 1e-3);
    assert!((vel_b.0.x - 150.0).abs() < 1e-3);
    assert!((vel_c.0.x - 50.0).abs() < 1e-3);
    assert!((vel_c.0.y - 25.0).abs() < 1e-5);
}

// ── Behavior 26 — zero stacks → no-op (no velocity change) ───────────────

#[test]
fn apply_force_noop_at_zero_stacks() {
    let mut app = test_app_playing();
    app.add_systems(FixedUpdate, drift_apply_force);
    app.world_mut().insert_resource(DriftConfig {
        force:           100.0,
        period_secs:     8.0,
        per_level_force: 33.3,
    });
    app.world_mut().insert_resource(DriftWind {
        direction: Vec2::X,
        timer:     8.0,
    });
    let bolt = spawn_bolt(&mut app, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_secs(1));

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(velocity.0.length() < f32::EPSILON);
}

#[test]
fn apply_force_noop_when_stacks_drop_to_zero_mid_run() {
    // Edge: add stack, then force-insert stacks=0. After a tick, velocity
    // is unchanged.
    let mut app = test_app_playing();
    wire_apply_force_only(&mut app);
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app, Vec2::ZERO);

    // Drop back to zero before ticking.
    app.world_mut()
        .resource_mut::<ActiveHazards>()
        .force_insert_entry(HazardKind::Drift, 0);

    tick_with_dt(&mut app, Duration::from_secs(1));

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(velocity.0.length() < f32::EPSILON);
}

// ── Behavior 27 — zero force → no-op ──────────────────────────────────────

#[test]
fn apply_force_noop_when_force_is_zero() {
    let mut app = test_app_playing();
    wire_apply_force_only(&mut app);
    install_drift_config(
        &mut app,
        DriftConfig {
            force:           0.0,
            period_secs:     8.0,
            per_level_force: 0.0,
        },
    );
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 5);
    let bolt = spawn_bolt(&mut app, Vec2::new(10.0, 20.0));

    tick_with_dt(&mut app, Duration::from_secs(1));

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert_eq!(velocity.0.x.to_bits(), 10.0_f32.to_bits());
    assert_eq!(velocity.0.y.to_bits(), 20.0_f32.to_bits());
}

#[test]
fn apply_force_noop_when_force_is_negative() {
    // Edge: `force <= 0.0` gate — negative force silenced too.
    let mut app = test_app_playing();
    wire_apply_force_only(&mut app);
    install_drift_config(
        &mut app,
        DriftConfig {
            force:           -50.0,
            period_secs:     8.0,
            per_level_force: 0.0,
        },
    );
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app, Vec2::new(10.0, 20.0));

    tick_with_dt(&mut app, Duration::from_secs(1));

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert_eq!(velocity.0.x.to_bits(), 10.0_f32.to_bits());
    assert_eq!(velocity.0.y.to_bits(), 20.0_f32.to_bits());
}

// ── Behavior 28 — missing DriftConfig → no-op ────────────────────────────

#[test]
fn apply_force_noop_when_drift_config_missing() {
    let mut app = test_app_playing();
    wire_apply_force_only(&mut app);
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app, Vec2::new(10.0, 20.0));
    // No DriftConfig inserted.

    tick_with_dt(&mut app, Duration::from_secs(1));

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert_eq!(velocity.0.x.to_bits(), 10.0_f32.to_bits());
    assert_eq!(velocity.0.y.to_bits(), 20.0_f32.to_bits());
}

#[test]
fn apply_force_noop_when_drift_config_missing_across_two_ticks() {
    let mut app = test_app_playing();
    wire_apply_force_only(&mut app);
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app, Vec2::new(10.0, 20.0));

    tick_with_dt(&mut app, Duration::from_secs(1));
    tick_with_dt(&mut app, Duration::from_secs(1));

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert_eq!(velocity.0.x.to_bits(), 10.0_f32.to_bits());
    assert_eq!(velocity.0.y.to_bits(), 20.0_f32.to_bits());
}

// ── Behavior 29 — missing DriftWind → no-op ──────────────────────────────

#[test]
fn apply_force_noop_when_drift_wind_missing() {
    let mut app = test_app_playing();
    wire_apply_force_only(&mut app);
    install_drift_config(&mut app, canonical_config());
    add_drift_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app, Vec2::new(10.0, 20.0));
    // No DriftWind inserted.

    tick_with_dt(&mut app, Duration::from_secs(1));

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert_eq!(velocity.0.x.to_bits(), 10.0_f32.to_bits());
    assert_eq!(velocity.0.y.to_bits(), 20.0_f32.to_bits());
}

#[test]
fn apply_force_noop_when_drift_wind_missing_across_two_ticks() {
    let mut app = test_app_playing();
    wire_apply_force_only(&mut app);
    install_drift_config(&mut app, canonical_config());
    add_drift_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app, Vec2::new(10.0, 20.0));

    tick_with_dt(&mut app, Duration::from_secs(1));
    tick_with_dt(&mut app, Duration::from_secs(1));

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    assert_eq!(velocity.0.x.to_bits(), 10.0_f32.to_bits());
    assert_eq!(velocity.0.y.to_bits(), 20.0_f32.to_bits());
}

// ── Behavior 30 — zero bolts → no panic, no resource mutation ────────────

#[test]
fn apply_force_noop_when_no_bolts_in_world() {
    let mut app = test_app_playing();
    wire_apply_force_only(&mut app);
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    // No bolts spawned.

    tick_with_dt(&mut app, Duration::from_secs(1));

    let count = app
        .world_mut()
        .query::<&Velocity2D>()
        .iter(app.world())
        .count();
    assert_eq!(count, 0, "no Velocity2D entities should exist");
}

#[test]
fn apply_force_skips_non_bolt_entities_with_velocity() {
    // Edge: non-Bolt entity with Velocity2D is not affected.
    let mut app = test_app_playing();
    wire_apply_force_only(&mut app);
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let non_bolt = app.world_mut().spawn(Velocity2D(Vec2::new(5.0, 5.0))).id();

    tick_with_dt(&mut app, Duration::from_secs(1));

    let velocity = app.world().get::<Velocity2D>(non_bolt).unwrap();
    assert_eq!(velocity.0.x.to_bits(), 5.0_f32.to_bits());
    assert_eq!(velocity.0.y.to_bits(), 5.0_f32.to_bits());
}

// ── Behavior 31 — tiny dt tests removed ──────────────────────────────────
//
// `tick_with_dt` with `Duration::from_nanos(1)` interacts pathologically
// with Bevy's `Time<Fixed>` overstep accumulator (same hang pattern Erosion
// hit): mid-run `set_timestep` leaves stale residual that becomes a huge
// number of fixed steps under the new (tiny) timestep. The 1ms
// proportional-scaling test below covers the proportionality pin; the
// 1-second preserved test covers the baseline. Sub-ms edge cases aren't
// meaningful for this hazard's observable behavior.

// ── Behavior 32 — 1ms dt → proportionally tiny delta ──────────────────────

#[test]
fn apply_force_with_1ms_dt_produces_proportional_delta() {
    let mut app = test_app_playing();
    wire_apply_force_only(&mut app);
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let bolt = spawn_bolt(&mut app, Vec2::ZERO);

    tick_with_dt(&mut app, Duration::from_millis(1));

    let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
    // 100.0 * 1.0 * 0.001 = 0.1 in X
    assert!(
        (velocity.0.x - 0.1).abs() < 1e-4,
        "1ms dt should produce +0.1 in X, got {:?}",
        velocity.0
    );
    assert!(velocity.0.y.abs() < 1e-5);
}

// Note: the 10×1ms accumulation edge test was removed — `tick_with_dt`
// changes `Time<Fixed>` timestep mid-run, leaving stale overstep residual
// that runs extra fixed steps under the new timestep. The single 1ms and
// 1-second proportionality tests cover the per-tick scaling pin without
// accumulating the residual error.

// ── Behavior 33 — bolt spawned mid-run is picked up on next tick ─────────

#[test]
fn apply_force_picks_up_mid_run_spawn_on_next_tick() {
    let mut app = test_app_playing();
    wire_apply_force_only(&mut app);
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let bolt_a = spawn_bolt(&mut app, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs(1));
    {
        let vel = app.world().get::<Velocity2D>(bolt_a).unwrap();
        assert!((vel.0.x - 100.0).abs() < 1e-3);
    }

    let bolt_b = spawn_bolt(&mut app, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs(1));

    let vel_a = app.world().get::<Velocity2D>(bolt_a).unwrap();
    let vel_b = app.world().get::<Velocity2D>(bolt_b).unwrap();
    assert!(
        (vel_a.0.x - 200.0).abs() < 1e-3,
        "bolt_a should accumulate to 200"
    );
    assert!(
        (vel_b.0.x - 100.0).abs() < 1e-3,
        "bolt_b (spawned mid-run) should receive one tick = 100"
    );
}

#[test]
fn apply_force_despawn_and_spawn_mid_run_works_cleanly() {
    // Edge: despawn bolt_a between ticks 2 and 3, spawn bolt_c before
    // tick 3. After tick 3 — bolt_b has +200 (total +300 from 3 ticks,
    // minus the mid-run fact that bolt_b spawned at tick 2: 2 full ticks).
    // Actually re-reading the spec: bolt_a at tick 1 (start), bolt_b also
    // at start. Tick 1 → both +100. Tick 2 → both +100 (bolt_a=200, b=200).
    // Between 2 and 3: despawn a, spawn c. Tick 3 → b=300, c=100.
    // The spec says "bolt_b.velocity.0.x ≈ 200.0 AND bolt_c.velocity.0.x ≈ 100.0"
    // — that implies bolt_b spawned AFTER tick 1 (so 2 ticks total = 200).
    // Let's follow the spec narrative: spawn a at start, tick; spawn b, tick;
    // despawn a, spawn c, tick.
    let mut app = test_app_playing();
    wire_apply_force_only(&mut app);
    install_drift_config(&mut app, canonical_config());
    install_drift_wind(
        &mut app,
        DriftWind {
            direction: Vec2::X,
            timer:     8.0,
        },
    );
    add_drift_stacks(&mut app, 1);
    let bolt_a = spawn_bolt(&mut app, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs(1));

    let bolt_b = spawn_bolt(&mut app, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs(1));

    app.world_mut().despawn(bolt_a);
    let bolt_c = spawn_bolt(&mut app, Vec2::ZERO);
    tick_with_dt(&mut app, Duration::from_secs(1));

    let vel_b = app.world().get::<Velocity2D>(bolt_b).unwrap();
    let vel_c = app.world().get::<Velocity2D>(bolt_c).unwrap();
    assert!(
        (vel_b.0.x - 200.0).abs() < 1e-3,
        "bolt_b: 2 full ticks = 200, got {}",
        vel_b.0.x
    );
    assert!(
        (vel_c.0.x - 100.0).abs() < 1e-3,
        "bolt_c: 1 tick after spawn = 100, got {}",
        vel_c.0.x
    );
}
