//! Breaker and combined AABB dimension-match tests for the `check_aabb_matches_entity_dimensions` checker.

use bevy::prelude::*;
use breaker::{
    bolt::components::BoltRadius,
    breaker::components::{BreakerHeight, BreakerWidth},
    shared::EntityScale,
};
use rantzsoft_physics2d::aabb::Aabb2D;

use super::{super::checker::*, helpers::*};
use crate::{invariants::*, types::InvariantKind};

// ── Breaker Checks ──────────────────────────────────────────────

/// Behavior 9: No violation when breaker `Aabb2D` `half_extents` match width/height without `EntityScale`.
#[test]
fn no_violation_when_breaker_aabb_matches_without_scale() {
    let mut app = test_app();
    app.world_mut().spawn((
        ScenarioTagBreaker,
        Aabb2D::new(Vec2::ZERO, Vec2::new(40.0, 6.0)),
        BreakerWidth(80.0),
        BreakerHeight(12.0),
    ));
    tick(&mut app);
    let log = app.world().resource::<ViolationLog>();
    assert!(
        log.0.is_empty(),
        "no violation expected when breaker half_extents (40.0, 6.0) match half_width/half_height with no EntityScale"
    );
}

/// Behavior 10: No violation when breaker has `EntityScale` — stored `Aabb2D` is unscaled.
#[test]
fn no_violation_when_breaker_aabb_matches_with_scale() {
    let mut app = test_app();
    // Aabb2D stores unscaled base dimensions even when EntityScale is present.
    // The checker ignores EntityScale because collision systems apply scale at runtime.
    app.world_mut().spawn((
        ScenarioTagBreaker,
        Aabb2D::new(Vec2::ZERO, Vec2::new(40.0, 6.0)),
        BreakerWidth(80.0),
        BreakerHeight(12.0),
        EntityScale(2.0),
    ));
    tick(&mut app);
    let log = app.world().resource::<ViolationLog>();
    assert!(
        log.0.is_empty(),
        "no violation expected — Aabb2D stores unscaled (40.0, 6.0) regardless of EntityScale"
    );
}

/// Behavior 11: Violation fires when breaker `Aabb2D` `half_extents` do not match expected (no scale).
#[test]
fn violation_when_breaker_aabb_does_not_match_without_scale() {
    let mut app = test_app();
    let breaker = app
        .world_mut()
        .spawn((
            ScenarioTagBreaker,
            Aabb2D::new(Vec2::ZERO, Vec2::new(50.0, 6.0)),
            BreakerWidth(80.0),
            BreakerHeight(12.0),
        ))
        .id();
    tick(&mut app);
    let log = app.world().resource::<ViolationLog>();
    assert_eq!(
        log.0.len(),
        1,
        "expected 1 violation when breaker x half_extent (50.0) != expected (40.0)"
    );
    assert_eq!(
        log.0[0].invariant,
        InvariantKind::AabbMatchesEntityDimensions
    );
    assert_eq!(
        log.0[0].entity,
        Some(breaker),
        "violation should reference the breaker entity"
    );
}

/// Behavior 12: Violation fires when breaker `Aabb2D` doesn't match unscaled dimensions (`EntityScale` present but irrelevant).
#[test]
fn violation_when_breaker_aabb_does_not_match_with_scale() {
    let mut app = test_app();
    // EntityScale is present but the checker ignores it — expected is still (40.0, 6.0).
    // Aabb2D (80.0, 12.0) mismatches the unscaled expected (40.0, 6.0).
    app.world_mut().spawn((
        ScenarioTagBreaker,
        Aabb2D::new(Vec2::ZERO, Vec2::new(80.0, 12.0)),
        BreakerWidth(80.0),
        BreakerHeight(12.0),
        EntityScale(2.0),
    ));
    tick(&mut app);
    let log = app.world().resource::<ViolationLog>();
    assert_eq!(
        log.0.len(),
        1,
        "expected 1 violation when breaker half_extents (80.0, 12.0) != expected unscaled (40.0, 6.0)"
    );
    assert_eq!(
        log.0[0].invariant,
        InvariantKind::AabbMatchesEntityDimensions
    );
}

/// Behavior 13: No violation when breaker `half_extents` are within epsilon of expected.
#[test]
fn no_violation_when_breaker_aabb_within_epsilon() {
    let mut app = test_app();
    app.world_mut().spawn((
        ScenarioTagBreaker,
        Aabb2D::new(Vec2::ZERO, Vec2::new(40.0005, 6.0005)),
        BreakerWidth(80.0),
        BreakerHeight(12.0),
    ));
    tick(&mut app);
    let log = app.world().resource::<ViolationLog>();
    assert!(
        log.0.is_empty(),
        "no violation expected when breaker delta (0.0005) < epsilon (0.001) on both axes"
    );
}

/// Behavior 14: Breaker with `EntityScale(1.0)` behaves identically to no `EntityScale`.
#[test]
fn no_violation_when_breaker_has_explicit_scale_one() {
    let mut app = test_app();
    app.world_mut().spawn((
        ScenarioTagBreaker,
        Aabb2D::new(Vec2::ZERO, Vec2::new(40.0, 6.0)),
        BreakerWidth(80.0),
        BreakerHeight(12.0),
        EntityScale(1.0),
    ));
    tick(&mut app);
    let log = app.world().resource::<ViolationLog>();
    assert!(
        log.0.is_empty(),
        "no violation expected when EntityScale(1.0) is equivalent to absent scale"
    );
}

