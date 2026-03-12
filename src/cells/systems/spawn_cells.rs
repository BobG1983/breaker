//! System to spawn a grid of cell entities.

use bevy::prelude::*;

use crate::{
    cells::{
        components::{Cell, CellDamageVisuals, CellHealth, CellHeight, CellWidth},
        resources::CellConfig,
    },
    shared::{CleanupOnNodeExit, PlayfieldConfig},
};

/// Spawns a grid of cells at the top of the playfield.
///
/// Runs once when entering [`GameState::Playing`].
pub fn spawn_cells(
    mut commands: Commands,
    config: Res<CellConfig>,
    playfield: Res<PlayfieldConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let cell_width = config.width;
    let cell_height = config.height;
    let step_x = cell_width + config.padding_x;
    let step_y = cell_height + config.padding_y;

    // Center the grid horizontally
    #[allow(clippy::cast_precision_loss)]
    let grid_width = step_x.mul_add(config.grid_cols as f32, -config.padding_x);
    let start_x = -grid_width / 2.0 + config.width / 2.0;
    let start_y = playfield.top() - config.grid_top_offset - config.height / 2.0;

    let rect_mesh = meshes.add(Rectangle::new(1.0, 1.0));

    for row in 0..config.grid_rows {
        let is_tough = row == config.tough_row_index;
        let hp = if is_tough {
            config.tough_hp
        } else {
            config.standard_hp
        };
        let color = if is_tough {
            config.tough_color()
        } else {
            config.standard_color()
        };

        for col in 0..config.grid_cols {
            #[allow(clippy::cast_precision_loss)]
            let x = (col as f32).mul_add(step_x, start_x);
            #[allow(clippy::cast_precision_loss)]
            let y = (row as f32).mul_add(-step_y, start_y);

            commands.spawn((
                Cell,
                CellWidth(config.width),
                CellHeight(config.height),
                CellHealth::new(hp),
                CellDamageVisuals {
                    hdr_base: config.damage_hdr_base,
                    green_min: config.damage_green_min,
                    blue_range: config.damage_blue_range,
                    blue_base: config.damage_blue_base,
                },
                Mesh2d(rect_mesh.clone()),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(color))),
                Transform {
                    translation: Vec3::new(x, y, 0.0),
                    scale: Vec3::new(cell_width, cell_height, 1.0),
                    ..default()
                },
                CleanupOnNodeExit,
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cells::components::{Cell, CellHealth, CellHeight, CellWidth};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<CellConfig>();
        app.init_resource::<PlayfieldConfig>();
        app.init_resource::<Assets<Mesh>>();
        app.init_resource::<Assets<ColorMaterial>>();
        app.add_systems(Startup, spawn_cells);
        app
    }

    #[test]
    fn spawn_cells_creates_correct_count() {
        let mut app = test_app();
        app.update();

        let config = CellConfig::default();
        let expected = config.grid_cols * config.grid_rows;
        let count = app.world_mut().query::<&Cell>().iter(app.world()).count();
        #[allow(clippy::cast_possible_truncation)]
        let count_u32 = count as u32;
        assert_eq!(count_u32, expected);
    }

    #[test]
    fn tough_row_has_higher_hp() {
        let mut app = test_app();
        app.update();

        let config = CellConfig::default();
        let mut found_tough = false;
        let mut found_standard = false;

        for health in app.world_mut().query::<&CellHealth>().iter(app.world()) {
            if health.max == config.tough_hp {
                found_tough = true;
            }
            if health.max == config.standard_hp {
                found_standard = true;
            }
        }

        assert!(found_tough, "should have tough cells");
        assert!(found_standard, "should have standard cells");
    }

    #[test]
    fn all_cells_within_playfield() {
        let mut app = test_app();
        app.update();

        let config = CellConfig::default();
        let playfield = PlayfieldConfig::default();
        for transform in app
            .world_mut()
            .query_filtered::<&Transform, With<Cell>>()
            .iter(app.world())
        {
            let x = transform.translation.x;
            let y = transform.translation.y;
            assert!(
                x.abs() < playfield.right() + config.width / 2.0,
                "cell x={x} out of bounds"
            );
            assert!(
                y < playfield.top() + config.height / 2.0,
                "cell y={y} above playfield"
            );
        }
    }

    #[test]
    fn all_cells_have_dimensions() {
        let mut app = test_app();
        app.update();

        let config = CellConfig::default();
        for (cell_w, cell_h) in app
            .world_mut()
            .query::<(&CellWidth, &CellHeight)>()
            .iter(app.world())
        {
            assert!(
                (cell_w.0 - config.width).abs() < f32::EPSILON,
                "CellWidth should match config"
            );
            assert!(
                (cell_h.0 - config.height).abs() < f32::EPSILON,
                "CellHeight should match config"
            );
        }
    }

    #[test]
    fn all_cells_have_cleanup_marker() {
        let mut app = test_app();
        app.update();

        let cell_count = app.world_mut().query::<&Cell>().iter(app.world()).count();
        let marked_count = app
            .world_mut()
            .query::<(&Cell, &CleanupOnNodeExit)>()
            .iter(app.world())
            .count();
        assert_eq!(cell_count, marked_count);
    }
}
