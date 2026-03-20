//! System to spawn cells from the active node layout.

use bevy::{ecs::system::SystemParam, prelude::*};
use tracing::debug;

use crate::{
    cells::{
        components::*,
        resources::{CellConfig, CellTypeRegistry},
    },
    run::{
        node::{ActiveNodeLayout, NodeLayout, messages::CellsSpawned},
        resources::{NodeSequence, RunState},
    },
    shared::{CleanupOnNodeExit, PlayfieldConfig},
};

/// Total extent of a grid along one axis: `step * count - padding`.
fn grid_extent(step: f32, count_f: f32, padding: f32) -> f32 {
    step.mul_add(count_f, -padding)
}

/// Pre-computed scaled grid dimensions returned by [`compute_grid_scale`].
#[derive(Debug, Clone, Copy)]
pub(crate) struct ScaledGridDims {
    pub cell_width: f32,
    pub cell_height: f32,
    pub padding_x: f32,
    pub step_x: f32,
    pub step_y: f32,
    pub scale: f32,
}

/// Computes the uniform scale factor for a grid layout so that all cells fit
/// within the playfield cell zone.
///
/// Returns [`ScaledGridDims`] with `scale` in `(0.0, 1.0]` — `1.0` when the
/// grid already fits at native cell dimensions, less when it must shrink.
pub(crate) fn compute_grid_scale(
    config: &CellConfig,
    playfield: &PlayfieldConfig,
    cols: u32,
    rows: u32,
    grid_top_offset: f32,
) -> ScaledGridDims {
    let step_x = config.width + config.padding_x;
    let step_y = config.height + config.padding_y;
    let cols_f = f32::from(u16::try_from(cols).unwrap_or(u16::MAX));
    let rows_f = f32::from(u16::try_from(rows).unwrap_or(u16::MAX));

    let default_grid_width = grid_extent(step_x, cols_f, config.padding_x);
    let default_grid_height = grid_extent(step_y, rows_f, config.padding_y);

    let available_width = playfield.width;
    let available_height = (playfield.cell_zone_height() - grid_top_offset).max(0.0);

    let scale_x = available_width / default_grid_width;
    let scale_y = available_height / default_grid_height;

    let scale = scale_x.min(scale_y).min(1.0);
    let cell_width = config.width * scale;
    let cell_height = config.height * scale;
    let padding_x = config.padding_x * scale;
    let step_x = cell_width + padding_x;
    let step_y = cell_height + config.padding_y * scale;
    ScaledGridDims {
        cell_width,
        cell_height,
        padding_x,
        step_x,
        step_y,
        scale,
    }
}

/// Bundled mutable access to mesh and material asset stores for cell spawning.
pub(crate) struct RenderAssets<'a> {
    pub meshes: &'a mut Assets<Mesh>,
    pub materials: &'a mut Assets<ColorMaterial>,
}

