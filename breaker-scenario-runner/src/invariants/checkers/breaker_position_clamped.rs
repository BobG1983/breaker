use bevy::prelude::*;
use breaker::{breaker::components::BreakerWidth, shared::PlayfieldConfig};

use crate::{invariants::*, types::InvariantKind};

/// Checks that the tagged breaker's x position stays within `playfield.right() - half_width`.
///
/// Appends a [`ViolationEntry`] with [`InvariantKind::BreakerPositionClamped`] when the
/// breaker is outside the tight clamping bounds (with 1px tolerance).
pub fn check_breaker_position_clamped(
    breakers: Query<(Entity, &Transform, &BreakerWidth), With<ScenarioTagBreaker>>,
    playfield: Res<PlayfieldConfig>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
) {
    let tolerance = 1.0_f32;
    for (entity, transform, width) in &breakers {
        let half_width = width.half_width();
        let max_x = playfield.right() - half_width;
        let min_x = playfield.left() + half_width;
        let x = transform.translation.x;
        if x > max_x + tolerance || x < min_x - tolerance {
            log.0.push(ViolationEntry {
                frame: frame.0,
                invariant: InvariantKind::BreakerPositionClamped,
                entity: Some(entity),
                message: format!(
                    "BreakerPositionClamped FAIL frame={} entity={entity:?} x={x:.1} bounds=[{min_x:.1}, {max_x:.1}]",
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

    fn test_app_breaker_position_clamped() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ViolationLog::default())
            .insert_resource(ScenarioFrame::default())
            .insert_resource(PlayfieldConfig {
                width: 800.0,
                height: 700.0,
                background_color_rgb: [0.0, 0.0, 0.0],
                wall_thickness: 180.0,
                zone_fraction: 0.667,
            })
            .add_systems(FixedUpdate, check_breaker_position_clamped);
        app
    }

    /// Breaker at x=1000.0 is well outside `right() - half_width` (400.0 - 60.0 = 340.0).
    /// A [`ViolationEntry`] with [`InvariantKind::BreakerPositionClamped`] must fire.
    #[test]
    fn breaker_position_clamped_fires_when_outside_bounds() {
        let mut app = test_app_breaker_position_clamped();

        // BreakerWidth(120.0) → half_width = 60.0; right() = 400.0 → clamped max = 340.0
        app.world_mut().spawn((
            ScenarioTagBreaker,
            Transform::from_translation(Vec3::new(1000.0, -250.0, 0.0)),
            BreakerWidth(120.0),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected exactly one BreakerPositionClamped violation, got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::BreakerPositionClamped);
    }

    /// Breaker at x=0.0 is well within bounds. No violation should fire.
    #[test]
    fn breaker_position_clamped_does_not_fire_when_within_bounds() {
        let mut app = test_app_breaker_position_clamped();

        app.world_mut().spawn((
            ScenarioTagBreaker,
            Transform::from_translation(Vec3::new(0.0, -250.0, 0.0)),
            BreakerWidth(120.0),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation for breaker at x=0.0"
        );
    }

    /// Breaker at x = 340.0 (exactly `right() - half_width = 400.0 - 60.0`)
    /// is within tolerance. No violation should fire.
    #[test]
    fn breaker_position_clamped_does_not_fire_at_exact_boundary() {
        let mut app = test_app_breaker_position_clamped();

        // Exact boundary: right() - half_width = 400.0 - 60.0 = 340.0
        app.world_mut().spawn((
            ScenarioTagBreaker,
            Transform::from_translation(Vec3::new(340.0, -250.0, 0.0)),
            BreakerWidth(120.0),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when breaker is exactly at clamped boundary (340.0)"
        );
    }
}
