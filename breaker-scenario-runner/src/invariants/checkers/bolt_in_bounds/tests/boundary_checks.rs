//! Tests for basic in/out of bounds checks on all four boundaries.

use bevy::prelude::*;
use breaker::shared::PlayfieldConfig;
use rantzsoft_spatial2d::components::Position2D;

use super::helpers::*;
use crate::{invariants::*, types::InvariantKind};

/// A bolt at y = 500.0 is above the top bound of a playfield with
/// height 700.0 (top = 350.0). The system must append one
/// `ViolationEntry` with `InvariantKind::BoltInBounds`, frame 1842,
/// the entity id, and a message containing the actual position and the bound.
#[test]
fn bolt_in_bounds_appends_violation_when_bolt_is_above_top_bound() {
    let mut app = test_app_bolt_in_bounds();

    // height 700.0 -> top() = 350.0
    app.world_mut().insert_resource(PlayfieldConfig {
        width: 800.0,
        height: 700.0,
        background_color_rgb: [0.0, 0.0, 0.0],
        wall_thickness: 180.0,
        zone_fraction: 0.667,
    });
    app.world_mut().insert_resource(ScenarioFrame(1842));

    let bolt_entity = app
        .world_mut()
        .spawn((ScenarioTagBolt, Position2D(Vec2::new(0.0, 500.0))))
        .id();

    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert_eq!(
        log.0.len(),
        1,
        "expected exactly one violation, got {}",
        log.0.len()
    );

    let entry = &log.0[0];
    assert_eq!(entry.invariant, InvariantKind::BoltInBounds);
    assert_eq!(entry.frame, 1842);
    assert_eq!(entry.entity, Some(bolt_entity));
    assert!(
        entry.message.contains("1842"),
        "message should contain frame '1842', got: {}",
        entry.message
    );
    assert!(
        entry.message.contains("500"),
        "message should contain bolt y '500', got: {}",
        entry.message
    );
    assert!(
        entry.message.contains("350"),
        "message should contain bound '350', got: {}",
        entry.message
    );
}

/// A bolt at y = -100.0 is within a playfield with height 700.0
/// (bottom = -350.0). No violations should be recorded.
#[test]
fn bolt_in_bounds_does_not_fire_when_bolt_is_within_bounds() {
    let mut app = test_app_bolt_in_bounds();

    app.world_mut().insert_resource(PlayfieldConfig {
        width: 800.0,
        height: 700.0,
        background_color_rgb: [0.0, 0.0, 0.0],
        wall_thickness: 180.0,
        zone_fraction: 0.667,
    });
    app.world_mut().insert_resource(ScenarioFrame(10));

    app.world_mut()
        .spawn((ScenarioTagBolt, Position2D(Vec2::new(0.0, -100.0))));

    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert!(
        log.0.is_empty(),
        "expected no violations for in-bounds bolt at y = -100.0, got: {:?}",
        log.0.iter().map(|e| &e.message).collect::<Vec<_>>()
    );
}

/// A bolt exactly at y = -350.0 (the bottom boundary of a 700.0-height
/// playfield) should NOT trigger a violation -- it is at the edge, not below.
#[test]
fn bolt_in_bounds_does_not_fire_when_bolt_is_exactly_at_bottom_bound() {
    let mut app = test_app_bolt_in_bounds();

    app.world_mut().insert_resource(PlayfieldConfig {
        width: 800.0,
        height: 700.0,
        background_color_rgb: [0.0, 0.0, 0.0],
        wall_thickness: 180.0,
        zone_fraction: 0.667,
    });
    app.world_mut().insert_resource(ScenarioFrame(0));

    // `PlayfieldConfig::bottom()` returns -350.0 for height 700.0
    app.world_mut()
        .spawn((ScenarioTagBolt, Position2D(Vec2::new(0.0, -350.0))));

    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert!(
        log.0.is_empty(),
        "expected no violation when bolt is exactly at bottom bound (-350.0)"
    );
}

