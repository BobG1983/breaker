//! System to synchronize orbit cell positions from their parent shield.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use crate::cells::components::{OrbitAngle, OrbitCell, OrbitConfig, ShieldParent};

/// Query type for orbit cell data — avoids clippy `type_complexity`.
type OrbitQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Position2D,
        &'static OrbitAngle,
        &'static OrbitConfig,
    ),
    (With<OrbitCell>, Without<ShieldParent>),
>;

/// Writes world-space [`Position2D`] for each orbit cell based on its parent
/// shield's position, orbit radius, and current angle.
///
/// Formula: `orbit_pos = parent_pos + radius * (cos(angle), sin(angle))`
///
/// Uses [`PositionPropagation::Absolute`] on orbit cells so the quadtree
/// sees correct world-space coordinates.
pub(crate) fn sync_orbit_cell_positions(
    parent_query: Query<(&Position2D, &Children), With<ShieldParent>>,
    mut orbit_query: OrbitQuery,
) {
    for (parent_pos, children) in &parent_query {
        for child in children.iter() {
            if let Ok((mut pos, angle, config)) = orbit_query.get_mut(child) {
                pos.0 = parent_pos.0
                    + Vec2::new(config.radius * angle.0.cos(), config.radius * angle.0.sin());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::f32::consts::{FRAC_PI_2, PI};

    use rantzsoft_spatial2d::{components::Spatial2D, propagation::PositionPropagation};

    use super::*;
    use crate::cells::components::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(FixedUpdate, sync_orbit_cell_positions);
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    /// Spawns a shield parent at the given position, with orbit children at
    /// specified angles. Returns `(shield_entity, vec_of_orbit_entities)`.
    fn spawn_shield_with_orbits(
        app: &mut App,
        shield_pos: Vec2,
        radius: f32,
        speed: f32,
        angles: &[f32],
    ) -> (Entity, Vec<Entity>) {
        let shield = app
            .world_mut()
            .spawn((Cell, ShieldParent, Spatial2D, Position2D(shield_pos)))
            .id();

        let mut orbits = Vec::new();
        for &angle in angles {
            let orbit = app
                .world_mut()
                .spawn((
                    Cell,
                    OrbitCell,
                    Spatial2D,
                    PositionPropagation::Absolute,
                    Position2D(Vec2::ZERO), // will be set by sync
                    OrbitAngle(angle),
                    OrbitConfig { radius, speed },
                    ChildOf(shield),
                ))
                .id();
            orbits.push(orbit);
        }

        (shield, orbits)
    }

    // ── Behavior 7: sync_orbit_cell_positions writes world-space Position2D ──

    #[test]
    fn orbit_position_at_angle_zero_is_shield_pos_plus_radius_x() {
        // Given: shield at (100.0, 200.0), orbit at angle 0.0, radius 60.0
        // When: sync runs
        // Then: orbit Position2D = (100.0 + 60.0*cos(0), 200.0 + 60.0*sin(0))
        //                        = (160.0, 200.0)
        let mut app = test_app();
        let (_, orbits) =
            spawn_shield_with_orbits(&mut app, Vec2::new(100.0, 200.0), 60.0, FRAC_PI_2, &[0.0]);

        tick(&mut app);

        let pos = app.world().get::<Position2D>(orbits[0]).unwrap();
        assert!(
            (pos.0.x - 160.0).abs() < 1e-3,
            "orbit x should be 160.0, got {}",
            pos.0.x
        );
        assert!(
            (pos.0.y - 200.0).abs() < 1e-3,
            "orbit y should be 200.0, got {}",
            pos.0.y
        );
    }

    #[test]
    fn orbit_position_at_two_thirds_pi() {
        // Given: shield at (100.0, 200.0), orbit at angle 2*PI/3, radius 60.0
        // When: sync runs
        // Then: orbit Position2D = (100.0 + 60.0*cos(2PI/3), 200.0 + 60.0*sin(2PI/3))
        //       cos(2PI/3) = -0.5, sin(2PI/3) = sqrt(3)/2 ~ 0.866
        //       = (100.0 + 60.0*(-0.5), 200.0 + 60.0*0.866) = (70.0, 251.96)
        let angle = 2.0 * PI / 3.0;
        let mut app = test_app();
        let (_, orbits) =
            spawn_shield_with_orbits(&mut app, Vec2::new(100.0, 200.0), 60.0, FRAC_PI_2, &[angle]);

        tick(&mut app);

        let pos = app.world().get::<Position2D>(orbits[0]).unwrap();
        let expected_x = 60.0f32.mul_add(angle.cos(), 100.0);
        let expected_y = 60.0f32.mul_add(angle.sin(), 200.0);
        assert!(
            (pos.0.x - expected_x).abs() < 0.1,
            "orbit x should be ~{expected_x:.2}, got {:.2}",
            pos.0.x
        );
        assert!(
            (pos.0.y - expected_y).abs() < 0.1,
            "orbit y should be ~{expected_y:.2}, got {:.2}",
            pos.0.y
        );
    }

    #[test]
    fn orbit_position_at_four_thirds_pi() {
        // Given: shield at (100.0, 200.0), orbit at angle 4*PI/3, radius 60.0
        // When: sync runs
        // Then: orbit Position2D = (100.0 + 60.0*cos(4PI/3), 200.0 + 60.0*sin(4PI/3))
        //       cos(4PI/3) = -0.5, sin(4PI/3) = -sqrt(3)/2 ~ -0.866
        //       = (70.0, 148.04)
        let angle = 4.0 * PI / 3.0;
        let mut app = test_app();
        let (_, orbits) =
            spawn_shield_with_orbits(&mut app, Vec2::new(100.0, 200.0), 60.0, FRAC_PI_2, &[angle]);

        tick(&mut app);

        let pos = app.world().get::<Position2D>(orbits[0]).unwrap();
        let expected_x = 60.0f32.mul_add(angle.cos(), 100.0);
        let expected_y = 60.0f32.mul_add(angle.sin(), 200.0);
        assert!(
            (pos.0.x - expected_x).abs() < 0.1,
            "orbit x should be ~{expected_x:.2}, got {:.2}",
            pos.0.x
        );
        assert!(
            (pos.0.y - expected_y).abs() < 0.1,
            "orbit y should be ~{expected_y:.2}, got {:.2}",
            pos.0.y
        );
    }

    #[test]
    fn three_orbits_evenly_spaced_around_shield() {
        // Given: shield at (100.0, 200.0), 3 orbits at 0, 2PI/3, 4PI/3
        // When: sync runs
        // Then: all orbit positions are at radius 60.0 from shield center
        let angles = [0.0, 2.0 * PI / 3.0, 4.0 * PI / 3.0];
        let mut app = test_app();
        let (_, orbits) =
            spawn_shield_with_orbits(&mut app, Vec2::new(100.0, 200.0), 60.0, FRAC_PI_2, &angles);

        tick(&mut app);

        let shield_pos = Vec2::new(100.0, 200.0);
        for (i, &orbit_entity) in orbits.iter().enumerate() {
            let pos = app.world().get::<Position2D>(orbit_entity).unwrap();
            let dist = (pos.0 - shield_pos).length();
            assert!(
                (dist - 60.0).abs() < 0.1,
                "orbit {i} at angle {:.3} should be 60.0 from shield, got {dist:.2}",
                angles[i]
            );
        }
    }

    #[test]
    fn orbit_position_at_pi_over_2() {
        // Given: shield at (0.0, 0.0), orbit at angle PI/2, radius 60.0
        // When: sync runs
        // Then: orbit Position2D = (0.0 + 60.0*cos(PI/2), 0.0 + 60.0*sin(PI/2))
        //       = (0.0, 60.0)
        let mut app = test_app();
        let (_, orbits) =
            spawn_shield_with_orbits(&mut app, Vec2::ZERO, 60.0, FRAC_PI_2, &[FRAC_PI_2]);

        tick(&mut app);

        let pos = app.world().get::<Position2D>(orbits[0]).unwrap();
        assert!(
            pos.0.x.abs() < 1e-3,
            "orbit x should be ~0.0 at PI/2, got {}",
            pos.0.x
        );
        assert!(
            (pos.0.y - 60.0).abs() < 1e-3,
            "orbit y should be ~60.0 at PI/2, got {}",
            pos.0.y
        );
    }
}
