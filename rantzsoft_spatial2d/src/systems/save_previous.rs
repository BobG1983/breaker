//! Snapshots current position/rotation/scale to previous for interpolation.

use bevy::prelude::*;

use crate::components::{
    InterpolateTransform2D, Position2D, PreviousPosition, PreviousRotation, PreviousScale,
    Rotation2D, Scale2D,
};

/// Copies current `Position2D`, `Rotation2D`, and `Scale2D` into their
/// previous-frame snapshots for entities that have `InterpolateTransform2D`.
pub fn save_previous(
    mut query_pos: Query<(&Position2D, &mut PreviousPosition), With<InterpolateTransform2D>>,
    mut query_rot: Query<(&Rotation2D, &mut PreviousRotation), With<InterpolateTransform2D>>,
    mut query_scale: Query<(&Scale2D, &mut PreviousScale), With<InterpolateTransform2D>>,
) {
    for (pos, mut prev) in &mut query_pos {
        prev.0 = pos.0;
    }
    for (rot, mut prev) in &mut query_rot {
        prev.0 = rot.0;
    }
    for (scale, mut prev) in &mut query_scale {
        prev.x = scale.x;
        prev.y = scale.y;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::{PreviousScale, Scale2D};

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    #[test]
    fn copies_position_to_previous_position() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, save_previous);

        app.world_mut().spawn((
            InterpolateTransform2D,
            Position2D(Vec2::new(10.0, 20.0)),
            PreviousPosition(Vec2::ZERO),
            Rotation2D::default(),
            PreviousRotation::default(),
        ));

        tick(&mut app);

        let prev = app
            .world_mut()
            .query::<&PreviousPosition>()
            .iter(app.world())
            .next()
            .expect("entity should exist");

        assert_eq!(
            prev.0,
            Vec2::new(10.0, 20.0),
            "PreviousPosition should match current Position2D"
        );
    }

    #[test]
    fn copies_rotation_to_previous_rotation() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, save_previous);

        app.world_mut().spawn((
            InterpolateTransform2D,
            Position2D::default(),
            PreviousPosition::default(),
            Rotation2D::from_degrees(45.0),
            PreviousRotation::default(),
        ));

        tick(&mut app);

        let prev = app
            .world_mut()
            .query::<&PreviousRotation>()
            .iter(app.world())
            .next()
            .expect("entity should exist");

        assert!(
            (prev.0.as_radians() - std::f32::consts::FRAC_PI_4).abs() < 1e-5,
            "PreviousRotation should be ~45 degrees, got {} radians",
            prev.0.as_radians()
        );
    }

    #[test]
    fn skips_entity_without_interpolate_marker_position() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, save_previous);

        // No InterpolateTransform2D marker
        app.world_mut().spawn((
            Position2D(Vec2::new(99.0, 99.0)),
            PreviousPosition(Vec2::ZERO),
        ));

        tick(&mut app);

        let prev = app
            .world_mut()
            .query::<&PreviousPosition>()
            .iter(app.world())
            .next()
            .expect("entity should exist");

        assert_eq!(
            prev.0,
            Vec2::ZERO,
            "PreviousPosition should be unchanged without InterpolateTransform2D"
        );
    }

    #[test]
    fn skips_entity_without_interpolate_marker_rotation() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, save_previous);

        // No InterpolateTransform2D marker
        app.world_mut()
            .spawn((Rotation2D::from_degrees(90.0), PreviousRotation::default()));

        tick(&mut app);

        let prev = app
            .world_mut()
            .query::<&PreviousRotation>()
            .iter(app.world())
            .next()
            .expect("entity should exist");

        assert!(
            prev.0.as_radians().abs() < 1e-6,
            "PreviousRotation should be unchanged without InterpolateTransform2D"
        );
    }

    #[test]
    fn multiple_entities_all_updated() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, save_previous);

        let e1 = app
            .world_mut()
            .spawn((
                InterpolateTransform2D,
                Position2D(Vec2::new(1.0, 2.0)),
                PreviousPosition(Vec2::ZERO),
                Rotation2D::default(),
                PreviousRotation::default(),
            ))
            .id();

        let e2 = app
            .world_mut()
            .spawn((
                InterpolateTransform2D,
                Position2D(Vec2::new(30.0, 40.0)),
                PreviousPosition(Vec2::ZERO),
                Rotation2D::default(),
                PreviousRotation::default(),
            ))
            .id();

        tick(&mut app);

        let prev1 = app.world().get::<PreviousPosition>(e1).expect("e1 exists");
        let prev2 = app.world().get::<PreviousPosition>(e2).expect("e2 exists");

        assert_eq!(
            prev1.0,
            Vec2::new(1.0, 2.0),
            "first entity PreviousPosition should be updated"
        );
        assert_eq!(
            prev2.0,
            Vec2::new(30.0, 40.0),
            "second entity PreviousPosition should be updated"
        );
    }

    // ── PreviousScale ─────────────────────────────────────────

    #[test]
    fn copies_scale_to_previous_scale() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, save_previous);

        app.world_mut().spawn((
            InterpolateTransform2D,
            Scale2D { x: 3.0, y: 4.0 },
            PreviousScale { x: 1.0, y: 1.0 },
            Position2D::default(),
            PreviousPosition::default(),
            Rotation2D::default(),
            PreviousRotation::default(),
        ));

        tick(&mut app);

        let prev = app
            .world_mut()
            .query::<&PreviousScale>()
            .iter(app.world())
            .next()
            .expect("entity should exist");

        assert!(
            (prev.x - 3.0).abs() < f32::EPSILON,
            "PreviousScale.x should match current Scale2D.x (3.0), got {}",
            prev.x
        );
        assert!(
            (prev.y - 4.0).abs() < f32::EPSILON,
            "PreviousScale.y should match current Scale2D.y (4.0), got {}",
            prev.y
        );
    }

    #[test]
    fn skips_scale_without_interpolate_marker() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, save_previous);

        // No InterpolateTransform2D marker.
        app.world_mut()
            .spawn((Scale2D { x: 5.0, y: 6.0 }, PreviousScale { x: 1.0, y: 1.0 }));

        tick(&mut app);

        let prev = app
            .world_mut()
            .query::<&PreviousScale>()
            .iter(app.world())
            .next()
            .expect("entity should exist");

        assert!(
            (prev.x - 1.0).abs() < f32::EPSILON,
            "PreviousScale.x should remain 1.0 without InterpolateTransform2D, got {}",
            prev.x
        );
        assert!(
            (prev.y - 1.0).abs() < f32::EPSILON,
            "PreviousScale.y should remain 1.0 without InterpolateTransform2D, got {}",
            prev.y
        );
    }
}
