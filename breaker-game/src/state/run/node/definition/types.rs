//! Node layout definition — RON-deserialized data for a single node layout.

use std::collections::HashMap;

use bevy::prelude::*;
use serde::Deserialize;

#[cfg(test)]
use crate::cells::CellTypeRegistry;

/// Maps locked cell positions `(row, col)` to the positions of cells that
/// must be destroyed to unlock them.
pub type LockMap = HashMap<(usize, usize), Vec<(usize, usize)>>;

#[cfg(test)]
/// Maximum number of columns in a node grid.
pub(crate) const MAX_GRID_COLS: u32 = 128;
#[cfg(test)]
/// Maximum number of rows in a node grid.
pub(crate) const MAX_GRID_ROWS: u32 = 128;

/// Which pool a node layout belongs to — controls when it appears in a run.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum NodePool {
    /// Default pool — used for non-combat or early-game nodes.
    #[default]
    Passive,
    /// Active pool — higher-difficulty nodes with timers.
    Active,
    /// Boss pool — end-of-tier encounters.
    Boss,
}

/// A node layout loaded from RON. Grid uses nested String arrays.
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub struct NodeLayout {
    /// Display name.
    pub name: String,
    /// Timer duration in seconds for this node.
    pub timer_secs: f32,
    /// Number of columns.
    pub cols: u32,
    /// Number of rows.
    pub rows: u32,
    /// Y offset from playfield top for grid start.
    pub grid_top_offset: f32,
    /// Grid rows — each inner vec is a row, each String is alias or "." (empty).
    pub grid: Vec<Vec<String>>,
    /// Which pool this layout belongs to. Defaults to `Passive` for backward compatibility.
    #[serde(default)]
    pub pool: NodePool,
    /// Scale factor for breaker and bolt entities (0.5..=1.0). Defaults to 1.0.
    #[serde(default = "default_entity_scale")]
    pub entity_scale: f32,
    /// Lock groups: maps each locked cell position `(row, col)` to the positions
    /// of cells that must be destroyed to unlock it.
    /// Absent or `None` means no locks in this layout.
    #[serde(default)]
    pub locks: Option<LockMap>,
}

#[cfg(test)]
/// Minimum allowed entity scale — below this, bolt is visually illegible (~4px).
pub(crate) const MIN_ENTITY_SCALE: f32 = 0.5;
#[cfg(test)]
/// Maximum entity scale — unscaled.
pub(crate) const MAX_ENTITY_SCALE: f32 = 1.0;

/// Default value for `entity_scale` used by serde when the field is absent.
const fn default_entity_scale() -> f32 {
    1.0
}

impl NodeLayout {
    /// Validates that grid dimensions match declared cols/rows and all non-"."
    /// strings exist in the given registry.
    ///
    /// # Errors
    ///
    /// Returns an error string if row/column counts don't match the declared
    /// dimensions or if the grid contains an alias not found in the registry.
    #[cfg(test)]
    pub(crate) fn validate(&self, registry: &CellTypeRegistry) -> Result<(), String> {
        if self.entity_scale < MIN_ENTITY_SCALE || self.entity_scale > MAX_ENTITY_SCALE {
            return Err(format!(
                "layout '{}': entity_scale {} must be {}..={}",
                self.name, self.entity_scale, MIN_ENTITY_SCALE, MAX_ENTITY_SCALE,
            ));
        }
        if self.cols == 0 || self.cols > MAX_GRID_COLS {
            return Err(format!(
                "layout '{}': cols {} must be 1..={}",
                self.name, self.cols, MAX_GRID_COLS,
            ));
        }
        if self.rows == 0 || self.rows > MAX_GRID_ROWS {
            return Err(format!(
                "layout '{}': rows {} must be 1..={}",
                self.name, self.rows, MAX_GRID_ROWS,
            ));
        }
        if self.grid.len() != self.rows as usize {
            return Err(format!(
                "layout '{}': grid has {} rows, expected {}",
                self.name,
                self.grid.len(),
                self.rows,
            ));
        }
        for (i, row) in self.grid.iter().enumerate() {
            if row.len() != self.cols as usize {
                return Err(format!(
                    "layout '{}': row {} has {} cols, expected {}",
                    self.name,
                    i,
                    row.len(),
                    self.cols,
                ));
            }
            for ch in row {
                if ch != "." && !registry.contains(ch) {
                    return Err(format!(
                        "layout '{}': unknown alias '{}' at row {}",
                        self.name, ch, i,
                    ));
                }
            }
        }
        if let Some(ref locks) = self.locks {
            let max_row = self.rows as usize;
            let max_col = self.cols as usize;
            for (&(key_row, key_col), targets) in locks {
                if key_row >= max_row || key_col >= max_col {
                    return Err(format!(
                        "layout '{}': lock key ({}, {}) is out of grid bounds ({}x{})",
                        self.name, key_row, key_col, self.rows, self.cols,
                    ));
                }
                if self.grid[key_row][key_col] == "." {
                    return Err(format!(
                        "layout '{}': lock key ({}, {}) references an empty cell",
                        self.name, key_row, key_col,
                    ));
                }
                for &(target_row, target_col) in targets {
                    if target_row >= max_row || target_col >= max_col {
                        return Err(format!(
                            "layout '{}': lock target ({}, {}) is out of grid bounds ({}x{})",
                            self.name, target_row, target_col, self.rows, self.cols,
                        ));
                    }
                    if self.grid[target_row][target_col] == "." {
                        return Err(format!(
                            "layout '{}': lock target ({}, {}) references an empty cell",
                            self.name, target_row, target_col,
                        ));
                    }
                    if target_row == key_row && target_col == key_col {
                        return Err(format!(
                            "layout '{}': lock at ({}, {}) has self-referencing target at same position",
                            self.name, key_row, key_col,
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    /// Counts non-"." cells in the grid.
    #[must_use]
    #[cfg(test)]
    pub fn cell_count(&self) -> usize {
        self.grid
            .iter()
            .flat_map(|row| row.iter())
            .filter(|ch| ch.as_str() != ".")
            .count()
    }
}
