use bevy::prelude::*;
use breaker::shared::PlayfieldConfig;
use rantzsoft_spatial2d::components::Position2D;

use crate::{invariants::*, types::InvariantKind};

/// Checks that all [`ScenarioTagBreaker`] entities remain within playfield bounds.
///
/// Appends a [`ViolationEntry`] for every breaker whose `Position2D` x
/// is outside `PlayfieldConfig::left()` or `PlayfieldConfig::right()` (with 50.0 margin).
pub fn check_breaker_in_bounds(
    breakers: Query<(Entity, &Position2D), With<ScenarioTagBreaker>>,
    playfield: Res<PlayfieldConfig>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
    mut stats: Option<ResMut<ScenarioStats>>,
) {
    if let Some(ref mut s) = stats {
        s.invariant_checks += 1;
    }
    let margin = 50.0;
    let left = playfield.left() - margin;
    let right = playfield.right() + margin;
    for (entity, position) in &breakers {
        let x = position.0.x;
        if x < left || x > right {
            log.0.push(ViolationEntry {
                frame: frame.0,
                invariant: InvariantKind::BreakerInBounds,
                entity: Some(entity),
                message: format!(
                    "BreakerInBounds FAIL frame={} entity={entity:?} x={x:.1} bounds=[{left:.1}, {right:.1}]",
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

    fn test_app_breaker_in_bounds() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(PlayfieldConfig::default())
            .insert_resource(ViolationLog::default())
            .insert_resource(ScenarioFrame::default())
            .add_systems(FixedUpdate, check_breaker_in_bounds);
        app
    }

    #[test]
    fn breaker_in_bounds_fires_when_breaker_far_outside_right() {
        let mut app = test_app_breaker_in_bounds();

        app.world_mut()
            .spawn((ScenarioTagBreaker, Position2D(Vec2::new(1000.0, 0.0))));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert_eq!(log.0.len(), 1);
        assert_eq!(log.0[0].invariant, InvariantKind::BreakerInBounds);
    }

    #[test]
    fn breaker_in_bounds_does_not_fire_when_breaker_centered() {
        let mut app = test_app_breaker_in_bounds();

        app.world_mut()
            .spawn((ScenarioTagBreaker, Position2D(Vec2::new(0.0, 0.0))));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(log.0.is_empty());
    }
}
