//! System to spawn cells from the active node layout.

use std::collections::{HashMap, HashSet, VecDeque};

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{
    cells::{
        builder::core::types::GuardianSpawnConfig,
        components::*,
        definition::{CellBehavior, Toughness},
        resources::{CellConfig, CellTypeRegistry, ToughnessConfig},
    },
    shared::PlayfieldConfig,
    state::run::{
        definition::NodeType,
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

    if default_grid_width <= 0.0 || default_grid_height <= 0.0 {
        warn!(
            "compute_grid_scale: degenerate layout (cols={cols}, rows={rows}), \
             grid extent is zero or negative"
        );
        return ScaledGridDims {
            cell_width: 0.0,
            cell_height: 0.0,
            padding_x: 0.0,
            step_x: 0.0,
            step_y: 0.0,
            scale: 0.0,
        };
    }

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

/// Context for toughness-based HP resolution at spawn time.
pub(crate) struct HpContext {
    pub tier: u32,
    pub position_in_tier: u32,
    pub is_boss: bool,
}

/// Bundles toughness config and HP context to reduce argument count on
/// [`spawn_cells_from_grid`].
pub(crate) struct ToughnessHpData<'a> {
    pub toughness_config: Option<&'a ToughnessConfig>,
    pub hp_context: HpContext,
}

/// Immutable context for cell spawning helpers. Bundles the shared read-only
/// state so individual functions stay under clippy's argument limit.
struct GridCellContext<'a> {
    layout: &'a NodeLayout,
    registry: &'a CellTypeRegistry,
    params: GridSpawnParams,
    /// Precomputed HP scaling — tier scale computed once per batch, config
    /// reference kept for per-cell `base_hp()` lookup.
    hp_scale: HpScale<'a>,
}

/// Precomputed HP scaling for a spawn batch. Caches the tier scale factor
/// (computed once via `powi()`) and keeps the config reference for per-cell
/// `base_hp()` lookups. Without a config, falls back to `default_base_hp()`.
#[derive(Clone, Copy)]
struct HpScale<'a> {
    config: Option<&'a ToughnessConfig>,
    /// `tier_scale * boss_multiplier` (if boss) or `tier_scale` (if not).
    scale: f32,
}

impl<'a> HpScale<'a> {
    fn from_context(config: Option<&'a ToughnessConfig>, hp_context: &HpContext) -> Self {
        config.map_or(
            Self {
                config: None,
                scale: 1.0,
            },
            |c| {
                let tier = c.tier_scale(hp_context.tier, hp_context.position_in_tier);
                Self {
                    config: Some(c),
                    scale: if hp_context.is_boss {
                        tier * c.boss_multiplier
                    } else {
                        tier
                    },
                }
            },
        )
    }
}

