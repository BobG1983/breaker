//! Behavioral tests for the `apply_bolt_forces` consumer system.
//!
//! All tests use the resource-driven helper-writer pattern: a `PendingForces`
//! resource is populated before each `tick()` call; an `enqueue_forces` helper
//! system drains it into the `MessageWriter<ApplyBoltForce>` before
//! `apply_bolt_forces` runs.

use bevy::prelude::*;

use super::system::apply_bolt_forces;
use crate::{
    bolt::{messages::ApplyBoltForce, test_utils::spawn_bolt},
    prelude::*,
};

/// Fixed-timestep in seconds used by `TestAppBuilder::new()` (1/64 s).
const DT: f32 = 1.0 / 64.0;
/// Absolute tolerance for f32 comparisons that involve `DT`.
const TOLERANCE: f32 = 1e-4;

fn close_to(actual: f32, expected: f32) -> bool {
    (actual - expected).abs() < TOLERANCE
}

// ── Test harness ─────────────────────────────────────────────────────────────

/// Helper resource: tests push the `ApplyBoltForce` messages they want written
/// this tick. `enqueue_forces` drains it into the writer before
/// `apply_bolt_forces` runs.
#[derive(Resource, Default)]
struct PendingForces(Vec<ApplyBoltForce>);

/// Helper test system. Runs in `FixedUpdate`, ordered `before apply_bolt_forces`.
fn enqueue_forces(mut pending: ResMut<PendingForces>, mut writer: MessageWriter<ApplyBoltForce>) {
    for msg in pending.0.drain(..) {
        writer.write(msg);
    }
}

fn test_app() -> App {
    TestAppBuilder::new()
        .with_message::<ApplyBoltForce>()
        .with_resource::<PendingForces>()
        .with_system(
            FixedUpdate,
            (enqueue_forces.before(apply_bolt_forces), apply_bolt_forces),
        )
        .build()
}

// ── Behavior 1: single force converts to velocity delta ───────────────────────

/// A single `ApplyBoltForce { force: (100.0, 0.0) }` must produce a velocity
/// delta of `100.0 * dt` on the x axis. This pins the acceleration semantics:
/// if the implementation forgot `* dt`, it would write `100.0` instead of
/// `1.5625` and this test would catch it.
#[test]
fn single_force_converts_to_velocity_delta() {
    let mut app = test_app();
    let bolt = spawn_bolt(&mut app, 0.0, 0.0, 0.0, 0.0);

    app.insert_resource(PendingForces(vec![ApplyBoltForce {
        bolt,
        force: Vec2::new(100.0, 0.0),
    }]));
    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        close_to(vel.0.x, 100.0 * DT),
        "expected vel.x ≈ {}, got {}",
        100.0 * DT,
        vel.0.x
    );
    assert!(
        close_to(vel.0.y, 0.0),
        "expected vel.y == 0.0, got {}",
        vel.0.y
    );
}

// ── Behavior 2: multiple forces are aggregated before the dt multiply ─────────

/// Three forces on the same bolt must produce a velocity delta equal to the
/// vector sum of all forces multiplied by `dt` once. x: (50+50)*dt ≈ 1.5625.
/// y: 100*dt ≈ 1.5625. For equal-weight forces, sum-then-multiply and
/// multiply-then-sum are numerically identical — this test pins the result,
/// not the internal ordering.
#[test]
fn multiple_forces_aggregate_before_applying() {
    let mut app = test_app();
    let bolt = spawn_bolt(&mut app, 0.0, 0.0, 0.0, 0.0);

    app.insert_resource(PendingForces(vec![
        ApplyBoltForce {
            bolt,
            force: Vec2::new(50.0, 0.0),
        },
        ApplyBoltForce {
            bolt,
            force: Vec2::new(50.0, 0.0),
        },
        ApplyBoltForce {
            bolt,
            force: Vec2::new(0.0, 100.0),
        },
    ]));
    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        close_to(vel.0.x, 100.0 * DT),
        "expected vel.x ≈ {} (sum of x forces * dt), got {}",
        100.0 * DT,
        vel.0.x
    );
    assert!(
        close_to(vel.0.y, 100.0 * DT),
        "expected vel.y ≈ {} (y force * dt), got {}",
        100.0 * DT,
        vel.0.y
    );
}

