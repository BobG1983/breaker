//! Breaker registry — maps breaker names to definitions.

use std::collections::HashMap;

use bevy::prelude::*;

use super::definition::BreakerDefinition;

/// Registry of all loaded breaker definitions, keyed by name.
#[derive(Resource, Debug, Default)]
pub(crate) struct BreakerRegistry {
    /// Map from breaker name to its definition.
    breakers: HashMap<String, BreakerDefinition>,
}

impl BreakerRegistry {
    /// Returns a reference to the definition for `name`, if it exists.
    pub(crate) fn get(&self, name: &str) -> Option<&BreakerDefinition> {
        self.breakers.get(name)
    }

    /// Returns `true` if the registry contains a definition for `name`.
    pub(crate) fn contains(&self, name: &str) -> bool {
        self.breakers.contains_key(name)
    }

    /// Inserts a definition into the registry under the given `name`.
    pub(crate) fn insert(&mut self, name: String, def: BreakerDefinition) {
        self.breakers.insert(name, def);
    }

    /// Returns an iterator over all breaker names.
    pub(crate) fn names(&self) -> impl Iterator<Item = &String> {
        self.breakers.keys()
    }

    /// Returns an iterator over all `(name, definition)` pairs.
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&String, &BreakerDefinition)> {
        self.breakers.iter()
    }

    /// Returns an iterator over all definitions.
    pub(crate) fn values(&self) -> impl Iterator<Item = &BreakerDefinition> {
        self.breakers.values()
    }

    /// Removes all entries from the registry.
    pub(crate) fn clear(&mut self) {
        self.breakers.clear();
    }

    /// Returns the number of breakers in the registry.
    pub(crate) fn len(&self) -> usize {
        self.breakers.len()
    }

    /// Returns `true` if the registry contains no breakers.
    pub(crate) fn is_empty(&self) -> bool {
        self.breakers.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_registry_is_empty() {
        let registry = BreakerRegistry::default();
        assert!(registry.is_empty());
    }

    #[test]
    fn insert_and_lookup() {
        let mut registry = BreakerRegistry::default();
        let ron_str = include_str!("../../assets/breakers/aegis.breaker.ron");
        let def: BreakerDefinition = ron::de::from_str(ron_str).expect("aegis RON should parse");
        registry.insert(def.name.clone(), def);
        assert!(registry.contains("Aegis"));
    }
}
