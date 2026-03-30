//! Tests for `BoltRadius`-aware margin checks and the open bottom boundary.

use bevy::prelude::*;
use breaker::bolt::components::BoltRadius;
use rantzsoft_spatial2d::components::Position2D;

use super::helpers::*;
use crate::{invariants::*, types::InvariantKind};

/// Playfield height=700.0 -> bottom=-350.0. Bolt at y=-358.0 with `BoltRadius(8.0)`.
/// The allowed margin is `bottom - (radius + 1.0)` = -350.0 - 9.0 = -359.0.
/// At -358.0 the bolt center is within the radius margin -- no violation.
#[test]
fn bolt_in_bounds_no_violation_when_bolt_slightly_below_bottom_within_radius_margin() {
    let mut app = test_app_bolt_in_bounds_with_radius();

    app.world_mut().spawn((
        ScenarioTagBolt,
        Position2D(Vec2::new(0.0, -358.0)),
        BoltRadius(8.0),
    ));

    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert!(
        !log.0
            .iter()
            .any(|v| v.invariant == InvariantKind::BoltInBounds),
        "expected no BoltInBounds violation for bolt at y=-358.0 with BoltRadius(8.0) \
        (bottom=-350.0, margin=-359.0 — bolt is within margin), got: {:?}",
        log.0
            .iter()
            .filter(|v| v.invariant == InvariantKind::BoltInBounds)
            .map(|e| &e.message)
            .collect::<Vec<_>>()
    );
}

/// Bolt at y=500.0 with `BoltRadius(8.0)`. The allowed margin is top + 9.0 = 359.0.
/// 500.0 is well beyond 359.0 -- violation fires.
#[test]
fn bolt_in_bounds_fires_when_bolt_far_above_top_beyond_radius_margin() {
    let mut app = test_app_bolt_in_bounds_with_radius();

    // top() = 350.0, margin = 8.0 + 1.0 = 9.0; allowed = 359.0; 500.0 well beyond
    app.world_mut().spawn((
        ScenarioTagBolt,
        Position2D(Vec2::new(0.0, 500.0)),
        BoltRadius(8.0),
    ));

    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert_eq!(
        log.0
            .iter()
            .filter(|v| v.invariant == InvariantKind::BoltInBounds)
            .count(),
        1,
        "expected exactly 1 BoltInBounds violation for bolt at y=500.0 with BoltRadius(8.0) \
        (far beyond margin of 359.0), got: {:?}",
        log.0
            .iter()
            .filter(|v| v.invariant == InvariantKind::BoltInBounds)
            .map(|e| &e.message)
            .collect::<Vec<_>>()
    );
    assert_eq!(log.0[0].invariant, InvariantKind::BoltInBounds);
}

/// Playfield width=800.0 -> right=400.0. Bolt at x=408.0 with `BoltRadius(8.0)`.
/// The allowed margin is `right + (radius + 1.0)` = 400.0 + 9.0 = 409.0.
/// At 408.0 the bolt center is within the radius margin -- no violation.
#[test]
fn bolt_in_bounds_no_violation_when_bolt_slightly_past_right_wall_within_radius_margin() {
    let mut app = test_app_bolt_in_bounds_with_radius();

    app.world_mut().spawn((
        ScenarioTagBolt,
        Position2D(Vec2::new(408.0, 0.0)),
        BoltRadius(8.0),
    ));

    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert!(
        !log.0
            .iter()
            .any(|v| v.invariant == InvariantKind::BoltInBounds),
        "expected no BoltInBounds violation for bolt at x=408.0 with BoltRadius(8.0) \
        (right=400.0, margin=409.0 — bolt is within margin), got: {:?}",
        log.0
            .iter()
            .filter(|v| v.invariant == InvariantKind::BoltInBounds)
            .map(|e| &e.message)
            .collect::<Vec<_>>()
    );
}

/// Bolt at y=-350.0 (exactly the bottom boundary) with `BoltRadius(8.0)`.
/// The bolt center is exactly at the boundary -- well within the radius margin
/// of -359.0. No violation must fire.
#[test]
fn bolt_in_bounds_no_violation_when_bolt_center_at_exact_boundary_with_radius() {
    let mut app = test_app_bolt_in_bounds_with_radius();

    app.world_mut().spawn((
        ScenarioTagBolt,
        Position2D(Vec2::new(0.0, -350.0)),
        BoltRadius(8.0),
    ));

    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert!(
        !log.0
            .iter()
            .any(|v| v.invariant == InvariantKind::BoltInBounds),
        "expected no BoltInBounds violation when bolt center is exactly at bottom \
        boundary (-350.0) with BoltRadius(8.0) — center is within the radius margin",
    );
}

/// Bolt exits through bottom during life-loss. The bottom boundary is
/// intentionally open (no floor wall), so `check_bolt_in_bounds` should not
/// check the bottom at all. A bolt at y=-1000.0 (far below) should not fire.
#[test]
fn bolt_in_bounds_does_not_fire_when_bolt_exits_through_open_bottom() {
    let mut app = test_app_bolt_in_bounds_with_radius();

    // Bolt far below bottom — simulates life-loss exit through open floor
    app.world_mut().spawn((
        ScenarioTagBolt,
        Position2D(Vec2::new(0.0, -1000.0)),
        BoltRadius(14.0),
    ));

    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert!(
        !log.0
            .iter()
            .any(|v| v.invariant == InvariantKind::BoltInBounds),
        "expected no BoltInBounds violation for bolt exiting through open bottom \
        (no floor wall by design), got: {:?}",
        log.0
            .iter()
            .filter(|v| v.invariant == InvariantKind::BoltInBounds)
            .map(|e| &e.message)
            .collect::<Vec<_>>()
    );
}