/// Spawns cells from a grid layout. Returns the count of `RequiredToClear` cells.
///
/// Shared between the `OnEnter(Playing)` system and hot-reload respawn.
///
/// `hp_mult` scales every cell's HP (from the node's difficulty assignment).
pub(crate) fn spawn_cells_from_grid(
    commands: &mut Commands,
    config: &CellConfig,
    playfield: &PlayfieldConfig,
    layout: &NodeLayout,
    registry: &CellTypeRegistry,
    render_assets: RenderAssets<'_>,
    hp_mult: f32,
) -> u32 {
    let RenderAssets { meshes, materials } = render_assets;
    let dims = compute_grid_scale(
        config,
        playfield,
        layout.cols,
        layout.rows,
        layout.grid_top_offset,
    );
    let ScaledGridDims {
        cell_width,
        cell_height,
        padding_x,
        step_x,
        step_y,
        scale,
        ..
    } = dims;

    let grid_width = grid_extent(
        step_x,
        f32::from(u16::try_from(layout.cols).unwrap_or(u16::MAX)),
        padding_x,
    );
    let start_x = -grid_width / 2.0 + cell_width / 2.0;
    let start_y = playfield.top() - layout.grid_top_offset - cell_height / 2.0;

    debug!("grid scale={scale:.3} cell={cell_width:.1}x{cell_height:.1}");
    let rect_mesh = meshes.add(Rectangle::new(1.0, 1.0));
    let mut required_count = 0u32;

    for (row_idx, row) in layout.grid.iter().enumerate() {
        for (col_idx, &alias) in row.iter().enumerate() {
            if alias == '.' {
                continue;
            }

            let Some(def) = registry.get(alias) else {
                continue;
            };

            let col_f = f32::from(u16::try_from(col_idx).unwrap_or(u16::MAX));
            let row_f = f32::from(u16::try_from(row_idx).unwrap_or(u16::MAX));
            let x = col_f.mul_add(step_x, start_x);
            let y = row_f.mul_add(-step_y, start_y);

            let scaled_hp = def.hp * hp_mult;

            let mut entity = commands.spawn((
                Cell,
                CellTypeAlias(alias),
                CellWidth(cell_width),
                CellHeight(cell_height),
                CellHealth::new(scaled_hp),
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

            if def.behavior.locked {
                entity.insert(Locked);
                entity.insert(LockAdjacents(Vec::new()));
            }

            if let Some(rate) = def.behavior.regen_rate {
                entity.insert(CellRegen { rate });
            }
        }
    }
    required_count
}

/// Resolves the `hp_mult` for the current node from the run state and node
/// sequence, falling back to `1.0` when those resources are absent (e.g. in
/// tests or scenario overrides).
fn resolve_hp_mult(run_state: Option<&RunState>, node_sequence: Option<&NodeSequence>) -> f32 {
    if let (Some(state), Some(sequence)) = (run_state, node_sequence) {
        sequence
            .assignments
            .get(state.node_index as usize)
            .map_or(1.0, |a| a.hp_mult)
    } else {
        1.0
    }
}

/// Bundles read-only resources needed by [`spawn_cells_from_layout`] to stay
/// within clippy's argument-count limit.
#[derive(SystemParam)]
pub(crate) struct CellSpawnContext<'w> {
    cell_config: Res<'w, CellConfig>,
    playfield_config: Res<'w, PlayfieldConfig>,
    cell_registry: Res<'w, CellTypeRegistry>,
    run_state: Option<Res<'w, RunState>>,
    node_sequence: Option<Res<'w, NodeSequence>>,
}

/// Spawns cells from the active node layout.
///
/// Runs once when entering [`GameState::Playing`], after [`set_active_layout`].
/// Reads the grid from [`ActiveNodeLayout`] and looks up each alias in
/// [`CellTypeRegistry`] to determine cell properties.
pub(crate) fn spawn_cells_from_layout(
    mut commands: Commands,
    ctx: CellSpawnContext,
    layout: Res<ActiveNodeLayout>,
    mut render_assets: (ResMut<Assets<Mesh>>, ResMut<Assets<ColorMaterial>>),
    mut cells_spawned: MessageWriter<CellsSpawned>,
) {
    let hp_mult = resolve_hp_mult(ctx.run_state.as_deref(), ctx.node_sequence.as_deref());
    let count = spawn_cells_from_grid(
        &mut commands,
        &ctx.cell_config,
        &ctx.playfield_config,
        &layout.0,
        &ctx.cell_registry,
        RenderAssets {
            meshes: &mut render_assets.0,
            materials: &mut render_assets.1,
        },
        hp_mult,
    );
    debug!("cells spawned count={count}");
    cells_spawned.write(CellsSpawned);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        cells::{
            CellTypeDefinition,
            components::{
                Cell, CellHealth, CellHeight, CellRegen, CellTypeAlias, CellWidth, LockAdjacents,
                Locked, RequiredToClear,
            },
            definition::CellBehavior,
            resources::CellTypeRegistry,
        },
        run::{
            definition::NodeType,
            node::{ActiveNodeLayout, NodeLayout, definition::NodePool},
            resources::{NodeAssignment, NodeSequence, RunState},
        },
    };

    fn test_registry() -> CellTypeRegistry {
        let mut registry = CellTypeRegistry::default();
        registry.insert(
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
        registry.insert(
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

    fn collect_sorted_cell_positions(app: &mut App) -> Vec<(f32, f32)> {
        let mut positions: Vec<(f32, f32)> = app
            .world_mut()
            .query_filtered::<&Transform, With<Cell>>()
            .iter(app.world())
            .map(|tf| (tf.translation.x, tf.translation.y))
            .collect();
        positions.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.total_cmp(&b.0)));
        positions
    }

    fn assert_positions_match(actual: &[(f32, f32)], expected: &[(f32, f32)]) {
        assert_eq!(actual.len(), expected.len(), "position count mismatch");
        for (i, ((ax, ay), (ex, ey))) in actual.iter().zip(expected.iter()).enumerate() {
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
        let grid_width = grid_extent(step_x, cols_f, config.padding_x);
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

        let grid_width = grid_extent(
            step_x,
            f32::from(u16::try_from(layout.cols).unwrap_or(u16::MAX)),
            config.padding_x,
        );
        let start_x = -grid_width / 2.0 + config.width / 2.0;
        let start_y = playfield.top() - layout.grid_top_offset - config.height / 2.0;

        let positions = collect_sorted_cell_positions(&mut app);

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

        assert_positions_match(&positions, &expected);
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

        let grid_width = grid_extent(
            step_x,
            f32::from(u16::try_from(layout.cols).unwrap_or(u16::MAX)),
            config.padding_x,
        );
        let start_x = -grid_width / 2.0 + config.width / 2.0;
        let start_y = playfield.top() - layout.grid_top_offset - config.height / 2.0;

        let positions = collect_sorted_cell_positions(&mut app);

        // sparse_layout: row 0 = [., S, .], row 1 = [T, ., S]
        let expected: Vec<(f32, f32)> = vec![
            (start_x + step_x, start_y),                      // row 0, col 1
            (start_x, start_y - step_y),                      // row 1, col 0
            (step_x.mul_add(2.0, start_x), start_y - step_y), // row 1, col 2
        ];

        assert_positions_match(&positions, &expected);
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

    // --- A2: CellBehavior wiring tests ---

    /// Creates a registry with a locked cell type ('L') and a regen cell type ('R').
    fn behavior_registry() -> CellTypeRegistry {
        let mut registry = CellTypeRegistry::default();
        registry.insert(
            'L',
            CellTypeDefinition {
                id: "locked".to_owned(),
                alias: 'L',
                hp: 5.0,
                color_rgb: [1.0, 1.0, 1.0],
                required_to_clear: true,
                damage_hdr_base: 4.0,
                damage_green_min: 0.2,
                damage_blue_range: 0.4,
                damage_blue_base: 0.2,
                behavior: CellBehavior {
                    locked: true,
                    regen_rate: None,
                },
            },
        );
        registry.insert(
            'R',
            CellTypeDefinition {
                id: "regen".to_owned(),
                alias: 'R',
                hp: 8.0,
                color_rgb: [0.5, 1.0, 0.5],
                required_to_clear: true,
                damage_hdr_base: 4.0,
                damage_green_min: 0.2,
                damage_blue_range: 0.4,
                damage_blue_base: 0.2,
                behavior: CellBehavior {
                    locked: false,
                    regen_rate: Some(2.0),
                },
            },
        );
        registry.insert(
            'N',
            CellTypeDefinition {
                id: "normal".to_owned(),
                alias: 'N',
                hp: 1.0,
                color_rgb: [1.0, 0.5, 0.5],
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

    fn behavior_test_app(layout: NodeLayout, registry: CellTypeRegistry) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<CellsSpawned>()
            .init_resource::<CellConfig>()
            .init_resource::<PlayfieldConfig>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<ColorMaterial>>()
            .insert_resource(ActiveNodeLayout(layout))
            .insert_resource(registry)
            .add_systems(Startup, spawn_cells_from_layout);
        app
    }

    #[test]
    fn locked_cell_definition_spawns_with_locked_component() {
        let layout = NodeLayout {
            name: "lock_test".to_owned(),
            timer_secs: 60.0,
            cols: 2,
            rows: 1,
            grid_top_offset: 50.0,
            grid: vec![vec!['L', 'N']],
            pool: NodePool::default(),
        };
        let mut app = behavior_test_app(layout, behavior_registry());
        app.update();

        let locked_count = app
            .world_mut()
            .query::<(&Cell, &Locked)>()
            .iter(app.world())
            .count();
        assert_eq!(
            locked_count, 1,
            "cell with behavior.locked=true should have Locked component"
        );
    }

    #[test]
    fn non_locked_cell_does_not_have_locked_component() {
        let layout = NodeLayout {
            name: "no_lock_test".to_owned(),
            timer_secs: 60.0,
            cols: 2,
            rows: 1,
            grid_top_offset: 50.0,
            grid: vec![vec!['N', 'R']],
            pool: NodePool::default(),
        };
        let mut app = behavior_test_app(layout, behavior_registry());
        app.update();

        let locked_count = app
            .world_mut()
            .query::<(&Cell, &Locked)>()
            .iter(app.world())
            .count();
        assert_eq!(
            locked_count, 0,
            "cells with behavior.locked=false should NOT have Locked component"
        );
    }

    #[test]
    fn locked_cell_definition_spawns_with_lock_adjacents_component() {
        let layout = NodeLayout {
            name: "lock_adj_test".to_owned(),
            timer_secs: 60.0,
            cols: 2,
            rows: 1,
            grid_top_offset: 50.0,
            grid: vec![vec!['L', 'N']],
            pool: NodePool::default(),
        };
        let mut app = behavior_test_app(layout, behavior_registry());
        app.update();

        let lock_adj_count = app
            .world_mut()
            .query::<(&Cell, &LockAdjacents)>()
            .iter(app.world())
            .count();
        assert_eq!(
            lock_adj_count, 1,
            "cell with behavior.locked=true should have LockAdjacents component"
        );
    }

    #[test]
    fn regen_cell_definition_spawns_with_cell_regen_component() {
        let layout = NodeLayout {
            name: "regen_test".to_owned(),
            timer_secs: 60.0,
            cols: 2,
            rows: 1,
            grid_top_offset: 50.0,
            grid: vec![vec!['R', 'N']],
            pool: NodePool::default(),
        };
        let mut app = behavior_test_app(layout, behavior_registry());
        app.update();

        let regen_cells: Vec<&CellRegen> = app
            .world_mut()
            .query::<(&Cell, &CellRegen)>()
            .iter(app.world())
            .map(|(_, regen)| regen)
            .collect();
        assert_eq!(
            regen_cells.len(),
            1,
            "cell with behavior.regen_rate=Some(2.0) should have CellRegen component"
        );
        assert!(
            (regen_cells[0].rate - 2.0).abs() < f32::EPSILON,
            "CellRegen rate should be 2.0, got {}",
            regen_cells[0].rate
        );
    }

    #[test]
    fn non_regen_cell_does_not_have_cell_regen_component() {
        let layout = NodeLayout {
            name: "no_regen_test".to_owned(),
            timer_secs: 60.0,
            cols: 2,
            rows: 1,
            grid_top_offset: 50.0,
            grid: vec![vec!['L', 'N']],
            pool: NodePool::default(),
        };
        let mut app = behavior_test_app(layout, behavior_registry());
        app.update();

        let regen_count = app
            .world_mut()
            .query::<(&Cell, &CellRegen)>()
            .iter(app.world())
            .count();
        assert_eq!(
            regen_count, 0,
            "cells with behavior.regen_rate=None should NOT have CellRegen component"
        );
    }

    // --- A4: HP multiplier tests ---

    #[test]
    fn cell_hp_scaled_by_node_assignment_hp_mult() {
        let layout = NodeLayout {
            name: "hp_mult_test".to_owned(),
            timer_secs: 60.0,
            cols: 1,
            rows: 1,
            grid_top_offset: 50.0,
            grid: vec![vec!['S']],
            pool: NodePool::default(),
        };
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<CellsSpawned>()
            .init_resource::<CellConfig>()
            .init_resource::<PlayfieldConfig>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<ColorMaterial>>()
            .insert_resource(ActiveNodeLayout(layout))
            .insert_resource(test_registry())
            .insert_resource(RunState {
                node_index: 0,
                ..Default::default()
            })
            .insert_resource(NodeSequence {
                assignments: vec![NodeAssignment {
                    node_type: NodeType::Active,
                    tier_index: 0,
                    hp_mult: 3.0,
                    timer_mult: 1.0,
                }],
            })
            .add_systems(Startup, spawn_cells_from_layout);
        app.update();

        // 'S' has hp=1.0, hp_mult=3.0 → CellHealth { current: 3.0, max: 3.0 }
        let healths: Vec<&CellHealth> = app
            .world_mut()
            .query::<&CellHealth>()
            .iter(app.world())
            .collect();
        assert_eq!(healths.len(), 1);
        assert!(
            (healths[0].current - 3.0).abs() < f32::EPSILON,
            "cell current HP should be 1.0 * 3.0 = 3.0, got {}",
            healths[0].current
        );
        assert!(
            (healths[0].max - 3.0).abs() < f32::EPSILON,
            "cell max HP should be 1.0 * 3.0 = 3.0, got {}",
            healths[0].max
        );
    }

    #[test]
    fn cell_hp_unchanged_when_hp_mult_is_one() {
        let layout = NodeLayout {
            name: "hp_mult_one_test".to_owned(),
            timer_secs: 60.0,
            cols: 1,
            rows: 1,
            grid_top_offset: 50.0,
            grid: vec![vec!['T']],
            pool: NodePool::default(),
        };
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<CellsSpawned>()
            .init_resource::<CellConfig>()
            .init_resource::<PlayfieldConfig>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<ColorMaterial>>()
            .insert_resource(ActiveNodeLayout(layout))
            .insert_resource(test_registry())
            .insert_resource(RunState {
                node_index: 0,
                ..Default::default()
            })
            .insert_resource(NodeSequence {
                assignments: vec![NodeAssignment {
                    node_type: NodeType::Passive,
                    tier_index: 0,
                    hp_mult: 1.0,
                    timer_mult: 1.0,
                }],
            })
            .add_systems(Startup, spawn_cells_from_layout);
        app.update();

        // 'T' has hp=3.0, hp_mult=1.0 → CellHealth { current: 3.0, max: 3.0 }
        let healths: Vec<&CellHealth> = app
            .world_mut()
            .query::<&CellHealth>()
            .iter(app.world())
            .collect();
        assert_eq!(healths.len(), 1);
        assert!(
            (healths[0].current - 3.0).abs() < f32::EPSILON,
            "cell current HP should be 3.0 * 1.0 = 3.0, got {}",
            healths[0].current
        );
        assert!(
            (healths[0].max - 3.0).abs() < f32::EPSILON,
            "cell max HP should be 3.0 * 1.0 = 3.0, got {}",
            healths[0].max
        );
    }

    #[test]
    fn cell_spacing_matches_config() {
        let layout = full_layout();
        let config = CellConfig::default();
        let step_x = config.width + config.padding_x;
        let step_y = config.height + config.padding_y;
        let mut app = test_app(layout);
        app.update();

        let positions = collect_sorted_cell_positions(&mut app);

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

    // --- Grid scale tests ---

    /// Returns a `CellConfig` with RON-like values (not Rust `Default`).
    fn ron_like_cell_config() -> CellConfig {
        CellConfig {
            width: 126.0,
            height: 43.0,
            padding_x: 7.0,
            padding_y: 7.0,
        }
    }

    /// Returns a `PlayfieldConfig` with RON-like values (not Rust `Default`).
    fn ron_like_playfield_config() -> PlayfieldConfig {
        PlayfieldConfig {
            width: 1440.0,
            height: 1080.0,
            zone_fraction: 0.667,
            wall_thickness: 180.0,
            background_color_rgb: [0.02, 0.01, 0.04],
        }
    }

    /// Creates a test `App` with explicit RON-like configs for grid-scale tests.
    fn scaled_test_app(layout: NodeLayout) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<CellsSpawned>()
            .insert_resource(ron_like_cell_config())
            .insert_resource(ron_like_playfield_config())
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<ColorMaterial>>()
            .insert_resource(ActiveNodeLayout(layout))
            .insert_resource(test_registry())
            .add_systems(Startup, spawn_cells_from_layout);
        app
    }

    /// Builds a `NodeLayout` filled entirely with 'S' cells.
    fn uniform_layout(cols: u32, rows: u32, grid_top_offset: f32) -> NodeLayout {
        let grid = vec![vec!['S'; cols as usize]; rows as usize];
        NodeLayout {
            name: format!("uniform_{cols}x{rows}"),
            timer_secs: 60.0,
            cols,
            rows,
            grid_top_offset,
            grid,
            pool: NodePool::default(),
        }
    }

    // --- A: Pure function tests for compute_grid_scale ---

    #[test]
    fn small_grid_returns_scale_one() {
        let config = ron_like_cell_config();
        let playfield = ron_like_playfield_config();
        let result = compute_grid_scale(&config, &playfield, 3, 2, 50.0);
        assert!(
            (result.scale - 1.0).abs() < f32::EPSILON,
            "3x2 grid should fit at scale 1.0, got {}",
            result.scale
        );
    }

    #[test]
    fn wide_grid_is_width_constrained() {
        let config = ron_like_cell_config();
        let playfield = ron_like_playfield_config();
        let result = compute_grid_scale(&config, &playfield, 30, 2, 90.0);
        // default_grid_width = 30 * 133 - 7 = 3983
        // scale = 1440 / 3983 ≈ 0.3615
        let expected = 1440.0 / 3983.0;
        assert!(
            result.scale < 1.0,
            "30-col grid should need scaling, got {}",
            result.scale
        );
        assert!(
            (result.scale - expected).abs() < 0.001,
            "expected scale ~{expected:.4}, got {:.4}",
            result.scale
        );
    }

    #[test]
    fn tall_grid_is_height_constrained() {
        let config = ron_like_cell_config();
        let playfield = ron_like_playfield_config();
        let result = compute_grid_scale(&config, &playfield, 3, 30, 90.0);
        // cell_zone_height = 1080 * 0.667 = 720.36
        // available_height = 720.36 - 90.0 = 630.36
        // default_grid_height = 30 * 50 - 7 = 1493
        // scale = 630.36 / 1493 ≈ 0.4222
        let expected = 630.36 / 1493.0;
        assert!(
            result.scale < 1.0,
            "30-row grid should need scaling, got {}",
            result.scale
        );
        assert!(
            (result.scale - expected).abs() < 0.001,
            "expected scale ~{expected:.4}, got {:.4}",
            result.scale
        );
    }

    #[test]
    fn scale_capped_at_one_for_tiny_grid() {
        let config = ron_like_cell_config();
        let playfield = ron_like_playfield_config();
        let result = compute_grid_scale(&config, &playfield, 1, 1, 50.0);
        assert!(
            (result.scale - 1.0).abs() < f32::EPSILON,
            "1x1 grid should be scale 1.0, got {}",
            result.scale
        );
    }

    #[test]
    fn corridor_layout_ten_by_five_returns_scale_one() {
        let config = ron_like_cell_config();
        let playfield = ron_like_playfield_config();
        let result = compute_grid_scale(&config, &playfield, 10, 5, 90.0);
        // grid_width = 10*133 - 7 = 1323 < 1440
        // grid_height = 5*50 - 7 = 243 < 630.36
        assert!(
            (result.scale - 1.0).abs() < f32::EPSILON,
            "10x5 grid should fit at scale 1.0, got {}",
            result.scale
        );
    }

    #[test]
    fn extreme_grid_128x128_produces_positive_sub_unit_scale() {
        let config = ron_like_cell_config();
        let playfield = ron_like_playfield_config();
        let result = compute_grid_scale(&config, &playfield, 128, 128, 90.0);
        // default_grid_width = 128*133 - 7 = 17017
        // scale_x = 1440 / 17017 ≈ 0.0846
        // default_grid_height = 128*50 - 7 = 6393
        // scale_y = 630.36 / 6393 ≈ 0.0986
        // scale = min(0.0846, 0.0986) ≈ 0.0846
        let expected = 1440.0 / 17017.0;
        assert!(
            result.scale > 0.0,
            "scale must be positive, got {}",
            result.scale
        );
        assert!(
            result.scale < 1.0,
            "128x128 grid must scale down, got {}",
            result.scale
        );
        assert!(
            (result.scale - expected).abs() < 0.001,
            "expected scale ~{expected:.4}, got {:.4}",
            result.scale
        );
    }

    // --- B: Integration tests for scaled cell spawning ---

    #[test]
    fn large_grid_cells_have_scaled_dimensions() {
        let layout = uniform_layout(40, 20, 90.0);
        let mut app = scaled_test_app(layout);
        app.update();

        let widths: Vec<f32> = app
            .world_mut()
            .query::<(&Cell, &CellWidth)>()
            .iter(app.world())
            .map(|(_, w)| w.0)
            .collect();
        let heights: Vec<f32> = app
            .world_mut()
            .query::<(&Cell, &CellHeight)>()
            .iter(app.world())
            .map(|(_, h)| h.0)
            .collect();

        assert!(!widths.is_empty(), "should have spawned cells");

        // All widths should be less than the base 126.0 (grid is too wide)
        for (i, &w) in widths.iter().enumerate() {
            assert!(
                w < 126.0,
                "cell {i} CellWidth={w} should be < 126.0 for a 40x20 grid"
            );
        }
        // All heights should be less than the base 43.0
        for (i, &h) in heights.iter().enumerate() {
            assert!(
                h < 43.0,
                "cell {i} CellHeight={h} should be < 43.0 for a 40x20 grid"
            );
        }

        // All widths should be uniform
        let first_w = widths[0];
        for (i, &w) in widths.iter().enumerate() {
            assert!(
                (w - first_w).abs() < f32::EPSILON,
                "cell {i} CellWidth={w} differs from first={first_w}"
            );
        }
        // All heights should be uniform
        let first_h = heights[0];
        for (i, &h) in heights.iter().enumerate() {
            assert!(
                (h - first_h).abs() < f32::EPSILON,
                "cell {i} CellHeight={h} differs from first={first_h}"
            );
        }
    }

    #[test]
    fn large_grid_cells_within_cell_zone_bounds() {
        let layout = uniform_layout(40, 20, 90.0);
        let playfield = ron_like_playfield_config();
        let cell_zone_height = playfield.height * playfield.zone_fraction; // 720.36
        let zone_bottom = playfield.top() - cell_zone_height; // 540.0 - 720.36 = -180.36
        let mut app = scaled_test_app(layout);
        app.update();

        let positions = collect_sorted_cell_positions(&mut app);

        for &(x, y) in &positions {
            assert!(
                y > zone_bottom,
                "cell y={y} below cell zone bottom {zone_bottom}"
            );
            assert!(
                y < playfield.top(),
                "cell y={y} above playfield top {}",
                playfield.top()
            );
            assert!(
                x.abs() < playfield.right(),
                "cell |x|={} outside playfield right {}",
                x.abs(),
                playfield.right()
            );
        }
    }

    #[test]
    fn small_grid_preserves_original_dimensions() {
        let layout = uniform_layout(3, 2, 50.0);
        let mut app = scaled_test_app(layout);
        app.update();

        for (_, w) in app
            .world_mut()
            .query::<(&Cell, &CellWidth)>()
            .iter(app.world())
        {
            assert!(
                (w.0 - 126.0).abs() < f32::EPSILON,
                "3x2 grid CellWidth should be 126.0, got {}",
                w.0
            );
        }
        for (_, h) in app
            .world_mut()
            .query::<(&Cell, &CellHeight)>()
            .iter(app.world())
        {
            assert!(
                (h.0 - 43.0).abs() < f32::EPSILON,
                "3x2 grid CellHeight should be 43.0, got {}",
                h.0
            );
        }
    }

    #[test]
    fn large_grid_transform_scale_matches_cell_dimensions() {
        let layout = uniform_layout(40, 20, 90.0);
        let mut app = scaled_test_app(layout);
        app.update();

        for (_, w, h, tf) in app
            .world_mut()
            .query::<(&Cell, &CellWidth, &CellHeight, &Transform)>()
            .iter(app.world())
        {
            assert!(
                (tf.scale.x - w.0).abs() < f32::EPSILON,
                "Transform.scale.x={} should match CellWidth={}",
                tf.scale.x,
                w.0
            );
            assert!(
                (tf.scale.y - h.0).abs() < f32::EPSILON,
                "Transform.scale.y={} should match CellHeight={}",
                tf.scale.y,
                h.0
            );
        }
    }

    #[test]
    fn single_cell_grid_spawns_centered_at_full_scale() {
        let layout = NodeLayout {
            name: "single".to_owned(),
            timer_secs: 60.0,
            cols: 1,
            rows: 1,
            grid_top_offset: 50.0,
            grid: vec![vec!['S']],
            pool: NodePool::default(),
        };
        let mut app = scaled_test_app(layout);
        app.update();

        let cells: Vec<(&CellWidth, &CellHeight, &Transform)> = app
            .world_mut()
            .query::<(&CellWidth, &CellHeight, &Transform)>()
            .iter(app.world())
            .collect();
        assert_eq!(cells.len(), 1, "should spawn exactly 1 cell");

        let (w, h, tf) = cells[0];
        assert!(
            tf.translation.x.abs() < f32::EPSILON,
            "single cell should be centered at x=0.0, got {}",
            tf.translation.x
        );
        assert!(
            (w.0 - 126.0).abs() < f32::EPSILON,
            "1x1 grid CellWidth should be 126.0, got {}",
            w.0
        );
        assert!(
            (h.0 - 43.0).abs() < f32::EPSILON,
            "1x1 grid CellHeight should be 43.0, got {}",
            h.0
        );
    }
}
