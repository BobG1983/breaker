//! System to spawn cells from the active node layout.

use std::collections::{HashMap, VecDeque};

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
        node::{ActiveNodeLayout, NodeLayout, definition::LockMap, messages::CellsSpawned},
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

/// Pre-computed grid positions and dimensions shared across spawn helpers.
struct GridSpawnParams {
    step_x: f32,
    step_y: f32,
    start_x: f32,
    start_y: f32,
    cell_width: f32,
    cell_height: f32,
    scale: f32,
}

impl GridSpawnParams {
    /// Computes the world-space position for a given grid coordinate.
    fn cell_pos(&self, row_idx: usize, col_idx: usize) -> Vec2 {
        let col_f = f32::from(u16::try_from(col_idx).unwrap_or(u16::MAX));
        let row_f = f32::from(u16::try_from(row_idx).unwrap_or(u16::MAX));
        let x = col_f.mul_add(self.step_x, self.start_x);
        let y = row_f.mul_add(-self.step_y, self.start_y);
        Vec2::new(x, y)
    }
}

/// Immutable context for cell spawning helpers. Bundles the shared read-only
/// state so individual functions stay under clippy's argument limit.
struct GridCellContext<'a> {
    layout: &'a NodeLayout,
    registry: &'a CellTypeRegistry,
    params: GridSpawnParams,
    hp_mult: f32,
}

impl GridCellContext<'_> {
    /// Pass 1: spawns non-locked cells and shield cells. Returns
    /// required-to-clear count for cells spawned in this pass.
    fn spawn_pass1(
        &self,
        commands: &mut Commands,
        lock_keys: &HashMap<(usize, usize), &Vec<(usize, usize)>>,
        rect_mesh: &Handle<Mesh>,
        meshes: &mut Assets<Mesh>,
        materials: &mut Assets<ColorMaterial>,
        entity_map: &mut HashMap<(usize, usize), Entity>,
    ) -> u32 {
        let mut required_count = 0u32;

        for (row_idx, row) in self.layout.grid.iter().enumerate() {
            for (col_idx, alias) in row.iter().enumerate() {
                if alias == "." {
                    continue;
                }
                let Some(def) = self.registry.get(alias) else {
                    continue;
                };

                let coord = (row_idx, col_idx);

                // Shield cells use the existing manual spawn path.
                if def.shield.is_some() {
                    let (entity_id, req_delta) = self.spawn_shield_cell(
                        commands,
                        def,
                        alias,
                        coord,
                        lock_keys,
                        (rect_mesh, materials),
                    );
                    required_count += req_delta;
                    entity_map.insert(coord, entity_id);
                    continue;
                }

                // Skip cells that are in the locks map — handled in Pass 2.
                if lock_keys.contains_key(&coord) {
                    continue;
                }

                // Non-locked, non-shield cell: spawn via builder.
                let pos = self.params.cell_pos(row_idx, col_idx);
                let scaled_hp = def.hp * self.hp_mult;

                let entity_id = Cell::builder()
                    .definition(def)
                    .position(pos)
                    .dimensions(self.params.cell_width, self.params.cell_height)
                    .override_hp(scaled_hp)
                    .alias(alias.clone())
                    .rendered(meshes, materials)
                    .spawn(commands);

                if def.required_to_clear {
                    required_count += 1;
                }
                entity_map.insert(coord, entity_id);
            }
        }

        required_count
    }

    /// Spawns a single shield cell (manual path) and inserts orbit children.
    /// Returns `(entity_id, required_count_delta)`.
    fn spawn_shield_cell(
        &self,
        commands: &mut Commands,
        def: &crate::cells::CellTypeDefinition,
        alias: &str,
        coord: (usize, usize),
        lock_keys: &HashMap<(usize, usize), &Vec<(usize, usize)>>,
        render: (&Handle<Mesh>, &mut Assets<ColorMaterial>),
    ) -> (Entity, u32) {
        let (rect_mesh, materials) = render;
        let (row_idx, col_idx) = coord;

        if lock_keys.contains_key(&coord) {
            debug!(
                "layout '{}': shield cell at ({}, {}) also appears in locks map — \
                 shield path takes priority, lock entry skipped",
                self.layout.name, row_idx, col_idx
            );
        }

        let pos = self.params.cell_pos(row_idx, col_idx);
        let scaled_hp = def.hp * self.hp_mult;
        let cell_width = self.params.cell_width;
        let cell_height = self.params.cell_height;

        let mut required_delta = 0u32;

        // Block scope limits EntityCommands borrow so `commands` can
        // be used for orbit children below.
        let cell_entity_id = {
            let mut entity = commands.spawn((
                Cell,
                CellTypeAlias(alias.to_owned()),
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
                Position2D(pos),
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
                required_delta = 1;
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
            entity.insert((ShieldParent, Locked));
            entity.id()
        };

        if let Some(ref shield) = def.shield {
            spawn_orbit_children(
                commands,
                shield,
                cell_entity_id,
                pos,
                self.params.scale,
                self.hp_mult,
                (rect_mesh, materials),
            );
        }
        (cell_entity_id, required_delta)
    }
}

/// Spawns cells from a grid layout. Returns the count of `RequiredToClear` cells.
///
/// Shared between the `OnEnter(Playing)` system and hot-reload respawn.
///
/// `hp_mult` scales every cell's HP (from the node's difficulty assignment).
///
/// Uses a two-pass approach when the layout has locks:
/// - **Pass 1**: spawns non-locked cells (and shield cells via the existing manual path)
/// - **Pass 2**: spawns locked cells in topological order with resolved entity IDs
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

    let grid_width = grid_extent(
        dims.step_x,
        f32::from(u16::try_from(layout.cols).unwrap_or(u16::MAX)),
        dims.padding_x,
    );

    let rect_mesh = meshes.add(Rectangle::new(1.0, 1.0));

    let lock_keys: HashMap<(usize, usize), &Vec<(usize, usize)>> = layout
        .locks
        .as_ref()
        .map(|locks| locks.iter().map(|(k, v)| (*k, v)).collect())
        .unwrap_or_default();

    let ctx = GridCellContext {
        layout,
        registry,
        hp_mult,
        params: GridSpawnParams {
            step_x: dims.step_x,
            step_y: dims.step_y,
            start_x: -grid_width / 2.0 + dims.cell_width / 2.0,
            start_y: playfield.top() - layout.grid_top_offset - dims.cell_height / 2.0,
            cell_width: dims.cell_width,
            cell_height: dims.cell_height,
            scale: dims.scale,
        },
    };

    let mut entity_map: HashMap<(usize, usize), Entity> = HashMap::new();

    let mut required_count = ctx.spawn_pass1(
        commands,
        &lock_keys,
        &rect_mesh,
        meshes,
        materials,
        &mut entity_map,
    );

    required_count += resolve_and_spawn_locks(&ctx, commands, &mut entity_map, meshes, materials);

    required_count
}