impl GridCellContext<'_> {
    /// Computes HP for a cell: `base_hp(toughness) * precomputed_scale`.
    fn compute_hp(&self, toughness: Toughness) -> f32 {
        let base = self
            .hp_scale
            .config
            .map_or_else(|| toughness.default_base_hp(), |c| c.base_hp(toughness));
        base * self.hp_scale.scale
    }

    /// Pass 1: spawns non-locked cells. Returns
    /// required-to-clear count for cells spawned in this pass.
    fn spawn_pass1(
        &self,
        commands: &mut Commands,
        lock_keys: &HashMap<(usize, usize), &Vec<(usize, usize)>>,
        gu_skip: &HashSet<(usize, usize)>,
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
                let coord = (row_idx, col_idx);

                // Skip guardian grid positions — consumed by the guarded parent.
                if gu_skip.contains(&coord) {
                    continue;
                }

                let Some(def) = self.registry.get(alias) else {
                    continue;
                };

                // Check if this cell type has a Guarded behavior.
                let guarded_behavior = def.behaviors.as_ref().and_then(|behaviors| {
                    behaviors.iter().find_map(|b| match b {
                        CellBehavior::Guarded(g) => Some(g),
                        CellBehavior::Regen { .. } => None,
                    })
                });

                if let Some(guarded) = guarded_behavior {
                    let guardian_slots =
                        collect_guardian_slots(&self.layout.grid, row_idx, col_idx);
                    let parent_hp = self.compute_hp(def.toughness);
                    let guardian_hp = parent_hp * guarded.guardian_hp_fraction;
                    let config = GuardianSpawnConfig {
                        hp: guardian_hp,
                        color_rgb: guarded.guardian_color_rgb,
                        slide_speed: guarded.slide_speed,
                        cell_height: self.params.cell_height,
                        step_x: self.params.step_x,
                        step_y: self.params.step_y,
                    };
                    let pos = self.params.cell_pos(row_idx, col_idx);
                    let entity_id = Cell::builder()
                        .definition(def)
                        .position(pos)
                        .dimensions(self.params.cell_width, self.params.cell_height)
                        .override_hp(parent_hp)
                        .alias(alias.clone())
                        .guarded(guardian_slots, config)
                        .rendered(meshes, materials)
                        .spawn(commands);
                    if def.required_to_clear {
                        required_count += 1;
                    }
                    entity_map.insert(coord, entity_id);
                    continue;
                }

                // Skip cells that are in the locks map — handled in Pass 2.
                if lock_keys.contains_key(&coord) {
                    continue;
                }

                // Non-locked cell: spawn via builder.
                let pos = self.params.cell_pos(row_idx, col_idx);
                let scaled_hp = self.compute_hp(def.toughness);

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
}

