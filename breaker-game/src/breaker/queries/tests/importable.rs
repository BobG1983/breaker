use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;

use super::{super::data::*, helpers::test_app};
use crate::breaker::components::{
    Breaker, BreakerTilt, BumpEarlyWindow, BumpLateWindow, BumpPerfectWindow, BumpState, DashState,
};

// ── Part I: BreakerTelemetryData (read-only, dev-only) ──────────

// Behavior 18: BreakerTelemetryData compiles and queries under dev feature
#[cfg(feature = "dev")]
#[test]
fn breaker_telemetry_data_query_under_dev_feature() {
    let mut app = test_app();
    app.world_mut().spawn((
        Breaker,
        DashState::Idle,
        BumpState::default(),
        BreakerTilt::default(),
        Velocity2D(Vec2::new(100.0, 0.0)),
        BumpPerfectWindow(0.05),
        BumpEarlyWindow(0.15),
        BumpLateWindow(0.1),
    ));

    let mut query = app
        .world_mut()
        .query_filtered::<BreakerTelemetryData, With<Breaker>>();
    let data = query.single(app.world()).unwrap();
    assert_eq!(*data.state, DashState::Idle);
    assert!(!data.bump.active);
    assert!((data.tilt.angle - 0.0).abs() < f32::EPSILON);
    assert_eq!(data.velocity.0, Vec2::new(100.0, 0.0));
    assert!((data.perfect_window.0 - 0.05).abs() < f32::EPSILON);
    assert!((data.early_window.0 - 0.15).abs() < f32::EPSILON);
    assert!((data.late_window.0 - 0.1).abs() < f32::EPSILON);
}

// ── Part J: Cross-struct consistency ─────────────────────────────

// Behavior 19: All 9 QueryData structs importable and usable in queries
#[test]
fn all_querydata_structs_importable_and_queryable() {
    let mut app = test_app();

    // Read-only structs — queried directly (no ReadOnly variant)
    drop(
        app.world_mut()
            .query_filtered::<BreakerCollisionData, With<Breaker>>(),
    );
    drop(
        app.world_mut()
            .query_filtered::<BreakerSizeData, With<Breaker>>(),
    );

    // Mutable structs — ReadOnly variant also exists
    drop(
        app.world_mut()
            .query_filtered::<BreakerMovementDataReadOnly, With<Breaker>>(),
    );
    drop(
        app.world_mut()
            .query_filtered::<BreakerDashDataReadOnly, With<Breaker>>(),
    );
    drop(
        app.world_mut()
            .query_filtered::<BreakerResetDataReadOnly, With<Breaker>>(),
    );
    drop(
        app.world_mut()
            .query_filtered::<BreakerBumpTimingDataReadOnly, With<Breaker>>(),
    );
    drop(
        app.world_mut()
            .query_filtered::<BreakerBumpGradingDataReadOnly, With<Breaker>>(),
    );
    drop(
        app.world_mut()
            .query_filtered::<SyncBreakerScaleDataReadOnly, With<Breaker>>(),
    );
}

#[cfg(feature = "dev")]
#[test]
fn telemetry_data_importable_under_dev_feature() {
    let mut app = test_app();
    drop(
        app.world_mut()
            .query_filtered::<BreakerTelemetryData, With<Breaker>>(),
    );
}
