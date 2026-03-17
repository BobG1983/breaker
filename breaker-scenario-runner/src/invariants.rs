//! Invariant checker systems and `ViolationLog` resource.
//!
//! Invariant systems run in `FixedUpdate` after gameplay. They query game state
//! and append to a [`ViolationLog`] resource. They never panic — they collect
//! all violations for end-of-run reporting.

use bevy::prelude::*;
use breaker::shared::PlayfieldConfig;

use crate::types::InvariantKind;

/// Query filter that matches entities tagged for invariant checking.
type TaggedTransformQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static Transform),
    Or<(With<ScenarioTagBolt>, With<ScenarioTagBreaker>)>,
>;

/// Marker — attached by the lifecycle plugin to the bolt entity for invariant checking.
#[derive(Component)]
pub struct ScenarioTagBolt;

/// Marker — attached by the lifecycle plugin to the breaker entity for invariant checking.
#[derive(Component)]
pub struct ScenarioTagBreaker;

/// Tracks the current fixed-update frame number for violation logging.
#[derive(Resource, Default)]
pub struct ScenarioFrame(pub u32);

/// A single invariant violation recorded during a scenario run.
#[derive(Debug, Clone)]
pub struct ViolationEntry {
    /// Fixed-update frame when the violation was detected.
    pub frame: u32,
    /// Which invariant was violated.
    pub invariant: InvariantKind,
    /// Entity involved, if applicable.
    pub entity: Option<Entity>,
    /// Human-readable description with concrete values.
    pub message: String,
}

/// Accumulated violations from all invariant checks.
#[derive(Resource, Default)]
pub struct ViolationLog(pub Vec<ViolationEntry>);

/// Checks that all [`ScenarioTagBolt`] entities remain within playfield bounds.
///
/// Appends a [`ViolationEntry`] to [`ViolationLog`] for every bolt whose
/// `Transform` translation y is below `PlayfieldConfig::bottom()`.
pub fn check_bolt_in_bounds(
    bolts: Query<(Entity, &Transform), With<ScenarioTagBolt>>,
    playfield: Res<PlayfieldConfig>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
) {
    let bottom = playfield.bottom();
    for (entity, transform) in &bolts {
        let y = transform.translation.y;
        if y < bottom {
            log.0.push(ViolationEntry {
                frame: frame.0,
                invariant: InvariantKind::BoltInBounds,
                entity: Some(entity),
                message: format!(
                    "BoltInBounds FAIL frame={} entity={entity:?} position=(_, {y:.1}) bottom_bound={bottom:.1}",
                    frame.0,
                ),
            });
        }
    }
}

