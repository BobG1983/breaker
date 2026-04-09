//! Cells domain resources.

use std::collections::HashMap;

use bevy::prelude::*;
use rantzsoft_defaults::{GameConfig, prelude::SeedableRegistry};
use tracing::warn;

use super::super::definition::{CellTypeDefinition, Toughness};

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

/// Toughness scaling configuration resource — HP computation parameters.
#[derive(Resource, Debug, Clone, PartialEq, GameConfig)]
#[game_config(
    defaults = "ToughnessDefaults",
    path = "config/defaults.toughness.ron",
    ext = "toughness.ron"
)]
pub(crate) struct ToughnessConfig {
    /// Base HP for Weak cells.
    pub weak_base: f32,
    /// Base HP for Standard cells.
    pub standard_base: f32,
    /// Base HP for Tough cells.
    pub tough_base: f32,
    /// Exponential multiplier per tier.
    pub tier_multiplier: f32,
    /// Linear multiplier per position within a tier.
    pub node_multiplier: f32,
    /// HP multiplier for boss nodes.
    pub boss_multiplier: f32,
}

impl Default for ToughnessConfig {
    fn default() -> Self {
        Self {
            weak_base: 10.0,
            standard_base: 20.0,
            tough_base: 30.0,
            tier_multiplier: 1.2,
            node_multiplier: 0.05,
            boss_multiplier: 3.0,
        }
    }
}

impl ToughnessConfig {
    /// Returns the base HP for the given toughness level.
    #[must_use]
    pub(crate) const fn base_hp(&self, toughness: Toughness) -> f32 {
        match toughness {
            Toughness::Weak => self.weak_base,
            Toughness::Standard => self.standard_base,
            Toughness::Tough => self.tough_base,
        }
    }

    /// Computes the tier + position scaling factor.
    /// Formula: `tier_multiplier`^`tier` * (1.0 + `node_multiplier` * `position_in_tier`)
    #[must_use]
    pub(crate) fn tier_scale(&self, tier: u32, position_in_tier: u32) -> f32 {
        let tier_i32 = i32::try_from(tier).unwrap_or(i32::MAX);
        let pos_f32 = f32::from(u16::try_from(position_in_tier).unwrap_or(u16::MAX));
        self.tier_multiplier.powi(tier_i32) * self.node_multiplier.mul_add(pos_f32, 1.0)
    }

    /// Full HP computation: `base_hp(toughness)` * `tier_scale(tier, position_in_tier)`.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn hp_for(&self, toughness: Toughness, tier: u32, position_in_tier: u32) -> f32 {
        self.base_hp(toughness) * self.tier_scale(tier, position_in_tier)
    }

    /// Boss HP: `hp_for(...)` * `boss_multiplier`.
    #[cfg(test)]
    #[must_use]
    pub(crate) fn hp_for_boss(
        &self,
        toughness: Toughness,
        tier: u32,
        position_in_tier: u32,
    ) -> f32 {
        self.hp_for(toughness, tier, position_in_tier) * self.boss_multiplier
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