// ── Behavior 3: missing bolt entity does not panic ────────────────────────────

/// A force addressed to a despawned entity must be silently discarded without
/// panicking. The "witness" bolt must remain untouched — force addressed to a
/// missing entity must not spill onto other bolts.
#[test]
fn missing_bolt_entity_no_panic() {
    let mut app = test_app();

    // Spawn and immediately despawn to get a stale entity id.
    let ghost = spawn_bolt(&mut app, 0.0, 0.0, 0.0, 0.0);
    app.world_mut().despawn(ghost);

    // Witness bolt with a recognisable velocity we can check for contamination.
    let witness = spawn_bolt(&mut app, 0.0, 0.0, 7.0, 11.0);

    app.insert_resource(PendingForces(vec![ApplyBoltForce {
        bolt:  ghost,
        force: Vec2::new(500.0, 500.0),
    }]));

    // Must not panic.
    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(witness).unwrap();
    assert!(
        close_to(vel.0.x, 7.0),
        "witness vel.x must be unchanged (7.0), got {}",
        vel.0.x
    );
    assert!(
        close_to(vel.0.y, 11.0),
        "witness vel.y must be unchanged (11.0), got {}",
        vel.0.y
    );
}

// ── Behavior 4: multiple bolts receive independent forces ─────────────────────

/// Forces for different bolts must not bleed between entities. Bolt A gets
/// `(100, 0)` → x delta `100/64`; Bolt B gets `(0, 200)` → y delta `200/64`.
/// If the system aggregates into a single Vec2 instead of a per-entity map,
/// both bolts would see `(100/64, 200/64)` and the test would fail.
#[test]
fn multiple_bolts_receive_independent_forces() {
    let mut app = test_app();
    let bolt_a = spawn_bolt(&mut app, -100.0, 0.0, 0.0, 0.0);
    let bolt_b = spawn_bolt(&mut app, 100.0, 0.0, 0.0, 0.0);

    app.insert_resource(PendingForces(vec![
        ApplyBoltForce {
            bolt:  bolt_a,
            force: Vec2::new(100.0, 0.0),
        },
        ApplyBoltForce {
            bolt:  bolt_b,
            force: Vec2::new(0.0, 200.0),
        },
    ]));
    tick(&mut app);

    let vel_a = app.world().get::<Velocity2D>(bolt_a).unwrap();
    assert!(
        close_to(vel_a.0.x, 100.0 * DT),
        "bolt A vel.x expected ≈ {}, got {}",
        100.0 * DT,
        vel_a.0.x
    );
    assert!(
        close_to(vel_a.0.y, 0.0),
        "bolt A vel.y must be 0.0, got {}",
        vel_a.0.y
    );

    let vel_b = app.world().get::<Velocity2D>(bolt_b).unwrap();
    assert!(
        close_to(vel_b.0.x, 0.0),
        "bolt B vel.x must be 0.0, got {}",
        vel_b.0.x
    );
    assert!(
        close_to(vel_b.0.y, 200.0 * DT),
        "bolt B vel.y expected ≈ {}, got {}",
        200.0 * DT,
        vel_b.0.y
    );
}

// ── Behavior 5: zero force message does not alter pre-existing velocity ───────

