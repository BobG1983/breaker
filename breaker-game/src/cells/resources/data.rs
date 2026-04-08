//! Cells domain resources.

use std::collections::HashMap;

use bevy::prelude::*;
use rantzsoft_defaults::{GameConfig, prelude::SeedableRegistry};
use tracing::warn;

use super::super::definition::CellTypeDefinition;

/// Cell configuration resource — shared grid layout properties.
#[derive(Resource, Debug, Clone, PartialEq, GameConfig)]
#[game_config(
    defaults = "CellDefaults",
    path = "config/defaults.cells.ron",
    ext = "cells.ron"
)]
pub(crate) struct CellConfig {
    /// Full width of a cell in world units.
    pub width: f32,
    /// Full height of a cell in world units.
    pub height: f32,
    /// Horizontal padding between cells.
    pub padding_x: f32,
    /// Vertical padding between cells.
    pub padding_y: f32,
}

impl Default for CellConfig {
    fn default() -> Self {
        Self {
            width: 70.0,
            height: 24.0,
            padding_x: 4.0,
            padding_y: 4.0,
        }
    }
}

/// Registry mapping alias strings to cell type definitions. Built at boot from all loaded RONs.
#[derive(Resource, Debug, Default)]
pub(crate) struct CellTypeRegistry {
    /// Map from alias string to cell type definition.
    types: HashMap<String, CellTypeDefinition>,
}

impl CellTypeRegistry {
    /// Looks up a cell type definition by its alias string.
    pub(crate) fn get(&self, alias: &str) -> Option<&CellTypeDefinition> {
        self.types.get(alias)
    }

    /// Returns `true` if the registry contains a definition for the given alias.
    #[cfg(test)]
    pub(crate) fn contains(&self, alias: &str) -> bool {
        self.types.contains_key(alias)
    }

    /// Inserts a cell type definition under the given alias, returning the
    /// previous definition if one existed.
    #[cfg(test)]
    pub(crate) fn insert(
        &mut self,
        alias: String,
        def: CellTypeDefinition,
    ) -> Option<CellTypeDefinition> {
        self.types.insert(alias, def)
    }

    /// Returns the number of registered cell types.
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.types.len()
    }
}

impl SeedableRegistry for CellTypeRegistry {
    type Asset = CellTypeDefinition;

    fn asset_dir() -> &'static str {
        "cells"
    }

    fn extensions() -> &'static [&'static str] {
        &["cell.ron"]
    }

    fn seed(&mut self, assets: &[(AssetId<CellTypeDefinition>, CellTypeDefinition)]) {
        self.types.clear();
        for (_id, def) in assets {
            // Reserved alias is a programming error — panic before validate() filters it.
            assert!(def.alias != ".", "reserved alias '{}'", def.alias);
            if def.validate().is_err() {
                warn!(
                    "skipping cell type '{}' (alias '{}'): validation failed",
                    def.id, def.alias
                );
                continue;
            }
            assert!(
                !self.types.contains_key(&def.alias),
                "duplicate cell type alias '{}'",
                def.alias
            );
            self.types.insert(def.alias.clone(), def.clone());
        }
    }

    fn update_single(&mut self, _id: AssetId<CellTypeDefinition>, asset: &CellTypeDefinition) {
        // Reserved alias is a programming error — panic before validate() filters it.
        assert!(asset.alias != ".", "reserved alias '{}'", asset.alias);
        if asset.validate().is_err() {
            warn!(
                "ignoring invalid cell type update '{}' (alias '{}')",
                asset.id, asset.alias
            );
            return;
        }
        self.types.insert(asset.alias.clone(), asset.clone());
    }
}
