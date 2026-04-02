//! Bolt AABB dimension-match tests for the `check_aabb_matches_entity_dimensions` checker.

use bevy::prelude::*;
use breaker::shared::size::BaseRadius;
use rantzsoft_physics2d::aabb::Aabb2D;

use super::{super::checker::*, helpers::*};
use crate::{invariants::*, types::InvariantKind};

// ── Bolt Checks ─────────────────────────────────────────────────

/// Behavior 1: No violation when bolt `Aabb2D` `half_extents` match `BoltRadius`.
#[test]
fn no_violation_when_bolt_aabb_matches_radius() {
    let mut app = test_app();
    app.world_mut().spawn((
        ScenarioTagBolt,
        Aabb2D::new(Vec2::ZERO, Vec2::new(8.0, 8.0)),
        BaseRadius(8.0),
    ));
    tick(&mut app);
    let log = app.world().resource::<ViolationLog>();
    assert!(
        log.0.is_empty(),
        "no violation expected when bolt Aabb2D half_extents match BoltRadius"
    );
}

/// Behavior 2: Violation fires when bolt `Aabb2D` `half_extents` do not match `BoltRadius`.
#[test]
fn violation_when_bolt_aabb_does_not_match_radius() {
    let mut app = test_app();
    let bolt = app
        .world_mut()
        .spawn((
            ScenarioTagBolt,
            Aabb2D::new(Vec2::ZERO, Vec2::new(12.0, 12.0)),
            BaseRadius(8.0),
        ))
        .id();
    tick(&mut app);
    let log = app.world().resource::<ViolationLog>();
    assert_eq!(
        log.0.len(),
        1,
        "expected 1 violation when bolt half_extents (12.0) != BoltRadius (8.0)"
    );
    assert_eq!(
        log.0[0].invariant,
        InvariantKind::AabbMatchesEntityDimensions
    );
    assert_eq!(
        log.0[0].entity,
        Some(bolt),
        "violation should reference the bolt entity"
    );
}

/// Behavior 3: No violation when bolt `half_extents` are within epsilon of `BoltRadius`.
#[test]
fn no_violation_when_bolt_aabb_within_epsilon() {
    let mut app = test_app();
    app.world_mut().spawn((
        ScenarioTagBolt,
        Aabb2D::new(Vec2::ZERO, Vec2::new(8.0005, 8.0005)),
        BaseRadius(8.0),
    ));
    tick(&mut app);
    let log = app.world().resource::<ViolationLog>();
    assert!(
        log.0.is_empty(),
        "no violation expected when delta (0.0005) < epsilon (0.001)"
    );
}

/// Behavior 4: Violation fires when bolt `half_extents` differ by more than epsilon.
#[test]
fn violation_when_bolt_aabb_exceeds_epsilon() {
    let mut app = test_app();
    app.world_mut().spawn((
        ScenarioTagBolt,
        Aabb2D::new(Vec2::ZERO, Vec2::new(8.002, 8.002)),
        BaseRadius(8.0),
    ));
    tick(&mut app);
    let log = app.world().resource::<ViolationLog>();
    assert_eq!(
        log.0.len(),
        1,
        "expected 1 violation when bolt delta (0.002) > epsilon (0.001)"
    );
    assert_eq!(
        log.0[0].invariant,
        InvariantKind::AabbMatchesEntityDimensions
    );
}

/// Behavior 5: Violation fires when only one axis of bolt `half_extents` mismatches.
#[test]
fn violation_when_bolt_aabb_single_axis_mismatch() {
    let mut app = test_app();
    app.world_mut().spawn((
        ScenarioTagBolt,
        Aabb2D::new(Vec2::ZERO, Vec2::new(8.0, 12.0)),
        BaseRadius(8.0),
    ));
    tick(&mut app);
    let log = app.world().resource::<ViolationLog>();
    assert_eq!(
        log.0.len(),
        1,
        "expected 1 violation when only y-axis mismatches (12.0 vs 8.0)"
    );
    assert_eq!(
        log.0[0].invariant,
        InvariantKind::AabbMatchesEntityDimensions
    );
}

/// Behavior 6: No violation when no bolt entities exist.
#[test]
fn no_violation_when_no_bolts_exist() {
    let mut app = test_app();
    tick(&mut app);
    let log = app.world().resource::<ViolationLog>();
    assert!(
        log.0.is_empty(),
        "no violation expected when no bolt entities exist"
    );
}

/// Behavior 7: Violation fires per bolt entity independently.
#[test]
fn violation_fires_per_bolt_independently() {
    let mut app = test_app();
    // Entity A: correct
    app.world_mut().spawn((
        ScenarioTagBolt,
        Aabb2D::new(Vec2::ZERO, Vec2::new(8.0, 8.0)),
        BaseRadius(8.0),
    ));
    // Entity B: incorrect
    let entity_b = app
        .world_mut()
        .spawn((
            ScenarioTagBolt,
            Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
            BaseRadius(8.0),
        ))
        .id();
    tick(&mut app);
    let log = app.world().resource::<ViolationLog>();
    assert_eq!(
        log.0.len(),
        1,
        "expected exactly 1 violation for the incorrect bolt only"
    );
    assert_eq!(
        log.0[0].entity,
        Some(entity_b),
        "violation should reference entity B (the incorrect bolt)"
    );
}

/// Behavior 8: Bolt with non-default `BoltRadius` is checked correctly.
#[test]
fn no_violation_when_bolt_has_non_default_radius() {
    let mut app = test_app();
    app.world_mut().spawn((
        ScenarioTagBolt,
        Aabb2D::new(Vec2::ZERO, Vec2::new(6.0, 6.0)),
        BaseRadius(6.0),
    ));
    tick(&mut app);
    let log = app.world().resource::<ViolationLog>();
    assert!(
        log.0.is_empty(),
        "no violation expected when BaseRadius(6.0) matches half_extents (6.0, 6.0)"
    );
}

/// Behavior 18: At-epsilon-boundary bolt `Aabb2D` does not fire violation (strict greater-than).
#[test]
fn no_violation_when_bolt_delta_exactly_equals_epsilon() {
    let mut app = test_app();
    // Use radius 0.0 so the delta is exactly AABB_EPSILON with no fp
    // cancellation: (eps - 0.0) = eps, and eps > eps is false.
    app.world_mut().spawn((
        ScenarioTagBolt,
        Aabb2D::new(Vec2::ZERO, Vec2::splat(AABB_EPSILON)),
        BaseRadius(0.0),
    ));
    tick(&mut app);
    let log = app.world().resource::<ViolationLog>();
    assert!(
        log.0.is_empty(),
        "no violation expected when delta (0.001) == epsilon (0.001) -- strict greater-than comparison"
    );
}
