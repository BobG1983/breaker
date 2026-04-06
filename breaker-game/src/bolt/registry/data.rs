//! Bolt registry -- maps bolt names to definitions.

use std::collections::HashMap;

use bevy::prelude::*;
use rantzsoft_defaults::prelude::SeedableRegistry;
use tracing::warn;

use super::super::definition::BoltDefinition;

/// Registry of all loaded bolt definitions, keyed by name.
#[derive(Resource, Debug, Default)]
pub struct BoltRegistry {
    /// Map from bolt name to its definition.
    bolts: HashMap<String, BoltDefinition>,
}

impl BoltRegistry {
    /// Returns a reference to the definition for `name`, if it exists.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&BoltDefinition> {
        self.bolts.get(name)
    }

    /// Returns `true` if the registry contains a definition for `name`.
    #[must_use]
    pub fn contains(&self, name: &str) -> bool {
        self.bolts.contains_key(name)
    }

    /// Inserts a definition into the registry under the given `name`.
    pub fn insert(&mut self, name: String, def: BoltDefinition) {
        self.bolts.insert(name, def);
    }

    /// Returns an iterator over all bolt names.
    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.bolts.keys()
    }

    /// Returns an iterator over all `(name, definition)` pairs.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &BoltDefinition)> {
        self.bolts.iter()
    }

    /// Returns an iterator over all definitions.
    pub fn values(&self) -> impl Iterator<Item = &BoltDefinition> {
        self.bolts.values()
    }

    /// Removes all entries from the registry.
    pub fn clear(&mut self) {
        self.bolts.clear();
    }

    /// Returns the number of bolts in the registry.
    #[must_use]
    pub fn len(&self) -> usize {
        self.bolts.len()
    }

    /// Returns `true` if the registry contains no bolts.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bolts.is_empty()
    }
}

impl SeedableRegistry for BoltRegistry {
    type Asset = BoltDefinition;

    fn asset_dir() -> &'static str {
        "bolts"
    }

    fn extensions() -> &'static [&'static str] {
        &["bolt.ron"]
    }

    fn seed(&mut self, assets: &[(AssetId<BoltDefinition>, BoltDefinition)]) {
        self.bolts.clear();
        for (_id, def) in assets {
            if self.bolts.contains_key(&def.name) {
                warn!("duplicate bolt name '{}' — skipping", def.name);
                continue;
            }
            self.bolts.insert(def.name.clone(), def.clone());
        }
    }

    fn update_single(&mut self, _id: AssetId<BoltDefinition>, asset: &BoltDefinition) {
        self.bolts.insert(asset.name.clone(), asset.clone());
    }
}
