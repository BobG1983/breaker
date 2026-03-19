use bevy::prelude::*;

use crate::{invariants::*, types::InvariantKind};

/// Query filter that matches entities tagged for invariant checking.
type TaggedTransformQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static Transform),
    Or<(With<ScenarioTagBolt>, With<ScenarioTagBreaker>)>,
>;

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

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn test_app_no_nan() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ViolationLog::default())
            .insert_resource(ScenarioFrame::default())
            .add_systems(FixedUpdate, check_no_nan);
        app
    }

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
