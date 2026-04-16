//! System to spawn cells from the active node layout.

use std::collections::{HashMap, HashSet};

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{
    cells::{
        builder::core::types::GuardianSpawnConfig,
        components::*,
        definition::{CellBehavior, Toughness},
        resources::{CellConfig, CellTypeRegistry, ToughnessConfig},
    },
    prelude::*,
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
    pub cell_width:  f32,
    pub cell_height: f32,
    pub padding_x:   f32,
    pub step_x:      f32,
    pub step_y:      f32,
    pub scale:       f32,
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
            cell_width:  0.0,
            cell_height: 0.0,
            padding_x:   0.0,
            step_x:      0.0,
            step_y:      0.0,
            scale:       0.0,
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
    pub meshes:    &'a mut Assets<Mesh>,
    pub materials: &'a mut Assets<ColorMaterial>,
}

/// Pre-computed grid positions and dimensions shared across spawn helpers.
pub(super) struct GridSpawnParams {
    pub(super) step_x:      f32,
    pub(super) step_y:      f32,
    pub(super) start_x:     f32,
    pub(super) start_y:     f32,
    pub(super) cell_width:  f32,
    pub(super) cell_height: f32,
}

impl GridSpawnParams {
    /// Computes the world-space position for a given grid coordinate.
    pub(super) fn cell_pos(&self, row_idx: usize, col_idx: usize) -> Vec2 {
        let col_f = f32::from(u16::try_from(col_idx).unwrap_or(u16::MAX));
        let row_f = f32::from(u16::try_from(row_idx).unwrap_or(u16::MAX));
        let x = col_f.mul_add(self.step_x, self.start_x);
        let y = row_f.mul_add(-self.step_y, self.start_y);
        Vec2::new(x, y)
    }
}

/// Context for toughness-based HP resolution at spawn time.
pub(crate) struct HpContext {
    pub tier:             u32,
    pub position_in_tier: u32,
    pub is_boss:          bool,
}

/// Bundles toughness config and HP context to reduce argument count on
/// [`spawn_cells_from_grid`].
pub(crate) struct ToughnessHpData<'a> {
    pub toughness_config: Option<&'a ToughnessConfig>,
    pub hp_context:       HpContext,
}

/// Immutable context for cell spawning helpers. Bundles the shared read-only
/// state so individual functions stay under clippy's argument limit.
pub(super) struct GridCellContext<'a> {
    pub(super) layout:   &'a NodeLayout,
    pub(super) registry: &'a CellTypeRegistry,
    pub(super) params:   GridSpawnParams,
    /// Precomputed HP scaling — tier scale computed once per batch, config
    /// reference kept for per-cell `base_hp()` lookup.
    hp_scale:            HpScale<'a>,
}

/// Precomputed HP scaling for a spawn batch. Caches the tier scale factor
/// (computed once via `powi()`) and keeps the config reference for per-cell
/// `base_hp()` lookups. Without a config, falls back to `default_base_hp()`.
#[derive(Clone, Copy)]
struct HpScale<'a> {
    config: Option<&'a ToughnessConfig>,
    /// `tier_scale * boss_multiplier` (if boss) or `tier_scale` (if not).
    scale:  f32,
}

impl<'a> HpScale<'a> {
    fn from_context(config: Option<&'a ToughnessConfig>, hp_context: &HpContext) -> Self {
        config.map_or(
            Self {
                config: None,
                scale:  1.0,
            },
            |c| {
                let tier = c.tier_scale(hp_context.tier, hp_context.position_in_tier);
                Self {
                    config: Some(c),
                    scale:  if hp_context.is_boss {
                        tier * c.boss_multiplier
                    } else {
                        tier
                    },
                }
            },
        )
    }
}

/// Read-only lookup maps consumed by `spawn_pass1` — grouped so the method
/// signature stays under clippy's argument-count ceiling.
pub(super) struct Pass1Lookups<'a> {
    pub lock_keys:       &'a HashMap<(usize, usize), &'a Vec<(usize, usize)>>,
    pub sequence_lookup: &'a HashMap<(usize, usize), (u32, u32)>,
    pub gu_skip:         &'a HashSet<(usize, usize)>,
}