/// Pass 2: resolves lock dependencies via topological sort and spawns locked
/// cells in dependency order. Returns the additional required-to-clear count.
fn resolve_and_spawn_locks(
    ctx: &GridCellContext<'_>,
    commands: &mut Commands,
    entity_map: &mut HashMap<(usize, usize), Entity>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) -> u32 {
    let Some(ref locks) = ctx.layout.locks else {
        return 0;
    };
    if locks.is_empty() {
        return 0;
    }

    // Filter out lock keys that are shield cells (already spawned in Pass 1).
    let effective_locks: LockMap = locks
        .iter()
        .filter(|&(&(r, c), _)| !entity_map.contains_key(&(r, c)))
        .map(|(&k, v)| (k, v.clone()))
        .collect();

    let (sorted, cyclic) = topological_sort_locks(&effective_locks);
    let mut required_count = 0u32;

    // Spawn sorted (acyclic) locked cells.
    for coord in sorted {
        required_count += spawn_locked_cell(
            ctx,
            commands,
            &effective_locks,
            entity_map,
            coord,
            meshes,
            materials,
        );
    }

    // Spawn cyclic entries as unlocked fallbacks.
    for coord in cyclic {
        required_count +=
            spawn_unlocked_fallback(ctx, commands, entity_map, coord, meshes, materials);
    }

    required_count
}

