//! Grid geometry helpers — pure computation, no ECS.

use bevy::prelude::*;

use crate::{cells::resources::CellConfig, prelude::*};

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
