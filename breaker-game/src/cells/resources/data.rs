//! Cells domain resources.

use std::collections::HashMap;

use bevy::prelude::*;
use rantzsoft_defaults::{GameConfig, prelude::SeedableRegistry};

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

/// Registry mapping alias chars to cell type definitions. Built at boot from all loaded RONs.
#[derive(Resource, Debug, Default)]
pub(crate) struct CellTypeRegistry {
    /// Map from alias char to cell type definition.
    types: HashMap<char, CellTypeDefinition>,
}

impl CellTypeRegistry {
    /// Looks up a cell type definition by its alias char.
    pub(crate) fn get(&self, alias: char) -> Option<&CellTypeDefinition> {
        self.types.get(&alias)
    }

    /// Returns `true` if the registry contains a definition for the given alias.
    #[cfg(test)]
    pub(crate) fn contains(&self, alias: char) -> bool {
        self.types.contains_key(&alias)
    }

    /// Inserts a cell type definition under the given alias, returning the
    /// previous definition if one existed.
    #[cfg(test)]
    pub(crate) fn insert(
        &mut self,
        alias: char,
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
            if let Err(e) = def.validate() {
                warn!("Skipping cell type '{}': {e}", def.id);
                continue;
            }
            assert!(
                def.alias != '.',
                "cell type '{}' uses reserved alias '.'",
                def.id
            );
            assert!(
                !self.types.contains_key(&def.alias),
                "duplicate cell type alias '{}' from '{}'",
                def.alias,
                def.id
            );
            self.types.insert(def.alias, def.clone());
        }
    }

    fn update_single(&mut self, _id: AssetId<CellTypeDefinition>, asset: &CellTypeDefinition) {
        if let Err(e) = asset.validate() {
            warn!("Skipping cell type '{}': {e}", asset.id);
            return;
        }
        assert!(
            asset.alias != '.',
            "cell type '{}' uses reserved alias '.'",
            asset.id
        );
        self.types.insert(asset.alias, asset.clone());
    }
}
