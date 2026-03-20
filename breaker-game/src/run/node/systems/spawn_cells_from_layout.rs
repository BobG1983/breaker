//! System to spawn cells from the active node layout.

use bevy::prelude::*;
use tracing::debug;

use crate::{
    cells::{
        components::*,
        resources::{CellConfig, CellTypeRegistry},
    },
    run::node::{ActiveNodeLayout, NodeLayout, messages::CellsSpawned},
    shared::{CleanupOnNodeExit, PlayfieldConfig},
};

/// Spawns cells from a grid layout. Returns the count of `RequiredToClear` cells.
///
/// Shared between the `OnEnter(Playing)` system and hot-reload respawn.
pub fn spawn_cells_from_grid(
    commands: &mut Commands,
    config: &CellConfig,
    playfield: &PlayfieldConfig,
    layout: &NodeLayout,
    registry: &CellTypeRegistry,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) -> u32 {
    let cell_width = config.width;
    let cell_height = config.height;
    let step_x = cell_width + config.padding_x;
    let step_y = cell_height + config.padding_y;

    let grid_width = step_x.mul_add(
        f32::from(u16::try_from(layout.cols).unwrap_or(u16::MAX)),
        -config.padding_x,
    );
    let start_x = -grid_width / 2.0 + cell_width / 2.0;
    let start_y = playfield.top() - layout.grid_top_offset - cell_height / 2.0;

    let rect_mesh = meshes.add(Rectangle::new(1.0, 1.0));
    let mut required_count = 0u32;

    for (row_idx, row) in layout.grid.iter().enumerate() {
        for (col_idx, &alias) in row.iter().enumerate() {
            if alias == '.' {
                continue;
            }

            let Some(def) = registry.types.get(&alias) else {
                continue;
            };

            let col_f = f32::from(u16::try_from(col_idx).unwrap_or(u16::MAX));
            let row_f = f32::from(u16::try_from(row_idx).unwrap_or(u16::MAX));
            let x = col_f.mul_add(step_x, start_x);
            let y = row_f.mul_add(-step_y, start_y);

            let mut entity = commands.spawn((
                Cell,
                CellTypeAlias(alias),
                CellWidth(config.width),
                CellHeight(config.height),
                CellHealth::new(def.hp),
                CellDamageVisuals {
                    hdr_base: def.damage_hdr_base,
                    green_min: def.damage_green_min,
                    blue_range: def.damage_blue_range,
                    blue_base: def.damage_blue_base,
                },
                Mesh2d(rect_mesh.clone()),
                MeshMaterial2d(materials.add(ColorMaterial::from_color(def.color()))),
                Transform {
                    translation: Vec3::new(x, y, 0.0),
                    scale: Vec3::new(cell_width, cell_height, 1.0),
                    ..default()
                },
                CleanupOnNodeExit,
            ));

            if def.required_to_clear {
                entity.insert(RequiredToClear);
                required_count += 1;
            }
        }
    }
    required_count
}

