//! Steers bolts toward their active attraction targets each fixed tick.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

use crate::{
    bolt::components::Bolt,
    cells::components::Cell,
    effect::{
        definition::AttractionType,
        effects::{attraction::ActiveAttractions, second_wind::SecondWindWall},
    },
    wall::components::Wall,
};

/// Steers each bolt toward the nearest entity matching its active attraction
/// types, preserving speed while biasing direction.
pub(crate) fn apply_attraction(
    time: Res<Time<Fixed>>,
    mut bolt_query: Query<(&mut Velocity2D, &Position2D, &ActiveAttractions), With<Bolt>>,
    cell_query: Query<&Position2D, With<Cell>>,
    wall_query: Query<&Position2D, (With<Wall>, Without<SecondWindWall>)>,
) {
    let dt = time.delta_secs();
    for (mut vel, bolt_pos, attractions) in &mut bolt_query {
        let original_speed = vel.0.length();
        if original_speed < f32::EPSILON {
            continue;
        }

        let mut steering = Vec2::ZERO;

        for entry in &attractions.entries {
            if !entry.active {
                continue;
            }

            let nearest_pos = match entry.attraction_type {
                AttractionType::Cell => nearest_in(cell_query.iter(), bolt_pos.0),
                AttractionType::Wall => nearest_in(wall_query.iter(), bolt_pos.0),
                AttractionType::Breaker => None,
            };

            if let Some(target_pos) = nearest_pos {
                let dir = (target_pos - bolt_pos.0).normalize_or_zero();
                steering += dir * entry.force;
            }
        }

        vel.0 += steering * dt;
        // Preserve original speed.
        vel.0 = vel.0.normalize_or_zero() * original_speed;
    }
}

/// Finds the closest position from an iterator of `Position2D` references.
fn nearest_in<'a>(iter: impl Iterator<Item = &'a Position2D>, from: Vec2) -> Option<Vec2> {
    iter.map(|p| (p.0, p.0.distance_squared(from)))
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(pos, _)| pos)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_systems(FixedUpdate, apply_attraction);
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    #[test]
    fn apply_attraction_steers_bolt_toward_nearest_cell() {
        let mut app = test_app();

        // Bolt at origin, moving straight up
        app.world_mut().spawn((
            Bolt,
            Position2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::new(0.0, 400.0)),
            ActiveAttractions {
                entries: vec![crate::effect::effects::attraction::AttractionEntry {
                    attraction_type: crate::effect::definition::AttractionType::Cell,
                    force: 10.0,
                    active: true,
                }],
            },
        ));

        // Cell to the right and above
        app.world_mut()
            .spawn((Cell, Position2D(Vec2::new(50.0, 100.0))));

        tick(&mut app);

        let vel = app
            .world_mut()
            .query_filtered::<&Velocity2D, With<Bolt>>()
            .iter(app.world())
            .next()
            .expect("bolt entity should exist");

        assert!(
            vel.0.x > 0.0,
            "bolt velocity.x should bias toward cell at x=50, got vx={:.3}",
            vel.0.x
        );

        let speed = vel.0.length();
        assert!(
            (speed - 400.0).abs() < 1.0,
            "bolt speed should be preserved at ~400.0, got {speed:.3}"
        );
    }

    #[test]
    fn apply_attraction_skips_inactive_types() {
        let mut app = test_app();

        // Bolt at origin, moving straight up, but attraction is inactive
        app.world_mut().spawn((
            Bolt,
            Position2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::new(0.0, 400.0)),
            ActiveAttractions {
                entries: vec![crate::effect::effects::attraction::AttractionEntry {
                    attraction_type: crate::effect::definition::AttractionType::Cell,
                    force: 10.0,
                    active: false,
                }],
            },
        ));

        // Cell to the right
        app.world_mut()
            .spawn((Cell, Position2D(Vec2::new(50.0, 100.0))));

        tick(&mut app);

        let vel = app
            .world_mut()
            .query_filtered::<&Velocity2D, With<Bolt>>()
            .iter(app.world())
            .next()
            .expect("bolt entity should exist");

        assert!(
            (vel.0.x - 0.0).abs() < f32::EPSILON,
            "inactive attraction should not steer bolt, got vx={:.3}",
            vel.0.x
        );
        assert!(
            (vel.0.y - 400.0).abs() < f32::EPSILON,
            "velocity.y should be unchanged at 400.0, got vy={:.3}",
            vel.0.y
        );
    }

    #[test]
    fn apply_attraction_filters_second_wind_wall_from_wall_attraction() {
        let mut app = test_app();

        // Bolt at origin, moving downward, wall attraction active
        app.world_mut().spawn((
            Bolt,
            Position2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::new(0.0, -400.0)),
            ActiveAttractions {
                entries: vec![crate::effect::effects::attraction::AttractionEntry {
                    attraction_type: crate::effect::definition::AttractionType::Wall,
                    force: 10.0,
                    active: true,
                }],
            },
        ));

        // Regular wall to the left
        app.world_mut()
            .spawn((Wall, Position2D(Vec2::new(-490.0, 0.0))));

        // SecondWindWall at bottom — should be filtered out
        app.world_mut()
            .spawn((Wall, SecondWindWall, Position2D(Vec2::new(0.0, -390.0))));

        tick(&mut app);

        let vel = app
            .world_mut()
            .query_filtered::<&Velocity2D, With<Bolt>>()
            .iter(app.world())
            .next()
            .expect("bolt entity should exist");

        assert!(
            vel.0.x < 0.0,
            "bolt should steer toward regular wall at x=-490 (not SecondWindWall), got vx={:.3}",
            vel.0.x
        );
    }

    #[test]
    fn apply_attraction_is_no_op_without_active_attractions() {
        let mut app = test_app();

        // Bolt with no ActiveAttractions component
        app.world_mut().spawn((
            Bolt,
            Position2D(Vec2::new(0.0, 0.0)),
            Velocity2D(Vec2::new(0.0, 400.0)),
        ));

        // Cell nearby
        app.world_mut()
            .spawn((Cell, Position2D(Vec2::new(50.0, 100.0))));

        tick(&mut app);

        let vel = app
            .world_mut()
            .query_filtered::<&Velocity2D, With<Bolt>>()
            .iter(app.world())
            .next()
            .expect("bolt entity should exist");

        assert!(
            (vel.0.x - 0.0).abs() < f32::EPSILON,
            "bolt without ActiveAttractions should have unchanged vx, got {:.3}",
            vel.0.x
        );
        assert!(
            (vel.0.y - 400.0).abs() < f32::EPSILON,
            "bolt without ActiveAttractions should have unchanged vy, got {:.3}",
            vel.0.y
        );
    }
}
