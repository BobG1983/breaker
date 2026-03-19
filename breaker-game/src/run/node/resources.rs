//! Node subdomain resources — layout definitions, registry, active layout, timer, and completion tracking.

use bevy::prelude::*;
use serde::Deserialize;

use crate::cells::CellTypeRegistry;

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
}

impl NodeLayout {
    /// Validates that grid dimensions match declared cols/rows and all non-'.'
    /// chars exist in the given registry.
    ///
    /// # Errors
    ///
    /// Returns an error string if row/column counts don't match the declared
    /// dimensions or if the grid contains an alias not found in the registry.
    pub fn validate(&self, registry: &CellTypeRegistry) -> Result<(), String> {
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
                if ch != '.' && !registry.types.contains_key(&ch) {
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

/// The active node layout for the current node.
#[derive(Resource, Debug, Clone)]
pub struct ActiveNodeLayout(pub NodeLayout);

/// Registry of all loaded node layouts.
#[derive(Resource, Debug, Default, Clone)]
pub struct NodeLayoutRegistry {
    /// All loaded layouts, indexed by position.
    pub layouts: Vec<NodeLayout>,
}

impl NodeLayoutRegistry {
    /// Returns the first layout whose name matches `name`, or `None` if not found.
    #[must_use]
    pub fn get_by_name(&self, name: &str) -> Option<&NodeLayout> {
        self.layouts.iter().find(|l| l.name == name)
    }
}

/// Countdown timer for the current node.
#[derive(Resource, Debug, Clone, Default)]
pub struct NodeTimer {
    /// Seconds remaining.
    pub remaining: f32,
    /// Total seconds for this node (used for ratio calculations).
    pub total: f32,
}

/// When set, overrides normal index-based layout selection in `set_active_layout`.
///
/// Set `Some(name)` before entering `GameState::Playing` to force a specific
/// named layout. Used by the scenario runner to drive deterministic test runs.
/// `None` (the default) restores normal index-based selection.
#[derive(Resource, Debug, Default, Clone)]
pub struct ScenarioLayoutOverride(pub Option<String>);

/// Tracks remaining cells that must be cleared for node completion.
#[derive(Resource, Debug, Default)]
pub struct ClearRemainingCount {
    /// Number of `RequiredToClear` cells still alive.
    pub remaining: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cells::CellTypeDefinition;

    fn make_layout(name: &str) -> NodeLayout {
        NodeLayout {
            name: name.to_owned(),
            timer_secs: 60.0,
            cols: 2,
            rows: 1,
            grid_top_offset: 50.0,
            grid: vec![vec!['.', '.']],
        }
    }

    fn test_registry() -> CellTypeRegistry {
        let mut registry = CellTypeRegistry::default();
        registry.types.insert(
            'S',
            CellTypeDefinition {
                id: "standard".to_owned(),
                alias: 'S',
                hp: 1,
                color_rgb: [4.0, 0.2, 0.5],
                required_to_clear: true,
                damage_hdr_base: 4.0,
                damage_green_min: 0.2,
                damage_blue_range: 0.4,
                damage_blue_base: 0.2,
            },
        );
        registry.types.insert(
            'T',
            CellTypeDefinition {
                id: "tough".to_owned(),
                alias: 'T',
                hp: 3,
                color_rgb: [2.5, 0.2, 4.0],
                required_to_clear: true,
                damage_hdr_base: 4.0,
                damage_green_min: 0.2,
                damage_blue_range: 0.4,
                damage_blue_base: 0.2,
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
        };
        let registry = test_registry();
        assert!(layout.validate(&registry).is_err());
    }

    #[test]
    fn get_by_name_returns_layout_with_matching_name() {
        let registry = NodeLayoutRegistry {
            layouts: vec![make_layout("corridor"), make_layout("open")],
        };
        let result = registry.get_by_name("corridor");
        assert!(result.is_some());
        assert_eq!(result.unwrap().name, "corridor");
    }

    #[test]
    fn get_by_name_returns_none_for_missing_name() {
        let registry = NodeLayoutRegistry {
            layouts: vec![make_layout("corridor")],
        };
        assert!(registry.get_by_name("missing").is_none());
    }

    #[test]
    fn get_by_name_on_empty_registry_returns_none() {
        let registry = NodeLayoutRegistry { layouts: vec![] };
        assert!(registry.get_by_name("anything").is_none());
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
        };
        assert_eq!(layout.cell_count(), 3);
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
}