/// Behavior 15: No violation when no breaker entities exist.
#[test]
fn no_violation_when_no_breakers_exist() {
    let mut app = test_app();
    tick(&mut app);
    let log = app.world().resource::<ViolationLog>();
    assert!(
        log.0.is_empty(),
        "no violation expected when no breaker entities exist"
    );
}

// ── Combined Checks ─────────────────────────────────────────────

/// Behavior 16: Both bolt and breaker checked in single system run.
#[test]
fn bolt_and_breaker_checked_only_incorrect_flagged() {
    let mut app = test_app();
    // Bolt: correct
    app.world_mut().spawn((
        ScenarioTagBolt,
        Aabb2D::new(Vec2::ZERO, Vec2::new(8.0, 8.0)),
        BoltRadius(8.0),
    ));
    // Breaker: incorrect (expected half_width 40.0, got 50.0)
    let breaker = app
        .world_mut()
        .spawn((
            ScenarioTagBreaker,
            Aabb2D::new(Vec2::ZERO, Vec2::new(50.0, 6.0)),
            BreakerWidth(80.0),
            BreakerHeight(12.0),
        ))
        .id();
    tick(&mut app);
    let log = app.world().resource::<ViolationLog>();
    assert_eq!(
        log.0.len(),
        1,
        "expected exactly 1 violation (breaker only, bolt is correct)"
    );
    assert_eq!(
        log.0[0].entity,
        Some(breaker),
        "violation should reference the breaker entity"
    );
    assert_eq!(
        log.0[0].invariant,
        InvariantKind::AabbMatchesEntityDimensions
    );
}

/// Behavior 17: Violation entry contains correct invariant kind, entity, and diagnostic message.
#[test]
fn violation_entry_has_correct_fields_and_message() {
    let mut app = test_app();
    let bolt = app
        .world_mut()
        .spawn((
            ScenarioTagBolt,
            Aabb2D::new(Vec2::ZERO, Vec2::new(12.0, 12.0)),
            BoltRadius(8.0),
        ))
        .id();
    tick(&mut app);
    let log = app.world().resource::<ViolationLog>();
    assert_eq!(log.0.len(), 1, "expected exactly 1 violation");
    let entry = &log.0[0];
    assert_eq!(
        entry.invariant,
        InvariantKind::AabbMatchesEntityDimensions,
        "invariant kind should be AabbMatchesEntityDimensions"
    );
    assert_eq!(
        entry.entity,
        Some(bolt),
        "violation should reference the bolt entity"
    );
    assert!(
        entry.message.contains("expected="),
        "violation message should include 'expected=', got: {}",
        entry.message
    );
    assert!(
        entry.message.contains("actual="),
        "violation message should include 'actual=', got: {}",
        entry.message
    );
}

/// Behavior 19: At-epsilon-boundary breaker `Aabb2D` does not fire violation (strict greater-than).
#[test]
fn no_violation_when_breaker_delta_exactly_equals_epsilon() {
    let mut app = test_app();
    // Use zero dimensions so delta is exactly AABB_EPSILON with no fp
    // cancellation: (eps - 0.0) = eps, and eps > eps is false.
    app.world_mut().spawn((
        ScenarioTagBreaker,
        Aabb2D::new(Vec2::ZERO, Vec2::splat(AABB_EPSILON)),
        BreakerWidth(0.0),
        BreakerHeight(0.0),
    ));
    tick(&mut app);
    let log = app.world().resource::<ViolationLog>();
    assert!(
        log.0.is_empty(),
        "no violation expected when breaker delta (0.001) == epsilon (0.001) -- strict greater-than"
    );
}

/// Behavior 20: Both bolt and breaker violations fire when both are mismatched simultaneously.
#[test]
fn violations_fire_for_both_bolt_and_breaker_when_both_mismatched() {
    let mut app = test_app();
    // Bolt: mismatched (12.0 vs expected 8.0)
    let bolt = app
        .world_mut()
        .spawn((
            ScenarioTagBolt,
            Aabb2D::new(Vec2::ZERO, Vec2::new(12.0, 12.0)),
            BoltRadius(8.0),
        ))
        .id();
    // Breaker: mismatched (expected half_extents (40.0, 6.0), actual (50.0, 10.0))
    let breaker = app
        .world_mut()
        .spawn((
            ScenarioTagBreaker,
            Aabb2D::new(Vec2::ZERO, Vec2::new(50.0, 10.0)),
            BreakerWidth(80.0),
            BreakerHeight(12.0),
        ))
        .id();
    tick(&mut app);
    let log = app.world().resource::<ViolationLog>();
    assert_eq!(
        log.0.len(),
        2,
        "expected 2 violations -- one for bolt, one for breaker"
    );

    // Both should be AabbMatchesEntityDimensions
    assert!(
        log.0
            .iter()
            .all(|v| v.invariant == InvariantKind::AabbMatchesEntityDimensions),
        "all violations should be AabbMatchesEntityDimensions"
    );

    // One should be for bolt, one for breaker
    let bolt_violation_count = log.0.iter().filter(|v| v.entity == Some(bolt)).count();
    let breaker_violation_count = log.0.iter().filter(|v| v.entity == Some(breaker)).count();
    assert_eq!(
        bolt_violation_count, 1,
        "expected exactly 1 violation for the bolt entity"
    );
    assert_eq!(
        breaker_violation_count, 1,
        "expected exactly 1 violation for the breaker entity"
    );
}
