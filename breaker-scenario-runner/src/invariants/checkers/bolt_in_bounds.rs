use bevy::prelude::*;
use breaker::{bolt::components::BoltRadius, shared::PlayfieldConfig};

use crate::{invariants::*, types::InvariantKind};

/// Checks that all [`ScenarioTagBolt`] entities remain within playfield bounds.
///
/// Appends a [`ViolationEntry`] to [`ViolationLog`] for every bolt whose
/// `Transform` translation is outside the top, left, or right playfield boundaries,
/// expanded by `BoltRadius + 1.0` when [`BoltRadius`] is present (zero margin when
/// absent). The bottom is intentionally open (no floor wall) — bolts exit through
/// the bottom during life-loss, so no bottom check is performed.
///
/// Increments [`ScenarioStats::invariant_checks`] by the number of bolts checked.
pub fn check_bolt_in_bounds(
    bolts: Query<(Entity, &Transform, Option<&BoltRadius>), With<ScenarioTagBolt>>,
    playfield: Res<PlayfieldConfig>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
    mut stats: Option<ResMut<ScenarioStats>>,
) {
    let top = playfield.top();
    let left = playfield.left();
    let right = playfield.right();
    let mut checks = 0u32;
    for (entity, transform, bolt_radius) in &bolts {
        checks += 1;
        let x = transform.translation.x;
        let y = transform.translation.y;
        let margin = bolt_radius.map_or(0.0, |r| r.0 + 1.0);
        // No bottom check — the floor is intentionally open (no wall). The bolt
        // exits through the bottom during life-loss, handled by `bolt_lost`.
        if y > top + margin {
            log.0.push(ViolationEntry {
                frame: frame.0,
                invariant: InvariantKind::BoltInBounds,
                entity: Some(entity),
                message: format!(
                    "BoltInBounds FAIL frame={} entity={entity:?} position=(_, {y:.1}) top_bound={top:.1}",
                    frame.0,
                ),
            });
        }
        if x < left - margin {
            log.0.push(ViolationEntry {
                frame: frame.0,
                invariant: InvariantKind::BoltInBounds,
                entity: Some(entity),
                message: format!(
                    "BoltInBounds FAIL frame={} entity={entity:?} position=({x:.1}, _) left_bound={left:.1}",
                    frame.0,
                ),
            });
        }
        if x > right + margin {
            log.0.push(ViolationEntry {
                frame: frame.0,
                invariant: InvariantKind::BoltInBounds,
                entity: Some(entity),
                message: format!(
                    "BoltInBounds FAIL frame={} entity={entity:?} position=({x:.1}, _) right_bound={right:.1}",
                    frame.0,
                ),
            });
        }
    }
    if let Some(ref mut s) = stats {
        s.invariant_checks += checks;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app_bolt_in_bounds() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(PlayfieldConfig::default())
            .insert_resource(ViolationLog::default())
            .insert_resource(ScenarioFrame::default())
            .add_systems(FixedUpdate, check_bolt_in_bounds);
        app
    }

    fn test_app_bolt_in_bounds_with_radius() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(PlayfieldConfig {
                width: 800.0,
                height: 700.0,
                background_color_rgb: [0.0, 0.0, 0.0],
                wall_thickness: 180.0,
            })
            .insert_resource(ViolationLog::default())
            .insert_resource(ScenarioFrame::default())
            .add_systems(FixedUpdate, check_bolt_in_bounds);
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    /// A bolt at y = 500.0 is above the top bound of a playfield with
    /// height 700.0 (top = 350.0). The system must append one
    /// [`ViolationEntry`] with [`InvariantKind::BoltInBounds`], frame 1842,
    /// the entity id, and a message containing the actual position and the bound.
    #[test]
    fn bolt_in_bounds_appends_violation_when_bolt_is_above_top_bound() {
        let mut app = test_app_bolt_in_bounds();

        // height 700.0 → top() = 350.0
        app.world_mut().insert_resource(PlayfieldConfig {
            width: 800.0,
            height: 700.0,
            background_color_rgb: [0.0, 0.0, 0.0],
            wall_thickness: 180.0,
        });
        app.world_mut().insert_resource(ScenarioFrame(1842));

        let bolt_entity = app
            .world_mut()
            .spawn((
                ScenarioTagBolt,
                Transform::from_translation(Vec3::new(0.0, 500.0, 0.0)),
            ))
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
        });
        app.world_mut().insert_resource(ScenarioFrame(10));

        app.world_mut().spawn((
            ScenarioTagBolt,
            Transform::from_translation(Vec3::new(0.0, -100.0, 0.0)),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violations for in-bounds bolt at y = -100.0, got: {:?}",
            log.0.iter().map(|e| &e.message).collect::<Vec<_>>()
        );
    }

    /// A bolt exactly at y = -350.0 (the bottom boundary of a 700.0-height
    /// playfield) should NOT trigger a violation — it is at the edge, not below.
    #[test]
    fn bolt_in_bounds_does_not_fire_when_bolt_is_exactly_at_bottom_bound() {
        let mut app = test_app_bolt_in_bounds();

        app.world_mut().insert_resource(PlayfieldConfig {
            width: 800.0,
            height: 700.0,
            background_color_rgb: [0.0, 0.0, 0.0],
            wall_thickness: 180.0,
        });
        app.world_mut().insert_resource(ScenarioFrame(0));

        // `PlayfieldConfig::bottom()` returns -350.0 for height 700.0
        app.world_mut().spawn((
            ScenarioTagBolt,
            Transform::from_translation(Vec3::new(0.0, -350.0, 0.0)),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when bolt is exactly at bottom bound (-350.0)"
        );
    }

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
        });
        app.world_mut().insert_resource(ScenarioStats::default());

        // Spawn 3 tagged bolts, all in-bounds
        for _ in 0..3 {
            app.world_mut().spawn((
                ScenarioTagBolt,
                Transform::from_translation(Vec3::new(0.0, 0.0, 0.0)),
            ));
        }

        tick(&mut app);

        let stats = app.world().resource::<ScenarioStats>();
        assert_eq!(
            stats.invariant_checks, 3,
            "expected invariant_checks=3 for 3 tagged bolts, got {}",
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
        });
        app.world_mut().insert_resource(ScenarioFrame(1842));

        let bolt_entity = app
            .world_mut()
            .spawn((
                ScenarioTagBolt,
                Transform::from_translation(Vec3::new(0.0, 500.0, 0.0)),
            ))
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

    /// A bolt at y = 1000.0 exceeds the top bound of a playfield with height=700.0
    /// (top = 350.0). The system must append one [`ViolationEntry`] with
    /// [`InvariantKind::BoltInBounds`].
    #[test]
    fn bolt_in_bounds_fires_when_bolt_is_above_top_bound() {
        let mut app = test_app_bolt_in_bounds();

        // width=800.0, height=700.0 → top() = 350.0
        app.world_mut().insert_resource(PlayfieldConfig {
            width: 800.0,
            height: 700.0,
            background_color_rgb: [0.0, 0.0, 0.0],
            wall_thickness: 180.0,
        });
        app.world_mut().insert_resource(ScenarioFrame(1));

        app.world_mut().spawn((
            ScenarioTagBolt,
            Transform::from_translation(Vec3::new(0.0, 1000.0, 0.0)),
        ));

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
    /// must NOT trigger a violation — the check is strict `>`.
    #[test]
    fn bolt_in_bounds_does_not_fire_when_bolt_is_exactly_at_top_bound() {
        let mut app = test_app_bolt_in_bounds();

        // top() = 700.0 / 2.0 = 350.0
        app.world_mut().insert_resource(PlayfieldConfig {
            width: 800.0,
            height: 700.0,
            background_color_rgb: [0.0, 0.0, 0.0],
            wall_thickness: 180.0,
        });
        app.world_mut().insert_resource(ScenarioFrame(1));

        app.world_mut().spawn((
            ScenarioTagBolt,
            Transform::from_translation(Vec3::new(0.0, 350.0, 0.0)),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when bolt is exactly at top bound (350.0) — check is strict >"
        );
    }

    /// A bolt at x = -2000.0 exceeds the left bound of a playfield with
    /// width=800.0 (left = -400.0). The system must append one
    /// [`ViolationEntry`] with [`InvariantKind::BoltInBounds`].
    #[test]
    fn bolt_in_bounds_fires_when_bolt_is_left_of_left_bound() {
        let mut app = test_app_bolt_in_bounds();

        // width=800.0 → left() = -400.0
        app.world_mut().insert_resource(PlayfieldConfig {
            width: 800.0,
            height: 700.0,
            background_color_rgb: [0.0, 0.0, 0.0],
            wall_thickness: 180.0,
        });
        app.world_mut().insert_resource(ScenarioFrame(1));

        app.world_mut().spawn((
            ScenarioTagBolt,
            Transform::from_translation(Vec3::new(-2000.0, 0.0, 0.0)),
        ));

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
    /// [`ViolationEntry`] with [`InvariantKind::BoltInBounds`].
    #[test]
    fn bolt_in_bounds_fires_when_bolt_is_right_of_right_bound() {
        let mut app = test_app_bolt_in_bounds();

        // width=800.0 → right() = 400.0
        app.world_mut().insert_resource(PlayfieldConfig {
            width: 800.0,
            height: 700.0,
            background_color_rgb: [0.0, 0.0, 0.0],
            wall_thickness: 180.0,
        });
        app.world_mut().insert_resource(ScenarioFrame(1));

        app.world_mut().spawn((
            ScenarioTagBolt,
            Transform::from_translation(Vec3::new(2000.0, 0.0, 0.0)),
        ));

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

    /// Playfield height=700.0 → bottom=-350.0. Bolt at y=-358.0 with BoltRadius(8.0).
    /// The allowed margin is `bottom - (radius + 1.0)` = -350.0 - 9.0 = -359.0.
    /// At -358.0 the bolt center is within the radius margin → no violation.
    #[test]
    fn bolt_in_bounds_no_violation_when_bolt_slightly_below_bottom_within_radius_margin() {
        let mut app = test_app_bolt_in_bounds_with_radius();

        app.world_mut().spawn((
            ScenarioTagBolt,
            Transform::from_translation(Vec3::new(0.0, -358.0, 0.0)),
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

    /// Bolt at y=500.0 with BoltRadius(8.0). The allowed margin is top + 9.0 = 359.0.
    /// 500.0 is well beyond 359.0 → violation fires.
    #[test]
    fn bolt_in_bounds_fires_when_bolt_far_above_top_beyond_radius_margin() {
        let mut app = test_app_bolt_in_bounds_with_radius();

        // top() = 350.0, margin = 8.0 + 1.0 = 9.0; allowed = 359.0; 500.0 well beyond
        app.world_mut().spawn((
            ScenarioTagBolt,
            Transform::from_translation(Vec3::new(0.0, 500.0, 0.0)),
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

    /// Playfield width=800.0 → right=400.0. Bolt at x=408.0 with BoltRadius(8.0).
    /// The allowed margin is `right + (radius + 1.0)` = 400.0 + 9.0 = 409.0.
    /// At 408.0 the bolt center is within the radius margin → no violation.
    #[test]
    fn bolt_in_bounds_no_violation_when_bolt_slightly_past_right_wall_within_radius_margin() {
        let mut app = test_app_bolt_in_bounds_with_radius();

        app.world_mut().spawn((
            ScenarioTagBolt,
            Transform::from_translation(Vec3::new(408.0, 0.0, 0.0)),
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

    /// Bolt at y=-350.0 (exactly the bottom boundary) with BoltRadius(8.0).
    /// The bolt center is exactly at the boundary — well within the radius margin
    /// of -359.0. No violation must fire.
    #[test]
    fn bolt_in_bounds_no_violation_when_bolt_center_at_exact_boundary_with_radius() {
        let mut app = test_app_bolt_in_bounds_with_radius();

        app.world_mut().spawn((
            ScenarioTagBolt,
            Transform::from_translation(Vec3::new(0.0, -350.0, 0.0)),
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
            Transform::from_translation(Vec3::new(0.0, -1000.0, 0.0)),
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
}
