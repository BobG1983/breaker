//! Wall registry — maps wall names to definitions.

use std::collections::HashMap;

use bevy::prelude::*;
use rantzsoft_defaults::prelude::SeedableRegistry;
use tracing::warn;

use super::super::definition::WallDefinition;

/// Registry of all loaded wall definitions, keyed by name.
#[derive(Resource, Debug, Default)]
pub struct WallRegistry {
    /// Map from wall name to its definition.
    walls: HashMap<String, WallDefinition>,
}

impl WallRegistry {
    /// Returns a reference to the definition for `name`, if it exists.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&WallDefinition> {
        self.walls.get(name)
    }

    /// Returns `true` if the registry contains a definition for `name`.
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.walls.contains_key(name)
    }

    /// Inserts a definition into the registry under the given `name`.
    ///
    /// If `name` already exists, the existing entry is silently replaced.
    pub fn insert(&mut self, name: String, def: WallDefinition) {
        self.walls.insert(name, def);
    }

    /// Returns an iterator over all wall names.
    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.walls.keys()
    }

    /// Returns an iterator over all `(name, definition)` pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &WallDefinition)> {
        self.walls.iter()
    }

    /// Returns an iterator over all definitions.
    pub fn values(&self) -> impl Iterator<Item = &WallDefinition> {
        self.walls.values()
    }

    /// Removes all entries from the registry.
    pub fn clear(&mut self) {
        self.walls.clear();
    }

    /// Returns the number of walls in the registry.
    #[must_use]
    pub fn len(&self) -> usize {
        self.walls.len()
    }

    /// Returns `true` if the registry contains no walls.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.walls.is_empty()
    }
}

impl SeedableRegistry for WallRegistry {
    type Asset = WallDefinition;

    fn asset_dir() -> &'static str {
        "walls"
    }

    fn extensions() -> &'static [&'static str] {
        &["wall.ron"]
    }

    fn seed(&mut self, assets: &[(AssetId<WallDefinition>, WallDefinition)]) {
        self.walls.clear();
        for (_id, def) in assets {
            if self.walls.contains_key(&def.name) {
                warn!("duplicate wall name '{}' — skipping", def.name);
                continue;
            }
            self.walls.insert(def.name.clone(), def.clone());
        }
    }

    fn update_single(&mut self, _id: AssetId<WallDefinition>, asset: &WallDefinition) {
        self.walls.insert(asset.name.clone(), asset.clone());
    }
}