/// Spawns a single locked cell, resolving its lock targets to entity IDs.
/// Returns 1 if the cell is required-to-clear, 0 otherwise.
fn spawn_locked_cell(
    ctx: &GridCellContext<'_>,
    commands: &mut Commands,
    effective_locks: &LockMap,
    entity_map: &mut HashMap<(usize, usize), Entity>,
    coord: (usize, usize),
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) -> u32 {
    let (row_idx, col_idx) = coord;
    let Some(alias) = ctx
        .layout
        .grid
        .get(row_idx)
        .and_then(|row| row.get(col_idx))
    else {
        return 0;
    };
    if alias == "." {
        return 0;
    }
    let Some(def) = ctx.registry.get(alias) else {
        return 0;
    };

    let pos = ctx.params.cell_pos(row_idx, col_idx);
    let scaled_hp = def.hp * ctx.hp_mult;

    // Resolve lock targets to entity IDs.
    let targets = &effective_locks[&coord];
    let resolved: Vec<Entity> = targets
        .iter()
        .filter_map(|target| {
            entity_map.get(target).copied().or_else(|| {
                debug!(
                    "layout '{}': lock target ({}, {}) for cell ({}, {}) \
                     has no spawned entity — skipping target",
                    ctx.layout.name, target.0, target.1, row_idx, col_idx
                );
                None
            })
        })
        .collect();

    // If no targets resolved, spawn without Locked (graceful degradation).
    let entity_id = if resolved.is_empty() {
        Cell::builder()
            .definition(def)
            .position(pos)
            .dimensions(ctx.params.cell_width, ctx.params.cell_height)
            .override_hp(scaled_hp)
            .alias(alias.clone())
            .rendered(meshes, materials)
            .spawn(commands)
    } else {
        Cell::builder()
            .definition(def)
            .position(pos)
            .dimensions(ctx.params.cell_width, ctx.params.cell_height)
            .override_hp(scaled_hp)
            .alias(alias.clone())
            .locked(resolved)
            .rendered(meshes, materials)
            .spawn(commands)
    };

    entity_map.insert(coord, entity_id);
    u32::from(def.required_to_clear)
}

/// Spawns a cell as an unlocked fallback (used for cyclic lock entries).
/// Returns 1 if the cell is required-to-clear, 0 otherwise.
fn spawn_unlocked_fallback(
    ctx: &GridCellContext<'_>,
    commands: &mut Commands,
    entity_map: &mut HashMap<(usize, usize), Entity>,
    coord: (usize, usize),
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ColorMaterial>,
) -> u32 {
    let (row_idx, col_idx) = coord;
    let Some(alias) = ctx
        .layout
        .grid
        .get(row_idx)
        .and_then(|row| row.get(col_idx))
    else {
        return 0;
    };
    if alias == "." {
        return 0;
    }
    let Some(def) = ctx.registry.get(alias) else {
        return 0;
    };

    let pos = ctx.params.cell_pos(row_idx, col_idx);
    let scaled_hp = def.hp * ctx.hp_mult;

    let entity_id = Cell::builder()
        .definition(def)
        .position(pos)
        .dimensions(ctx.params.cell_width, ctx.params.cell_height)
        .override_hp(scaled_hp)
        .alias(alias.clone())
        .rendered(meshes, materials)
        .spawn(commands);

    entity_map.insert(coord, entity_id);
    u32::from(def.required_to_clear)
}

/// Performs a topological sort of lock entries using Kahn's algorithm.
///
/// Returns `(sorted, cyclic)`:
/// - `sorted`: lock keys in an order where all dependencies come first.
/// - `cyclic`: lock keys that participate in a cycle (could not be ordered).
type GridCoord = (usize, usize);

fn topological_sort_locks(locks: &LockMap) -> (Vec<GridCoord>, Vec<GridCoord>) {
    // Build in-degree map. Only edges between lock-map keys matter.
    let mut in_degree: HashMap<(usize, usize), usize> = HashMap::new();
    // dependents[target] = list of lock keys that depend on target
    let mut dependents: HashMap<(usize, usize), Vec<(usize, usize)>> = HashMap::new();

    for &key in locks.keys() {
        in_degree.entry(key).or_insert(0);
    }

    for (&key, targets) in locks {
        for target in targets {
            // Only count edges where the target is also a lock key.
            if locks.contains_key(target) {
                *in_degree.entry(key).or_insert(0) += 1;
                dependents.entry(*target).or_default().push(key);
            }
        }
    }

    // Seed the queue with all lock keys that have in-degree 0.
    let mut queue: VecDeque<(usize, usize)> = in_degree
        .iter()
        .filter(|entry| *entry.1 == 0)
        .map(|entry| *entry.0)
        .collect();

    let mut sorted = Vec::with_capacity(locks.len());

    while let Some(node) = queue.pop_front() {
        sorted.push(node);
        if let Some(deps) = dependents.get(&node) {
            for &dep in deps {
                if let Some(deg) = in_degree.get_mut(&dep) {
                    *deg -= 1;
                    if *deg == 0 {
                        queue.push_back(dep);
                    }
                }
            }
        }
    }

    // Remaining nodes with in-degree > 0 are cyclic.
    let cyclic: Vec<(usize, usize)> = in_degree
        .into_iter()
        .filter(|(_, deg)| *deg > 0)
        .map(|(k, _)| k)
        .collect();

    if !cyclic.is_empty() {
        debug!(
            "topological_sort_locks: detected cycle among {} lock entries: {:?}",
            cyclic.len(),
            cyclic
        );
    }

    (sorted, cyclic)
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
