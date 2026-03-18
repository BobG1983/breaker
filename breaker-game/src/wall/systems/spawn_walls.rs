//! Wall entity spawning — creates invisible boundary entities for CCD collision.

use bevy::prelude::*;

use crate::{
    shared::{CleanupOnNodeExit, PlayfieldConfig},
    wall::components::{Wall, WallSize},
};

/// Spawns left, right, and ceiling wall entities.
///
/// No floor wall — bolt-lost handles that case separately.
/// Wall thickness is loaded from [`PlayfieldConfig::wall_half_thickness`].
pub(crate) fn spawn_walls(mut commands: Commands, playfield: Res<PlayfieldConfig>) {
    let half_width = playfield.width / 2.0;
    let half_height = playfield.height / 2.0;
    let wall_ht = playfield.wall_half_thickness();

    // Left wall
    commands.spawn((
        Wall,
        WallSize {
            half_width: wall_ht,
            half_height,
        },
        Transform::from_xyz(playfield.left() - wall_ht, 0.0, 0.0),
        CleanupOnNodeExit,
    ));

    // Right wall
    commands.spawn((
        Wall,
        WallSize {
            half_width: wall_ht,
            half_height,
        },
        Transform::from_xyz(playfield.right() + wall_ht, 0.0, 0.0),
        CleanupOnNodeExit,
    ));

    // Ceiling
    commands.spawn((
        Wall,
        WallSize {
            half_width,
            half_height: wall_ht,
        },
        Transform::from_xyz(0.0, playfield.top() + wall_ht, 0.0),
        CleanupOnNodeExit,
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wall::components::{Wall, WallSize};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<PlayfieldConfig>();
        app.add_systems(Update, spawn_walls);
        app
    }

    #[test]
    fn spawns_three_walls() {
        let mut app = test_app();
        app.update();

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<Wall>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 3, "should spawn left, right, and ceiling walls");
    }

    #[test]
    fn walls_have_wall_size() {
        let mut app = test_app();
        app.update();

        let sizes: Vec<_> = app
            .world_mut()
            .query::<&WallSize>()
            .iter(app.world())
            .collect();
        assert_eq!(sizes.len(), 3);
        for size in sizes {
            assert!(size.half_width > 0.0);
            assert!(size.half_height > 0.0);
        }
    }

    #[test]
    fn walls_have_cleanup_marker() {
        let mut app = test_app();
        app.update();

        let count = app
            .world_mut()
            .query_filtered::<Entity, (With<Wall>, With<CleanupOnNodeExit>)>()
            .iter(app.world())
            .count();
        assert_eq!(count, 3, "all walls should have CleanupOnNodeExit");
    }

    #[test]
    fn wall_positions_match_playfield() {
        let mut app = test_app();
        app.update();
        let playfield = PlayfieldConfig::default();

        let walls: Vec<_> = app
            .world_mut()
            .query::<(&Transform, &WallSize)>()
            .iter(app.world())
            .map(|(tf, ws)| (tf.translation, ws.half_width, ws.half_height))
            .collect();

        // Left wall: x < playfield left
        let left = walls
            .iter()
            .find(|(pos, ..)| pos.x < playfield.left())
            .expect("should have left wall");
        assert!((left.0.y).abs() < f32::EPSILON, "left wall centered at y=0");

        // Right wall: x > playfield right
        let right = walls
            .iter()
            .find(|(pos, ..)| pos.x > playfield.right())
            .expect("should have right wall");
        assert!(
            (right.0.y).abs() < f32::EPSILON,
            "right wall centered at y=0"
        );

        // Ceiling: y > playfield top
        let ceiling = walls
            .iter()
            .find(|(pos, ..)| pos.y > playfield.top())
            .expect("should have ceiling wall");
        assert!(
            (ceiling.0.x).abs() < f32::EPSILON,
            "ceiling centered at x=0"
        );
    }
}
