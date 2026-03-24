//! Archetype registry — maps archetype names to definitions.

use std::collections::HashMap;

use bevy::prelude::*;

use super::definition::ArchetypeDefinition;

/// Registry of all loaded archetype definitions, keyed by name.
#[derive(Resource, Debug, Default)]
pub(crate) struct ArchetypeRegistry {
    /// Map from archetype name to its definition.
    archetypes: HashMap<String, ArchetypeDefinition>,
}

impl ArchetypeRegistry {
    /// Returns a reference to the definition for `name`, if it exists.
    pub(crate) fn get(&self, name: &str) -> Option<&ArchetypeDefinition> {
        self.archetypes.get(name)
    }

    /// Returns `true` if the registry contains a definition for `name`.
    pub(crate) fn contains(&self, name: &str) -> bool {
        self.archetypes.contains_key(name)
    }

    /// Inserts a definition into the registry under the given `name`.
    pub(crate) fn insert(&mut self, name: String, def: ArchetypeDefinition) {
        self.archetypes.insert(name, def);
    }

    /// Returns an iterator over all archetype names.
    pub(crate) fn names(&self) -> impl Iterator<Item = &String> {
        self.archetypes.keys()
    }

    /// Returns an iterator over all `(name, definition)` pairs.
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&String, &ArchetypeDefinition)> {
        self.archetypes.iter()
    }

    /// Returns an iterator over all definitions.
    pub(crate) fn values(&self) -> impl Iterator<Item = &ArchetypeDefinition> {
        self.archetypes.values()
    }

    /// Removes all entries from the registry.
    pub(crate) fn clear(&mut self) {
        self.archetypes.clear();
    }

    /// Returns the number of archetypes in the registry.
    pub(crate) fn len(&self) -> usize {
        self.archetypes.len()
    }

    /// Returns `true` if the registry contains no archetypes.
    pub(crate) fn is_empty(&self) -> bool {
        self.archetypes.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_registry_is_empty() {
        let registry = ArchetypeRegistry::default();
        assert!(registry.is_empty());
    }

    #[test]
    fn insert_and_lookup() {
        let mut registry = ArchetypeRegistry::default();
        let ron_str = include_str!("../../assets/breakers/aegis.breaker.ron");
        let def: ArchetypeDefinition = ron::de::from_str(ron_str).expect("aegis RON should parse");
        registry.insert(def.name.clone(), def);
        assert!(registry.contains("Aegis"));
    }
}
