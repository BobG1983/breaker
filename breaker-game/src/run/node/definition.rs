//! Node layout definition — RON-deserialized data for a single node layout.

use bevy::prelude::*;
use serde::Deserialize;

#[cfg(test)]
use crate::cells::CellTypeRegistry;

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

/// A node layout loaded from RON. Grid uses nested char arrays.
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
    /// Grid rows — each inner vec is a row, each char is alias or '.' (empty).
    pub grid: Vec<Vec<char>>,
    /// Which pool this layout belongs to. Defaults to `Passive` for backward compatibility.
    #[serde(default)]
    pub pool: NodePool,
    /// Scale factor for breaker and bolt entities (0.5..=1.0). Defaults to 1.0.
    #[serde(default = "default_entity_scale")]
    pub entity_scale: f32,
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
    /// Validates that grid dimensions match declared cols/rows and all non-'.'
    /// chars exist in the given registry.
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
            for &ch in row {
                if ch != '.' && !registry.contains(ch) {
                    return Err(format!(
                        "layout '{}': unknown alias '{}' at row {}",
                        self.name, ch, i,
                    ));
                }
            }
        }
        Ok(())
    }

    /// Counts non-'.' cells in the grid.
    #[must_use]
    #[cfg(test)]
    pub fn cell_count(&self) -> usize {
        self.grid
            .iter()
            .flat_map(|row| row.iter())
            .filter(|&&ch| ch != '.')
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cells::{CellTypeDefinition, CellTypeRegistry, definition::CellBehavior};

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
                effects: None,
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
                effects: None,
            },
        );
        registry
    }

    #[test]
    fn validate_passes_valid_layout() {
        let layout = NodeLayout {
            name: "test".to_owned(),
            timer_secs: 60.0,
            cols: 3,
            rows: 2,
            grid_top_offset: 50.0,
            grid: vec![vec!['S', 'T', '.'], vec!['.', 'S', 'S']],
            pool: NodePool::default(),
            entity_scale: 1.0,
        };
        let registry = test_registry();
        assert!(layout.validate(&registry).is_ok());
    }

    #[test]
    fn validate_rejects_unknown_alias() {
        let layout = NodeLayout {
            name: "bad".to_owned(),
            timer_secs: 60.0,
            cols: 2,
            rows: 1,
            grid_top_offset: 50.0,
            grid: vec![vec!['X', 'S']],
            pool: NodePool::default(),
            entity_scale: 1.0,
        };
        let registry = test_registry();
        assert!(layout.validate(&registry).is_err());
    }

    #[test]
    fn validate_rejects_wrong_row_count() {
        let layout = NodeLayout {
            name: "bad".to_owned(),
            timer_secs: 60.0,
            cols: 2,
            rows: 3,
            grid_top_offset: 50.0,
            grid: vec![vec!['S', 'S']],
            pool: NodePool::default(),
            entity_scale: 1.0,
        };
        let registry = test_registry();
        assert!(layout.validate(&registry).is_err());
    }

    #[test]
    fn validate_rejects_wrong_col_count() {
        let layout = NodeLayout {
            name: "bad".to_owned(),
            timer_secs: 60.0,
            cols: 2,
            rows: 1,
            grid_top_offset: 50.0,
            grid: vec![vec!['S']],
            pool: NodePool::default(),
            entity_scale: 1.0,
        };
        let registry = test_registry();
        assert!(layout.validate(&registry).is_err());
    }

    #[test]
    fn cell_count_skips_dots() {
        let layout = NodeLayout {
            name: "test".to_owned(),
            timer_secs: 60.0,
            cols: 3,
            rows: 2,
            grid_top_offset: 50.0,
            grid: vec![vec!['S', '.', 'T'], vec!['.', 'S', '.']],
            pool: NodePool::default(),
            entity_scale: 1.0,
        };
        assert_eq!(layout.cell_count(), 3);
    }

    #[test]
    fn deserialize_default_entity_scale_is_one() {
        let ron_str = r#"(name: "test", timer_secs: 60.0, cols: 2, rows: 1, grid_top_offset: 50.0, grid: [['S','S']])"#;
        let layout: NodeLayout = ron::de::from_str(ron_str).expect("should deserialize");
        assert!(
            (layout.entity_scale - 1.0).abs() < f32::EPSILON,
            "entity_scale should default to 1.0, got {}",
            layout.entity_scale,
        );
    }

    #[test]
    fn deserialize_explicit_entity_scale_one() {
        let ron_str = r#"(name: "test", timer_secs: 60.0, cols: 2, rows: 1, grid_top_offset: 50.0, grid: [['S','S']], entity_scale: 1.0)"#;
        let layout: NodeLayout = ron::de::from_str(ron_str).expect("should deserialize");
        assert!(
            (layout.entity_scale - 1.0).abs() < f32::EPSILON,
            "explicit entity_scale: 1.0 should deserialize to 1.0, got {}",
            layout.entity_scale,
        );
    }

    #[test]
    fn deserialize_explicit_entity_scale_custom() {
        let ron_str = r#"(name: "test", timer_secs: 60.0, cols: 2, rows: 1, grid_top_offset: 50.0, grid: [['S','S']], entity_scale: 0.7)"#;
        let layout: NodeLayout = ron::de::from_str(ron_str).expect("should deserialize");
        assert!(
            (layout.entity_scale - 0.7).abs() < f32::EPSILON,
            "entity_scale: 0.7 should deserialize to 0.7, got {}",
            layout.entity_scale,
        );
    }

    #[test]
    fn deserialize_entity_scale_at_minimum() {
        let ron_str = r#"(name: "test", timer_secs: 60.0, cols: 2, rows: 1, grid_top_offset: 50.0, grid: [['S','S']], entity_scale: 0.5)"#;
        let layout: NodeLayout = ron::de::from_str(ron_str).expect("should deserialize");
        assert!(
            (layout.entity_scale - 0.5).abs() < f32::EPSILON,
            "entity_scale: 0.5 should deserialize to 0.5, got {}",
            layout.entity_scale,
        );
    }

    #[test]
    fn validate_rejects_entity_scale_below_minimum() {
        let layout = NodeLayout {
            name: "scale_low".to_owned(),
            timer_secs: 60.0,
            cols: 2,
            rows: 1,
            grid_top_offset: 50.0,
            grid: vec![vec!['S', 'S']],
            pool: NodePool::default(),
            entity_scale: 0.49,
        };
        let registry = test_registry();
        let err = layout
            .validate(&registry)
            .expect_err("entity_scale 0.49 is below MIN_ENTITY_SCALE and must be rejected");
        assert!(
            err.contains("entity_scale"),
            "error message should mention entity_scale, got: {err}",
        );
    }

    #[test]
    fn validate_rejects_entity_scale_zero() {
        let layout = NodeLayout {
            name: "scale_zero".to_owned(),
            timer_secs: 60.0,
            cols: 2,
            rows: 1,
            grid_top_offset: 50.0,
            grid: vec![vec!['S', 'S']],
            pool: NodePool::default(),
            entity_scale: 0.0,
        };
        let registry = test_registry();
        let err = layout
            .validate(&registry)
            .expect_err("entity_scale 0.0 is below MIN_ENTITY_SCALE and must be rejected");
        assert!(
            err.contains("entity_scale"),
            "error message should mention entity_scale, got: {err}",
        );
    }

    #[test]
    fn validate_rejects_entity_scale_above_maximum() {
        let layout = NodeLayout {
            name: "scale_high".to_owned(),
            timer_secs: 60.0,
            cols: 2,
            rows: 1,
            grid_top_offset: 50.0,
            grid: vec![vec!['S', 'S']],
            pool: NodePool::default(),
            entity_scale: 1.01,
        };
        let registry = test_registry();
        let err = layout
            .validate(&registry)
            .expect_err("entity_scale 1.01 is above MAX_ENTITY_SCALE and must be rejected");
        assert!(
            err.contains("entity_scale"),
            "error message should mention entity_scale, got: {err}",
        );
    }

    #[test]
    fn validate_rejects_entity_scale_far_above_maximum() {
        let layout = NodeLayout {
            name: "scale_double".to_owned(),
            timer_secs: 60.0,
            cols: 2,
            rows: 1,
            grid_top_offset: 50.0,
            grid: vec![vec!['S', 'S']],
            pool: NodePool::default(),
            entity_scale: 2.0,
        };
        let registry = test_registry();
        let err = layout
            .validate(&registry)
            .expect_err("entity_scale 2.0 is above MAX_ENTITY_SCALE and must be rejected");
        assert!(
            err.contains("entity_scale"),
            "error message should mention entity_scale, got: {err}",
        );
    }

    #[test]
    fn validate_accepts_entity_scale_at_minimum() {
        let layout = NodeLayout {
            name: "scale_min".to_owned(),
            timer_secs: 60.0,
            cols: 2,
            rows: 1,
            grid_top_offset: 50.0,
            grid: vec![vec!['S', 'S']],
            pool: NodePool::default(),
            entity_scale: 0.5,
        };
        let registry = test_registry();
        assert!(
            layout.validate(&registry).is_ok(),
            "entity_scale 0.5 equals MIN_ENTITY_SCALE and must be accepted",
        );
    }

    #[test]
    fn validate_accepts_entity_scale_at_maximum() {
        let layout = NodeLayout {
            name: "scale_max".to_owned(),
            timer_secs: 60.0,
            cols: 2,
            rows: 1,
            grid_top_offset: 50.0,
            grid: vec![vec!['S', 'S']],
            pool: NodePool::default(),
            entity_scale: 1.0,
        };
        let registry = test_registry();
        assert!(
            layout.validate(&registry).is_ok(),
            "entity_scale 1.0 equals MAX_ENTITY_SCALE and must be accepted",
        );
    }

    #[test]
    fn all_node_rons_parse() {
        use std::fs;
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/assets/nodes");
        for entry in fs::read_dir(dir).expect("assets/nodes/ should exist") {
            let path = entry.unwrap().path();
            if path.extension().and_then(|e| e.to_str()) == Some("ron") {
                let content = fs::read_to_string(&path).unwrap();
                let layout: NodeLayout = ron::de::from_str(&content).unwrap_or_else(|e| {
                    panic!("{}: {e}", path.display());
                });
                let registry = test_registry();
                layout.validate(&registry).unwrap_or_else(|e| {
                    panic!("{}: {e}", path.display());
                });
            }
        }
    }

    #[test]
    fn validate_rejects_cols_above_max() {
        let layout = NodeLayout {
            name: "big_cols".to_owned(),
            timer_secs: 60.0,
            cols: 129,
            rows: 5,
            grid_top_offset: 50.0,
            grid: vec![vec!['S'; 129]; 5],
            pool: NodePool::default(),
            entity_scale: 1.0,
        };
        let registry = test_registry();
        assert!(
            layout.validate(&registry).is_err(),
            "cols=129 exceeds MAX_GRID_COLS={MAX_GRID_COLS} and must be rejected",
        );
    }

    #[test]
    fn validate_rejects_rows_above_max() {
        let layout = NodeLayout {
            name: "big_rows".to_owned(),
            timer_secs: 60.0,
            cols: 5,
            rows: 129,
            grid_top_offset: 50.0,
            grid: vec![vec!['S'; 5]; 129],
            pool: NodePool::default(),
            entity_scale: 1.0,
        };
        let registry = test_registry();
        assert!(
            layout.validate(&registry).is_err(),
            "rows=129 exceeds MAX_GRID_ROWS={MAX_GRID_ROWS} and must be rejected",
        );
    }

    #[test]
    fn validate_rejects_zero_cols() {
        let layout = NodeLayout {
            name: "zero_cols".to_owned(),
            timer_secs: 60.0,
            cols: 0,
            rows: 1,
            grid_top_offset: 50.0,
            grid: vec![vec![]],
            pool: NodePool::default(),
            entity_scale: 1.0,
        };
        let registry = test_registry();
        assert!(
            layout.validate(&registry).is_err(),
            "cols=0 must be rejected",
        );
    }

    #[test]
    fn validate_rejects_zero_rows() {
        let layout = NodeLayout {
            name: "zero_rows".to_owned(),
            timer_secs: 60.0,
            cols: 5,
            rows: 0,
            grid_top_offset: 50.0,
            grid: vec![],
            pool: NodePool::default(),
            entity_scale: 1.0,
        };
        let registry = test_registry();
        assert!(
            layout.validate(&registry).is_err(),
            "rows=0 must be rejected",
        );
    }

    #[test]
    fn validate_accepts_max_dimensions() {
        let layout = NodeLayout {
            name: "max_grid".to_owned(),
            timer_secs: 60.0,
            cols: 128,
            rows: 128,
            grid_top_offset: 50.0,
            grid: vec![vec!['S'; 128]; 128],
            pool: NodePool::default(),
            entity_scale: 1.0,
        };
        let registry = test_registry();
        assert!(
            layout.validate(&registry).is_ok(),
            "128x128 is the maximum valid dimension and must be accepted",
        );
    }

    #[test]
    fn validate_accepts_minimum_dimensions() {
        let layout = NodeLayout {
            name: "tiny".to_owned(),
            timer_secs: 60.0,
            cols: 1,
            rows: 1,
            grid_top_offset: 50.0,
            grid: vec![vec!['S']],
            pool: NodePool::default(),
            entity_scale: 1.0,
        };
        let registry = test_registry();
        assert!(
            layout.validate(&registry).is_ok(),
            "1x1 is the minimum valid dimension and must be accepted",
        );
    }
}