/// Spawns cells from the active node layout.
///
/// Runs once when entering [`GameState::Playing`], after [`set_active_layout`].
/// Reads the grid from [`ActiveNodeLayout`] and looks up each alias in
/// [`CellTypeRegistry`] to determine cell properties.
pub fn spawn_cells_from_layout(
    mut commands: Commands,
    config: Res<CellConfig>,
    playfield: Res<PlayfieldConfig>,
    layout: Res<ActiveNodeLayout>,
    registry: Res<CellTypeRegistry>,
    // Bundled as a tuple to stay within clippy's 7-argument limit.
    mut render_assets: (ResMut<Assets<Mesh>>, ResMut<Assets<ColorMaterial>>),
    mut cells_spawned: MessageWriter<CellsSpawned>,
) {
    let count = spawn_cells_from_grid(
        &mut commands,
        &config,
        &playfield,
        &layout.0,
        &registry,
        &mut render_assets.0,
        &mut render_assets.1,
    );
    debug!("cells spawned count={}", count);
    cells_spawned.write(CellsSpawned);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cells::{
            CellTypeDefinition,
            components::{Cell, CellHealth, CellHeight, CellTypeAlias, CellWidth, RequiredToClear},
            definition::CellBehavior,
            resources::CellTypeRegistry,
        },
        run::node::{ActiveNodeLayout, NodeLayout, definition::NodePool},
    };

    fn test_registry() -> CellTypeRegistry {
        let mut registry = CellTypeRegistry::default();
        registry.types.insert(
            'S',
            CellTypeDefinition {
                id: "standard".to_owned(),
                alias: 'S',
                hp: 1.0,
                color_rgb: [4.0, 0.2, 0.5],
                required_to_clear: true,
                damage_hdr_base: 4.0,
                damage_green_min: 0.2,
                damage_blue_range: 0.4,
                damage_blue_base: 0.2,
                behavior: CellBehavior::default(),
            },
        );
        registry.types.insert(
            'T',
            CellTypeDefinition {
                id: "tough".to_owned(),
                alias: 'T',
                hp: 3.0,
                color_rgb: [2.5, 0.2, 4.0],
                required_to_clear: true,
                damage_hdr_base: 4.0,
                damage_green_min: 0.2,
                damage_blue_range: 0.4,
                damage_blue_base: 0.2,
                behavior: CellBehavior::default(),
            },
        );
        registry
    }

    /// A full 3x2 layout with no gaps.
    fn full_layout() -> NodeLayout {
        NodeLayout {
            name: "full".to_owned(),
            timer_secs: 60.0,
            cols: 3,
            rows: 2,
            grid_top_offset: 50.0,
            grid: vec![vec!['T', 'S', 'S'], vec!['S', 'S', 'S']],
            pool: NodePool::default(),
        }
    }

    /// A 3x2 layout with gaps (dots).
    fn sparse_layout() -> NodeLayout {
        NodeLayout {
            name: "sparse".to_owned(),
            timer_secs: 60.0,
            cols: 3,
            rows: 2,
            grid_top_offset: 50.0,
            grid: vec![vec!['.', 'S', '.'], vec!['T', '.', 'S']],
            pool: NodePool::default(),
        }
    }

    fn test_app(layout: NodeLayout) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<CellsSpawned>()
            .init_resource::<CellConfig>()
            .init_resource::<PlayfieldConfig>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<ColorMaterial>>()
            .insert_resource(ActiveNodeLayout(layout))
            .insert_resource(test_registry())
            .add_systems(Startup, spawn_cells_from_layout);
        app
    }

    #[test]
    fn correct_cell_count_full_layout() {
        let layout = full_layout();
        let expected = layout.cell_count();
        let mut app = test_app(layout);
        app.update();

        let count = app.world_mut().query::<&Cell>().iter(app.world()).count();
        assert_eq!(count, expected);
        assert_eq!(count, 6);
    }

    #[test]
    fn dot_slots_produce_no_entities() {
        let layout = sparse_layout();
        let total_slots = (layout.cols * layout.rows) as usize;
        let mut app = test_app(layout);
        app.update();

        let count = app.world_mut().query::<&Cell>().iter(app.world()).count();
        assert_eq!(count, 3);
        assert!(count < total_slots, "dots should not spawn cells");
    }

    #[test]
    fn cells_get_hp_from_type_definition() {
        let layout = full_layout();
        let mut app = test_app(layout);
        app.update();

        let mut found_standard = false;
        let mut found_tough = false;
        for health in app.world_mut().query::<&CellHealth>().iter(app.world()) {
            if (health.max - 1.0).abs() < f32::EPSILON {
                found_standard = true;
            }
            if (health.max - 3.0).abs() < f32::EPSILON {
                found_tough = true;
            }
        }
        assert!(found_standard, "should have standard cells (hp=1.0)");
        assert!(found_tough, "should have tough cells (hp=3.0)");
    }

    #[test]
    fn required_to_clear_present_when_true() {
        let layout = full_layout();
        let mut app = test_app(layout);
        app.update();

        let cell_count = app.world_mut().query::<&Cell>().iter(app.world()).count();
        let required_count = app
            .world_mut()
            .query::<(&Cell, &RequiredToClear)>()
            .iter(app.world())
            .count();
        assert_eq!(cell_count, required_count);
    }

    #[test]
    fn spawn_cells_sends_cells_spawned_message() {
        let mut app = test_app(full_layout());
        app.update();

        let messages = app.world().resource::<Messages<CellsSpawned>>();
        assert!(
            messages.iter_current_update_messages().count() > 0,
            "spawn_cells_from_layout must send CellsSpawned message"
        );
    }

    #[test]
    fn all_cells_have_cleanup_marker() {
        let layout = full_layout();
        let mut app = test_app(layout);
        app.update();

        let cell_count = app.world_mut().query::<&Cell>().iter(app.world()).count();
        let marked_count = app
            .world_mut()
            .query::<(&Cell, &CleanupOnNodeExit)>()
            .iter(app.world())
            .count();
        assert_eq!(cell_count, marked_count);
    }

    #[test]
    fn all_cells_within_playfield() {
        let layout = full_layout();
        let config = CellConfig::default();
        let playfield = PlayfieldConfig::default();
        let mut app = test_app(layout);
        app.update();

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
    fn all_cells_have_dimensions_and_damage_visuals() {
        let layout = full_layout();
        let mut app = test_app(layout);
        app.update();

        let cell_count = app.world_mut().query::<&Cell>().iter(app.world()).count();
        let with_dims = app
            .world_mut()
            .query::<(&Cell, &CellWidth, &CellHeight, &CellDamageVisuals)>()
            .iter(app.world())
            .count();
        assert_eq!(cell_count, with_dims);
    }

    #[test]
    fn unrecognized_alias_produces_no_entity() {
        let layout = NodeLayout {
            name: "unknown".to_owned(),
            timer_secs: 60.0,
            cols: 3,
            rows: 1,
            grid_top_offset: 50.0,
            grid: vec![vec!['S', 'X', 'S']], // 'X' not in registry
            pool: NodePool::default(),
        };
        let mut app = test_app(layout);
        app.update();

        let count = app.world_mut().query::<&Cell>().iter(app.world()).count();
        assert_eq!(
            count, 2,
            "unrecognized alias 'X' should be silently skipped, only 2 cells spawned"
        );
    }

    // --- Cell position tests ---

    #[test]
    fn grid_is_horizontally_centered() {
        let layout = full_layout();
        let config = CellConfig::default();
        let step_x = config.width + config.padding_x;
        let mut app = test_app(layout.clone());
        app.update();

        // Grid should be centered: sum of all x positions per row should be ~0
        // With 3 columns the positions should be symmetric around 0
        let cols_f = f32::from(u16::try_from(layout.cols).unwrap_or(u16::MAX));
        let grid_width = step_x.mul_add(cols_f, -config.padding_x);
        let expected_start = -grid_width / 2.0 + config.width / 2.0;
        let expected_end = step_x.mul_add(cols_f - 1.0, expected_start);
        let center = f32::midpoint(expected_start, expected_end);

        assert!(
            center.abs() < 1.0,
            "grid center should be near 0, got {center:.2}"
        );
    }

    #[test]
    fn cell_positions_match_grid_coordinates() {
        let layout = full_layout();
        let config = CellConfig::default();
        let playfield = PlayfieldConfig::default();
        let step_x = config.width + config.padding_x;
        let step_y = config.height + config.padding_y;
        let mut app = test_app(layout.clone());
        app.update();

        let grid_width = step_x.mul_add(
            f32::from(u16::try_from(layout.cols).unwrap_or(u16::MAX)),
            -config.padding_x,
        );
        let start_x = -grid_width / 2.0 + config.width / 2.0;
        let start_y = playfield.top() - layout.grid_top_offset - config.height / 2.0;

        let mut positions: Vec<(f32, f32)> = app
            .world_mut()
            .query_filtered::<&Transform, With<Cell>>()
            .iter(app.world())
            .map(|tf| (tf.translation.x, tf.translation.y))
            .collect();
        positions.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.total_cmp(&b.0)));

        // full_layout: row 0 = [T, S, S], row 1 = [S, S, S]
        let expected: Vec<(f32, f32)> = vec![
            // Row 0
            (start_x, start_y),
            (start_x + step_x, start_y),
            (step_x.mul_add(2.0, start_x), start_y),
            // Row 1
            (start_x, start_y - step_y),
            (start_x + step_x, start_y - step_y),
            (step_x.mul_add(2.0, start_x), start_y - step_y),
        ];

        assert_eq!(positions.len(), expected.len());
        for (i, ((ax, ay), (ex, ey))) in positions.iter().zip(expected.iter()).enumerate() {
            assert!(
                (ax - ex).abs() < 0.01,
                "cell {i} x: expected {ex:.2}, got {ax:.2}"
            );
            assert!(
                (ay - ey).abs() < 0.01,
                "cell {i} y: expected {ey:.2}, got {ay:.2}"
            );
        }
    }

    #[test]
    fn sparse_layout_positions_skip_dots() {
        let layout = sparse_layout();
        let config = CellConfig::default();
        let playfield = PlayfieldConfig::default();
        let step_x = config.width + config.padding_x;
        let step_y = config.height + config.padding_y;
        let mut app = test_app(layout.clone());
        app.update();

        let grid_width = step_x.mul_add(
            f32::from(u16::try_from(layout.cols).unwrap_or(u16::MAX)),
            -config.padding_x,
        );
        let start_x = -grid_width / 2.0 + config.width / 2.0;
        let start_y = playfield.top() - layout.grid_top_offset - config.height / 2.0;

        let mut positions: Vec<(f32, f32)> = app
            .world_mut()
            .query_filtered::<&Transform, With<Cell>>()
            .iter(app.world())
            .map(|tf| (tf.translation.x, tf.translation.y))
            .collect();
        positions.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.total_cmp(&b.0)));

        // sparse_layout: row 0 = [., S, .], row 1 = [T, ., S]
        let expected: Vec<(f32, f32)> = vec![
            (start_x + step_x, start_y),                      // row 0, col 1
            (start_x, start_y - step_y),                      // row 1, col 0
            (step_x.mul_add(2.0, start_x), start_y - step_y), // row 1, col 2
        ];

        assert_eq!(positions.len(), expected.len());
        for (i, ((ax, ay), (ex, ey))) in positions.iter().zip(expected.iter()).enumerate() {
            assert!(
                (ax - ex).abs() < 0.01,
                "cell {i} x: expected {ex:.2}, got {ax:.2}"
            );
            assert!(
                (ay - ey).abs() < 0.01,
                "cell {i} y: expected {ey:.2}, got {ay:.2}"
            );
        }
    }

    #[test]
    fn all_cells_have_cell_type_alias() {
        let layout = full_layout();
        let mut app = test_app(layout);
        app.update();

        let cell_count = app.world_mut().query::<&Cell>().iter(app.world()).count();
        let alias_count = app
            .world_mut()
            .query::<(&Cell, &CellTypeAlias)>()
            .iter(app.world())
            .count();
        assert_eq!(
            cell_count, alias_count,
            "every cell should have a CellTypeAlias"
        );
    }

    #[test]
    fn cell_type_alias_matches_grid_char() {
        // full_layout: row 0 = [T, S, S], row 1 = [S, S, S] → 1 T, 5 S
        let layout = full_layout();
        let mut app = test_app(layout);
        app.update();

        let mut t_count = 0;
        let mut s_count = 0;
        for alias in app.world_mut().query::<&CellTypeAlias>().iter(app.world()) {
            match alias.0 {
                'T' => t_count += 1,
                'S' => s_count += 1,
                other => panic!("unexpected alias '{other}'"),
            }
        }
        assert_eq!(t_count, 1, "should have 1 tough cell");
        assert_eq!(s_count, 5, "should have 5 standard cells");
    }

    #[test]
    fn cell_spacing_matches_config() {
        let layout = full_layout();
        let config = CellConfig::default();
        let step_x = config.width + config.padding_x;
        let step_y = config.height + config.padding_y;
        let mut app = test_app(layout);
        app.update();

        let mut positions: Vec<(f32, f32)> = app
            .world_mut()
            .query_filtered::<&Transform, With<Cell>>()
            .iter(app.world())
            .map(|tf| (tf.translation.x, tf.translation.y))
            .collect();
        positions.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.total_cmp(&b.0)));

        // Check horizontal spacing within row 0 (first 3 cells)
        let dx_01 = positions[1].0 - positions[0].0;
        assert!(
            (dx_01 - step_x).abs() < 0.01,
            "horizontal spacing should be {step_x}, got {dx_01}"
        );
        let dx_12 = positions[2].0 - positions[1].0;
        assert!(
            (dx_12 - step_x).abs() < 0.01,
            "horizontal spacing should be {step_x}, got {dx_12}"
        );

        // Check vertical spacing between row 0 and row 1 (same column)
        let dy = positions[0].1 - positions[3].1;
        assert!(
            (dy - step_y).abs() < 0.01,
            "vertical spacing should be {step_y}, got {dy}"
        );
    }
}