/// Spawns cells from a grid layout. Returns the count of `RequiredToClear` cells.
///
/// Shared between the `OnEnter(Playing)` system and hot-reload respawn.
///
/// Uses a two-pass approach when the layout has locks:
/// - **Pass 1**: spawns non-locked cells
/// - **Pass 2**: spawns locked cells in topological order with resolved entity IDs
pub(crate) fn spawn_cells_from_grid(
    commands: &mut Commands,
    config: &CellConfig,
    playfield: &PlayfieldConfig,
    layout: &NodeLayout,
    registry: &CellTypeRegistry,
    render_assets: RenderAssets<'_>,
    toughness_hp: ToughnessHpData<'_>,
) -> u32 {
    let RenderAssets { meshes, materials } = render_assets;
    let ToughnessHpData {
        toughness_config,
        hp_context,
    } = toughness_hp;
    let dims = compute_grid_scale(
        config,
        playfield,
        layout.cols,
        layout.rows,
        layout.grid_top_offset,
    );

    debug!(
        "spawn_cells_from_grid: layout '{}' cols={} rows={} scale={:.3}",
        layout.name, layout.cols, layout.rows, dims.scale
    );

    let grid_width = grid_extent(
        dims.step_x,
        f32::from(u16::try_from(layout.cols).unwrap_or(u16::MAX)),
        dims.padding_x,
    );

    let lock_keys: HashMap<(usize, usize), &Vec<(usize, usize)>> = layout
        .locks
        .as_ref()
        .map(|locks| locks.iter().map(|(k, v)| (*k, v)).collect())
        .unwrap_or_default();

    let hp_scale = HpScale::from_context(toughness_config, &hp_context);
    let ctx = GridCellContext {
        layout,
        registry,
        hp_scale,
        params: GridSpawnParams {
            step_x: dims.step_x,
            step_y: dims.step_y,
            start_x: -grid_width / 2.0 + dims.cell_width / 2.0,
            start_y: playfield.top() - layout.grid_top_offset - dims.cell_height / 2.0,
            cell_width: dims.cell_width,
            cell_height: dims.cell_height,
        },
    };

    // Pre-scan: collect guardian (`gu`) positions that belong to guarded (`Gu`) parents.
    let gu_skip = build_guardian_skip_set(&layout.grid, registry);

    let mut entity_map: HashMap<(usize, usize), Entity> = HashMap::new();

    let mut required_count = ctx.spawn_pass1(
        commands,
        &lock_keys,
        &gu_skip,
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

    // Filter out lock keys that reference cells already spawned in Pass 1.
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
    let scaled_hp = ctx.compute_hp(def.toughness);

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
    let scaled_hp = ctx.compute_hp(def.toughness);

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

/// Builds a set of grid positions occupied by `gu` (guardian) aliases that are
/// adjacent to a `Gu` (guarded) cell. These positions are consumed by the
/// guarded parent's builder and must not be spawned independently.
fn build_guardian_skip_set(
    grid: &[Vec<String>],
    registry: &CellTypeRegistry,
) -> HashSet<(usize, usize)> {
    let mut skip = HashSet::new();
    let offsets: [(i32, i32); 8] = [
        (-1, -1),
        (-1, 0),
        (-1, 1),
        (0, 1),
        (1, 1),
        (1, 0),
        (1, -1),
        (0, -1),
    ];
    for (row_idx, row) in grid.iter().enumerate() {
        for (col_idx, alias) in row.iter().enumerate() {
            // Check if this cell has a Guarded behavior.
            let is_guarded = registry.get(alias).is_some_and(|def| {
                def.behaviors.as_ref().is_some_and(|behaviors| {
                    behaviors
                        .iter()
                        .any(|b| matches!(b, CellBehavior::Guarded(_)))
                })
            });
            if !is_guarded {
                continue;
            }
            for &(dr, dc) in &offsets {
                let Some(nr) =
                    usize::try_from(i32::try_from(row_idx).unwrap_or(i32::MAX) + dr).ok()
                else {
                    continue;
                };
                let Some(nc) =
                    usize::try_from(i32::try_from(col_idx).unwrap_or(i32::MAX) + dc).ok()
                else {
                    continue;
                };
                if let Some(neighbor_alias) = grid.get(nr).and_then(|r| r.get(nc))
                    && neighbor_alias == "gu"
                {
                    skip.insert((nr, nc));
                }
            }
        }
    }
    skip
}

/// Scans the 3x3 neighborhood around a guarded cell and returns ring slot
/// indices (0-7) for each adjacent position containing a `gu` alias.
fn collect_guardian_slots(grid: &[Vec<String>], center_row: usize, center_col: usize) -> Vec<u8> {
    // Grid offset (row_delta, col_delta) to ring slot index mapping.
    const OFFSET_TO_SLOT: [((i32, i32), u8); 8] = [
        ((-1, -1), 0),
        ((-1, 0), 1),
        ((-1, 1), 2),
        ((0, 1), 3),
        ((1, 1), 4),
        ((1, 0), 5),
        ((1, -1), 6),
        ((0, -1), 7),
    ];
    let mut slots = Vec::new();
    for &((dr, dc), slot) in &OFFSET_TO_SLOT {
        let Some(nr) = usize::try_from(i32::try_from(center_row).unwrap_or(i32::MAX) + dr).ok()
        else {
            continue;
        };
        let Some(nc) = usize::try_from(i32::try_from(center_col).unwrap_or(i32::MAX) + dc).ok()
        else {
            continue;
        };
        if let Some(alias) = grid.get(nr).and_then(|r| r.get(nc))
            && alias == "gu"
        {
            slots.push(slot);
        }
    }
    slots
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

/// Resolves the HP context from the current run state and node sequence.
fn resolve_hp_context(
    run_state: Option<&NodeOutcome>,
    node_sequence: Option<&NodeSequence>,
) -> HpContext {
    let (tier, position_in_tier, is_boss) =
        if let (Some(state), Some(sequence)) = (run_state, node_sequence) {
            let assignment = sequence
                .assignments
                .get(usize::try_from(state.node_index).unwrap_or(usize::MAX));
            let is_boss = assignment.is_some_and(|a| a.node_type == NodeType::Boss);
            (state.tier, state.position_in_tier, is_boss)
        } else {
            (0, 0, false)
        };
    HpContext {
        tier,
        position_in_tier,
        is_boss,
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
    toughness_config: Option<Res<'w, ToughnessConfig>>,
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
    let hp_context = resolve_hp_context(ctx.run_state.as_deref(), ctx.node_sequence.as_deref());
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
        ToughnessHpData {
            toughness_config: ctx.toughness_config.as_deref(),
            hp_context,
        },
    );
    cells_spawned.write(CellsSpawned);
}
