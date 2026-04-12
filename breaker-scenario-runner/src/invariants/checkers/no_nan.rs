use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use crate::{invariants::*, types::InvariantKind};

/// Query filter that matches entities tagged for invariant checking.
type TaggedPositionQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static Position2D),
    Or<(With<ScenarioTagBolt>, With<ScenarioTagBreaker>)>,
>;

/// Checks that all tagged entities have finite `Position2D` values (no NaN or Inf).
///
/// Appends a [`ViolationEntry`] to [`ViolationLog`] for every entity whose
/// position contains a non-finite value.
pub fn check_no_nan(
    tagged: TaggedPositionQuery,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
    mut stats: Option<ResMut<ScenarioStats>>,
) {
    if let Some(ref mut s) = stats {
        s.invariant_checks += 1;
    }
    for (entity, position) in &tagged {
        let p = position.0;
        if !p.is_finite() {
            log.0.push(ViolationEntry {
                frame:     frame.0,
                invariant: InvariantKind::NoNaN,
                entity:    Some(entity),
                message:   format!(
                    "NoNaN FAIL frame={} entity={entity:?} position={p:?}",
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

    /// A bolt entity with `f32::NAN` in the x component of position triggers a
    /// [`ViolationEntry`] with [`InvariantKind::NoNaN`], frame 5, and a message
    /// containing "NaN".
    #[test]
    fn no_nan_appends_violation_when_position_has_nan() {
        let mut app = test_app_no_nan();

        app.world_mut().insert_resource(ScenarioFrame(5));

        app.world_mut()
            .spawn((ScenarioTagBolt, Position2D(Vec2::new(f32::NAN, 0.0))));

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

    /// A bolt at (1.0, 2.0) is fully finite.
    /// No violations should be recorded.
    #[test]
    fn no_nan_does_not_fire_for_finite_positions() {
        let mut app = test_app_no_nan();

        app.world_mut().insert_resource(ScenarioFrame(0));

        app.world_mut()
            .spawn((ScenarioTagBolt, Position2D(Vec2::new(1.0, 2.0))));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violations for finite position, got: {:?}",
            log.0.iter().map(|e| &e.message).collect::<Vec<_>>()
        );
    }

    /// The `check_no_nan` system covers both [`ScenarioTagBolt`] and
    /// [`ScenarioTagBreaker`] entities. A breaker with `f32::NAN` in its
    /// y position should also trigger a violation.
    #[test]
    fn no_nan_fires_for_breaker_tagged_entity_with_nan_position() {
        let mut app = test_app_no_nan();

        app.world_mut().insert_resource(ScenarioFrame(99));

        app.world_mut()
            .spawn((ScenarioTagBreaker, Position2D(Vec2::new(0.0, f32::NAN))));

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
}