/// Checks that all tagged entities have finite `Transform` values (no NaN or Inf).
///
/// Appends a [`ViolationEntry`] to [`ViolationLog`] for every entity whose
/// translation or rotation contains a non-finite value.
pub fn check_no_nan(
    tagged: TaggedTransformQuery,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
) {
    for (entity, transform) in &tagged {
        let t = transform.translation;
        let r = transform.rotation;
        if !t.is_finite() || !r.is_finite() {
            log.0.push(ViolationEntry {
                frame: frame.0,
                invariant: InvariantKind::NoNaN,
                entity: Some(entity),
                message: format!(
                    "NoNaN FAIL frame={} entity={entity:?} translation={t:?} rotation={r:?}",
                    frame.0,
                ),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // -------------------------------------------------------------------------
    // Helpers
    // -------------------------------------------------------------------------

    /// Build a minimal test app with `check_bolt_in_bounds` registered plus
    /// required resources pre-inserted.
    fn test_app_bolt_in_bounds() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(PlayfieldConfig::default());
        app.insert_resource(ViolationLog::default());
        app.insert_resource(ScenarioFrame::default());
        app.add_systems(FixedUpdate, check_bolt_in_bounds);
        app
    }

    /// Build a minimal test app with `check_no_nan` registered plus
    /// required resources pre-inserted.
    fn test_app_no_nan() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(ViolationLog::default());
        app.insert_resource(ScenarioFrame::default());
        app.add_systems(FixedUpdate, check_no_nan);
        app
    }

    /// Advance one fixed-update timestep and run one update.
    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    // -------------------------------------------------------------------------
    // BoltInBounds — violation fires when bolt is below bottom bound
    // -------------------------------------------------------------------------

    /// A bolt at y = -500.0 is below the bottom bound of a playfield with
    /// height 700.0 (bottom = -350.0). The system must append one
    /// [`ViolationEntry`] with [`InvariantKind::BoltInBounds`], frame 1842,
    /// the entity id, and a message containing the actual position and the bound.
    #[test]
    fn bolt_in_bounds_appends_violation_when_bolt_is_below_bottom_bound() {
        let mut app = test_app_bolt_in_bounds();

        // height 700.0 → bottom() = -350.0
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
                Transform::from_translation(Vec3::new(0.0, -500.0, 0.0)),
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
            entry.message.contains("-500"),
            "message should contain bolt y '-500', got: {}",
            entry.message
        );
        assert!(
            entry.message.contains("-350"),
            "message should contain bound '-350', got: {}",
            entry.message
        );
    }

    // -------------------------------------------------------------------------
    // BoltInBounds — no violation when bolt is within bounds
    // -------------------------------------------------------------------------

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

    // -------------------------------------------------------------------------
    // BoltInBounds — edge case: bolt exactly at the bottom bound
    // -------------------------------------------------------------------------

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

    // -------------------------------------------------------------------------
    // ViolationEntry — fields populated correctly
    // -------------------------------------------------------------------------

    /// Verifies that the entry from the out-of-bounds case has all required
    /// fields set: frame, invariant, entity (`Some`), and a message with values.
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
                Transform::from_translation(Vec3::new(0.0, -500.0, 0.0)),
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
            entry.message.contains("-500"),
            "message must contain the bolt y position '-500', got: {}",
            entry.message
        );
        assert!(
            entry.message.contains("-350"),
            "message must contain the bound value '-350', got: {}",
            entry.message
        );
    }

    // -------------------------------------------------------------------------
    // NoNaN — violation fires when Transform translation has NaN
    // -------------------------------------------------------------------------

    /// A bolt entity with `f32::NAN` in the x component of translation triggers a
    /// [`ViolationEntry`] with [`InvariantKind::NoNaN`], frame 5, and a message
    /// containing "NaN".
    #[test]
    fn no_nan_appends_violation_when_transform_translation_has_nan() {
        let mut app = test_app_no_nan();

        app.world_mut().insert_resource(ScenarioFrame(5));

        app.world_mut().spawn((
            ScenarioTagBolt,
            Transform::from_translation(Vec3::new(f32::NAN, 0.0, 0.0)),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected exactly one NaN violation, got {}",
            log.0.len()
        );

        let entry = &log.0[0];
        assert_eq!(entry.invariant, InvariantKind::NoNaN);
        assert_eq!(entry.frame, 5);
        assert!(
            entry.message.contains("NaN"),
            "message must contain 'NaN', got: {}",
            entry.message
        );
    }

    // -------------------------------------------------------------------------
    // NoNaN — no violation for finite transforms
    // -------------------------------------------------------------------------

    /// A bolt at (1.0, 2.0, 0.0) with `Quat::IDENTITY` rotation is fully finite.
    /// No violations should be recorded.
    #[test]
    fn no_nan_does_not_fire_for_finite_transforms() {
        let mut app = test_app_no_nan();

        app.world_mut().insert_resource(ScenarioFrame(0));

        app.world_mut().spawn((
            ScenarioTagBolt,
            Transform {
                translation: Vec3::new(1.0, 2.0, 0.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violations for finite transform, got: {:?}",
            log.0.iter().map(|e| &e.message).collect::<Vec<_>>()
        );
    }

    // -------------------------------------------------------------------------
    // NoNaN — ScenarioTagBreaker entities are also checked
    // -------------------------------------------------------------------------

    /// The `check_no_nan` system covers both [`ScenarioTagBolt`] and
    /// [`ScenarioTagBreaker`] entities. A breaker with `f32::NAN` in its
    /// y translation should also trigger a violation.
    #[test]
    fn no_nan_fires_for_breaker_tagged_entity_with_nan_translation() {
        let mut app = test_app_no_nan();

        app.world_mut().insert_resource(ScenarioFrame(99));

        app.world_mut().spawn((
            ScenarioTagBreaker,
            Transform::from_translation(Vec3::new(0.0, f32::NAN, 0.0)),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            !log.0.is_empty(),
            "expected a NaN violation for ScenarioTagBreaker entity"
        );
        assert_eq!(log.0[0].invariant, InvariantKind::NoNaN);
        assert!(
            log.0[0].message.contains("NaN"),
            "message must contain 'NaN', got: {}",
            log.0[0].message
        );
    }

    // -------------------------------------------------------------------------
    // NoNaN — NaN in rotation triggers violation
    // -------------------------------------------------------------------------

    /// A transform with a NaN quaternion component should also trigger
    /// [`InvariantKind::NoNaN`]. This covers the rotation field, not just
    /// translation.
    #[test]
    fn no_nan_fires_when_rotation_contains_nan() {
        let mut app = test_app_no_nan();

        app.world_mut().insert_resource(ScenarioFrame(7));

        app.world_mut().spawn((
            ScenarioTagBolt,
            Transform {
                translation: Vec3::new(0.0, 0.0, 0.0),
                rotation: Quat::from_xyzw(0.0, 0.0, 0.0, f32::NAN),
                scale: Vec3::ONE,
            },
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            !log.0.is_empty(),
            "expected NoNaN violation for transform with NaN rotation"
        );
        assert_eq!(log.0[0].invariant, InvariantKind::NoNaN);
    }
}