/// A bolt at y = 1000.0 exceeds the top bound of a playfield with height=700.0
/// (top = 350.0). The system must append one `ViolationEntry` with
/// `InvariantKind::BoltInBounds`.
#[test]
fn bolt_in_bounds_fires_when_bolt_is_above_top_bound() {
    let mut app = test_app_bolt_in_bounds();

    // width=800.0, height=700.0 -> top() = 350.0
    app.world_mut().insert_resource(PlayfieldConfig {
        width: 800.0,
        height: 700.0,
        background_color_rgb: [0.0, 0.0, 0.0],
        wall_thickness: 180.0,
        zone_fraction: 0.667,
    });
    app.world_mut().insert_resource(ScenarioFrame(1));

    app.world_mut()
        .spawn((ScenarioTagBolt, Position2D(Vec2::new(0.0, 1000.0))));

    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert_eq!(
        log.0.len(),
        1,
        "expected exactly 1 BoltInBounds violation for bolt above top bound (y=1000.0 > top=350.0), got {}",
        log.0.len()
    );
    assert_eq!(log.0[0].invariant, InvariantKind::BoltInBounds);
}

/// A bolt exactly at y = 350.0 (the top boundary of a 700.0-height playfield)
/// must NOT trigger a violation -- the check is strict `>`.
#[test]
fn bolt_in_bounds_does_not_fire_when_bolt_is_exactly_at_top_bound() {
    let mut app = test_app_bolt_in_bounds();

    // top() = 700.0 / 2.0 = 350.0
    app.world_mut().insert_resource(PlayfieldConfig {
        width: 800.0,
        height: 700.0,
        background_color_rgb: [0.0, 0.0, 0.0],
        wall_thickness: 180.0,
        zone_fraction: 0.667,
    });
    app.world_mut().insert_resource(ScenarioFrame(1));

    app.world_mut()
        .spawn((ScenarioTagBolt, Position2D(Vec2::new(0.0, 350.0))));

    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert!(
        log.0.is_empty(),
        "expected no violation when bolt is exactly at top bound (350.0) — check is strict >"
    );
}

/// A bolt at x = -2000.0 exceeds the left bound of a playfield with
/// width=800.0 (left = -400.0). The system must append one
/// `ViolationEntry` with `InvariantKind::BoltInBounds`.
#[test]
fn bolt_in_bounds_fires_when_bolt_is_left_of_left_bound() {
    let mut app = test_app_bolt_in_bounds();

    // width=800.0 -> left() = -400.0
    app.world_mut().insert_resource(PlayfieldConfig {
        width: 800.0,
        height: 700.0,
        background_color_rgb: [0.0, 0.0, 0.0],
        wall_thickness: 180.0,
        zone_fraction: 0.667,
    });
    app.world_mut().insert_resource(ScenarioFrame(1));

    app.world_mut()
        .spawn((ScenarioTagBolt, Position2D(Vec2::new(-2000.0, 0.0))));

    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert_eq!(
        log.0.len(),
        1,
        "expected exactly 1 BoltInBounds violation for bolt left of left bound (x=-2000.0 < left=-400.0), got {}",
        log.0.len()
    );
    assert_eq!(log.0[0].invariant, InvariantKind::BoltInBounds);
}

/// A bolt at x = 2000.0 exceeds the right bound of a playfield with
/// width=800.0 (right = 400.0). The system must append one
/// `ViolationEntry` with `InvariantKind::BoltInBounds`.
#[test]
fn bolt_in_bounds_fires_when_bolt_is_right_of_right_bound() {
    let mut app = test_app_bolt_in_bounds();

    // width=800.0 -> right() = 400.0
    app.world_mut().insert_resource(PlayfieldConfig {
        width: 800.0,
        height: 700.0,
        background_color_rgb: [0.0, 0.0, 0.0],
        wall_thickness: 180.0,
        zone_fraction: 0.667,
    });
    app.world_mut().insert_resource(ScenarioFrame(1));

    app.world_mut()
        .spawn((ScenarioTagBolt, Position2D(Vec2::new(2000.0, 0.0))));

    tick(&mut app);

    let log = app.world().resource::<ViolationLog>();
    assert_eq!(
        log.0.len(),
        1,
        "expected exactly 1 BoltInBounds violation for bolt right of right bound (x=2000.0 > right=400.0), got {}",
        log.0.len()
    );
    assert_eq!(log.0[0].invariant, InvariantKind::BoltInBounds);
}
