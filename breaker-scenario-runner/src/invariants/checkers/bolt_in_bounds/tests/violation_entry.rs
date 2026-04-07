//! Tests for violation entry fields and `ScenarioStats` increment.

use bevy::prelude::*;
use breaker::shared::PlayfieldConfig;
use rantzsoft_spatial2d::components::Position2D;

use super::helpers::*;
use crate::{invariants::*, types::InvariantKind};

/// When `ScenarioStats` is present, `check_bolt_in_bounds` increments
/// `invariant_checks` by the number of tagged bolt entities checked.
#[test]
fn bolt_in_bounds_increments_invariant_checks_in_scenario_stats() {
    let mut app = test_app_bolt_in_bounds();

    app.world_mut().insert_resource(PlayfieldConfig {
        width: 800.0,
        height: 700.0,
        background_color_rgb: [0.0, 0.0, 0.0],
        wall_thickness: 180.0,
        zone_fraction: 0.667,
    });
    app.world_mut().insert_resource(ScenarioStats {
        entered_playing: true,
        ..Default::default()
    });

    // Spawn 3 tagged bolts, all in-bounds
    for _ in 0..3 {
        app.world_mut()
            .spawn((ScenarioTagBolt, Position2D(Vec2::new(0.0, 0.0))));
    }

    tick(&mut app);

    let stats = app.world().resource::<ScenarioStats>();
    assert_eq!(
        stats.invariant_checks, 1,
        "expected invariant_checks=1 (one checker invocation per frame), got {}",
        stats.invariant_checks
    );
}

/// Verifies that the entry from the out-of-bounds case has all required
/// fields set: frame, invariant, entity (`Some`), and a message with values.
/// Uses the top boundary (bottom is intentionally open).
#[test]
fn violation_entry_contains_frame_invariant_entity_and_message_with_values() {
    let mut app = test_app_bolt_in_bounds();

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
    let entry = &log.0[0];

    assert_eq!(entry.frame, 1842, "frame must be 1842");
    assert_eq!(
        entry.invariant,
        InvariantKind::BoltInBounds,
        "invariant must be BoltInBounds"
    );
    assert_eq!(
        entry.entity,
        Some(bolt_entity),
        "entity must be Some(bolt_entity)"
    );
    assert!(!entry.message.is_empty(), "message must not be empty");
    assert!(
        entry.message.contains("500"),
        "message must contain the bolt y position '500', got: {}",
        entry.message
    );
    assert!(
        entry.message.contains("350"),
        "message must contain the bound value '350', got: {}",
        entry.message
    );
}
