//! System to spawn cells from the active node layout.

use bevy::{ecs::system::SystemParam, prelude::*};
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::{
    components::{Position2D, Scale2D},
    propagation::PositionPropagation,
};

use crate::{
    cells::{
        components::*,
        definition::ShieldBehavior,
        resources::{CellConfig, CellTypeRegistry},
    },
    shared::{BOLT_LAYER, CELL_LAYER, GameDrawLayer, PlayfieldConfig},
    state::run::{
        node::{ActiveNodeLayout, NodeLayout, messages::CellsSpawned},
        resources::{NodeOutcome, NodeSequence},
    },
};

/// Total extent of a grid along one axis: `step * count - padding`.
pub(crate) fn grid_extent(step: f32, count_f: f32, padding: f32) -> f32 {
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
    let step_y = config.padding_y.mul_add(scale, cell_height);
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

    let rect_mesh = meshes.add(Rectangle::new(1.0, 1.0));
    let mut required_count = 0u32;

    for (row_idx, row) in layout.grid.iter().enumerate() {
        for (col_idx, alias) in row.iter().enumerate() {
            if alias == "." {
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

            // Block scope limits EntityCommands borrow so `commands` can
            // be used for orbit children below.
            let cell_entity_id = {
                let mut entity = commands.spawn((
                    Cell,
                    CellTypeAlias(alias.clone()),
                    CellWidth::new(cell_width),
                    CellHeight::new(cell_height),
                    CellHealth::new(scaled_hp),
                    CellDamageVisuals {
                        hdr_base: def.damage_hdr_base,
                        green_min: def.damage_green_min,
                        blue_range: def.damage_blue_range,
                        blue_base: def.damage_blue_base,
                    },
                    Mesh2d(rect_mesh.clone()),
                    MeshMaterial2d(materials.add(ColorMaterial::from_color(def.color()))),
                    Position2D(Vec2::new(x, y)),
                    Scale2D {
                        x: cell_width,
                        y: cell_height,
                    },
                    Aabb2D::new(Vec2::ZERO, Vec2::new(cell_width / 2.0, cell_height / 2.0)),
                    CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
                    GameDrawLayer::Cell,
                ));

                if def.required_to_clear {
                    entity.insert(RequiredToClear);
                    required_count += 1;
                }
                if let Some(ref behaviors) = def.behaviors {
                    for behavior in behaviors {
                        match behavior {
                            crate::cells::definition::CellBehavior::Regen { rate } => {
                                entity.insert(CellRegen { rate: *rate });
                            }
                        }
                    }
                }
                if def.shield.is_some() {
                    entity.insert((ShieldParent, Locked));
                }
                entity.id()
            };

            if let Some(ref shield) = def.shield {
                spawn_orbit_children(
                    commands,
                    shield,
                    cell_entity_id,
                    Vec2::new(x, y),
                    scale,
                    hp_mult,
                    (&rect_mesh, materials),
                );
            }
        }
    }
    required_count
}

/// Spawns orbit children around a shield cell and inserts `LockAdjacents`.
fn spawn_orbit_children(
    commands: &mut Commands,
    shield: &ShieldBehavior,
    shield_entity: Entity,
    center: Vec2,
    scale: f32,
    hp_mult: f32,
    render: (&Handle<Mesh>, &mut Assets<ColorMaterial>),
) {
    let (rect_mesh, materials) = render;
    let orbit_dim = 20.0 * scale;
    let orbit_half = orbit_dim / 2.0;
    let orbit_half_diag = orbit_half * std::f32::consts::SQRT_2;
    let min_clamp = orbit_half_diag + 1.0;
    let scaled_radius = (shield.radius * scale).max(min_clamp);

    let orbit_color = crate::shared::color_from_rgb(shield.color_rgb);
    let orbit_material = materials.add(ColorMaterial::from_color(orbit_color));

    let orbit_count_f = f32::from(u16::try_from(shield.count).unwrap_or(u16::MAX));
    let mut orbit_ids = Vec::with_capacity(shield.count as usize);
    for i in 0..shield.count {
        let i_f = f32::from(u16::try_from(i).unwrap_or(u16::MAX));
        let angle = 2.0 * std::f32::consts::PI * i_f / orbit_count_f;
        let offset = Vec2::new(scaled_radius * angle.cos(), scaled_radius * angle.sin());
        let orbit_pos = center + offset;

        let orbit_entity = commands
            .spawn((
                Cell,
                OrbitCell,
                ChildOf(shield_entity),
                CellHealth::new(shield.hp * hp_mult),
                CellWidth::new(orbit_dim),
                CellHeight::new(orbit_dim),
                Mesh2d(rect_mesh.clone()),
                MeshMaterial2d(orbit_material.clone()),
                Position2D(orbit_pos),
                PositionPropagation::Absolute,
                Scale2D {
                    x: orbit_dim,
                    y: orbit_dim,
                },
                (
                    Aabb2D::new(Vec2::ZERO, Vec2::new(orbit_half, orbit_half)),
                    CollisionLayers::new(CELL_LAYER, BOLT_LAYER),
                    OrbitAngle(angle),
                    OrbitConfig {
                        radius: scaled_radius,
                        speed: shield.speed,
                    },
                    GameDrawLayer::Cell,
                ),
            ))
            .id();
        orbit_ids.push(orbit_entity);
    }

    commands
        .entity(shield_entity)
        .insert(LockAdjacents(orbit_ids));
}

/// Resolves the `hp_mult` for the current node from the run state and node
/// sequence, falling back to `1.0` when those resources are absent (e.g. in
/// tests or scenario overrides).
fn resolve_hp_mult(run_state: Option<&NodeOutcome>, node_sequence: Option<&NodeSequence>) -> f32 {
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
    run_state: Option<Res<'w, NodeOutcome>>,
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
    spawn_cells_from_grid(
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
    cells_spawned.write(CellsSpawned);
}