impl GridCellContext<'_> {
    /// Computes HP for a cell: `base_hp(toughness) * precomputed_scale`.
    pub(super) fn compute_hp(&self, toughness: Toughness) -> f32 {
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
        lookups: &Pass1Lookups<'_>,
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
                if lookups.gu_skip.contains(&coord) {
                    continue;
                }

                let Some(def) = self.registry.get(alias) else {
                    continue;
                };

                // Check if this cell type has a Guarded behavior.
                let guarded_behavior = def.behaviors.as_ref().and_then(|behaviors| {
                    behaviors.iter().find_map(|b| match b {
                        CellBehavior::Guarded(g) => Some(g),
                        CellBehavior::Regen { .. }
                        | CellBehavior::Volatile { .. }
                        | CellBehavior::Sequence { .. }
                        | CellBehavior::Armored { .. } => None,
                    })
                });

                if let Some(guarded) = guarded_behavior {
                    let guardian_slots =
                        collect_guardian_slots(&self.layout.grid, row_idx, col_idx);
                    let parent_hp = self.compute_hp(def.toughness);
                    let guardian_hp = parent_hp * guarded.guardian_hp_fraction;
                    let config = GuardianSpawnConfig {
                        hp:          guardian_hp,
                        color_rgb:   guarded.guardian_color_rgb,
                        slide_speed: guarded.slide_speed,
                        cell_height: self.params.cell_height,
                        step_x:      self.params.step_x,
                        step_y:      self.params.step_y,
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
                if lookups.lock_keys.contains_key(&coord) {
                    continue;
                }

                // Non-locked cell: spawn via builder.
                let pos = self.params.cell_pos(row_idx, col_idx);
                let scaled_hp = self.compute_hp(def.toughness);

                let mut builder = Cell::builder()
                    .definition(def)
                    .position(pos)
                    .dimensions(self.params.cell_width, self.params.cell_height)
                    .override_hp(scaled_hp)
                    .alias(alias.clone());
                if let Some(&(group, position)) = lookups.sequence_lookup.get(&coord) {
                    builder = builder.sequence(group, position);
                }
                let entity_id = builder.rendered(meshes, materials).spawn(commands);

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

    let sequence_lookup: HashMap<(usize, usize), (u32, u32)> = layout
        .sequences
        .as_ref()
        .map(|sequences| {
            sequences
                .iter()
                .flat_map(|(&group, members)| {
                    members.iter().enumerate().map(move |(idx, &coord)| {
                        let position = u32::try_from(idx).unwrap_or(u32::MAX);
                        (coord, (group, position))
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    let hp_scale = HpScale::from_context(toughness_config, &hp_context);
    let ctx = GridCellContext {
        layout,
        registry,
        hp_scale,
        params: GridSpawnParams {
            step_x:      dims.step_x,
            step_y:      dims.step_y,
            start_x:     -grid_width / 2.0 + dims.cell_width / 2.0,
            start_y:     playfield.top() - layout.grid_top_offset - dims.cell_height / 2.0,
            cell_width:  dims.cell_width,
            cell_height: dims.cell_height,
        },
    };

    // Pre-scan: collect guardian (`gu`) positions that belong to guarded (`Gu`) parents.
    let gu_skip = build_guardian_skip_set(&layout.grid, registry);

    let mut entity_map: HashMap<(usize, usize), Entity> = HashMap::new();

    let lookups = Pass1Lookups {
        lock_keys:       &lock_keys,
        sequence_lookup: &sequence_lookup,
        gu_skip:         &gu_skip,
    };
    let mut required_count =
        ctx.spawn_pass1(commands, &lookups, meshes, materials, &mut entity_map);

    required_count += super::lock_resolution::resolve_and_spawn_locks(
        &ctx,
        commands,
        &mut entity_map,
        meshes,
        materials,
    );

    required_count
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
    cell_config:      Res<'w, CellConfig>,
    playfield_config: Res<'w, PlayfieldConfig>,
    cell_registry:    Res<'w, CellTypeRegistry>,
    run_state:        Option<Res<'w, NodeOutcome>>,
    node_sequence:    Option<Res<'w, NodeSequence>>,
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
            meshes:    &mut render_assets.0,
            materials: &mut render_assets.1,
        },
        ToughnessHpData {
            toughness_config: ctx.toughness_config.as_deref(),
            hp_context,
        },
    );
    cells_spawned.write(CellsSpawned);
}