/// A force of `Vec2::ZERO` is valid input. The bolt must retain its
/// pre-existing velocity exactly. This catches implementations that
/// **assign** `force * dt` to velocity instead of **adding** it — an assign
/// would produce `(0.0, 0.0)` and fail.
#[test]
fn zero_force_message_no_velocity_change() {
    let mut app = test_app();
    let bolt = spawn_bolt(&mut app, 0.0, 0.0, 50.0, 30.0);

    app.insert_resource(PendingForces(vec![ApplyBoltForce {
        bolt,
        force: Vec2::ZERO,
    }]));
    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        close_to(vel.0.x, 50.0),
        "vel.x must remain exactly 50.0 after zero force, got {}",
        vel.0.x
    );
    assert!(
        close_to(vel.0.y, 30.0),
        "vel.y must remain exactly 30.0 after zero force, got {}",
        vel.0.y
    );
}

// ── Behavior 6: empty message queue does not alter velocity ───────────────────

/// When no `ApplyBoltForce` messages are written this tick, the bolt's
/// pre-existing velocity must be completely unchanged. This catches
/// implementations that unconditionally overwrite or reset velocity outside
/// the message drain loop.
#[test]
fn empty_message_queue_no_velocity_change() {
    let mut app = test_app();
    let bolt = spawn_bolt(&mut app, 0.0, 0.0, 50.0, 30.0);

    // PendingForces is default-initialized (empty) — no messages written.
    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        close_to(vel.0.x, 50.0),
        "vel.x must remain exactly 50.0 with empty queue, got {}",
        vel.0.x
    );
    assert!(
        close_to(vel.0.y, 30.0),
        "vel.y must remain exactly 30.0 with empty queue, got {}",
        vel.0.y
    );
}

// ── Behavior 7: single-message first-path is additive, not assign ─────────────

/// A single queued force must be ADDED to the bolt's existing velocity, not
/// assigned. Starting from (50.0, 30.0) ensures the += vs = distinction is
/// observable: an assign implementation would produce (100/64, 30.0) instead
/// of (50 + 100/64, 30.0).
#[test]
fn single_message_additive_on_nonzero_velocity() {
    let mut app = test_app();
    let bolt = spawn_bolt(&mut app, 0.0, 0.0, 50.0, 30.0);

    app.insert_resource(PendingForces(vec![ApplyBoltForce {
        bolt,
        force: Vec2::new(100.0, 0.0),
    }]));
    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        close_to(vel.0.x, 100.0f32.mul_add(DT, 50.0)),
        "expected vel.x ≈ {} (50.0 + 100.0*dt), got {}",
        100.0f32.mul_add(DT, 50.0),
        vel.0.x
    );
    assert!(
        close_to(vel.0.y, 30.0),
        "expected vel.y == 30.0 (unchanged), got {}",
        vel.0.y
    );
}

// ── Behavior 8: forces from multiple emitters accumulate on a single bolt ─────

/// Forces from two independent emitters (e.g., drift hazard + gravity surge)
/// for the same bolt in the same tick must both be applied. This is the
/// primary use-case for `ApplyBoltForce`: decoupled producers composing
/// without coordination.
#[test]
fn cross_emitter_forces_accumulate() {
    let mut app = test_app();
    let bolt = spawn_bolt(&mut app, 0.0, 0.0, 0.0, 0.0);

    app.insert_resource(PendingForces(vec![
        ApplyBoltForce {
            bolt,
            force: Vec2::new(100.0, 0.0), // simulated drift force
        },
        ApplyBoltForce {
            bolt,
            force: Vec2::new(0.0, 200.0), // simulated gravity surge force
        },
    ]));
    tick(&mut app);

    let vel = app.world().get::<Velocity2D>(bolt).unwrap();
    assert!(
        close_to(vel.0.x, 100.0 * DT),
        "expected vel.x ≈ {} (drift force * dt), got {}",
        100.0 * DT,
        vel.0.x
    );
    assert!(
        close_to(vel.0.y, 200.0 * DT),
        "expected vel.y ≈ {} (gravity surge force * dt), got {}",
        200.0 * DT,
        vel.0.y
    );
}
