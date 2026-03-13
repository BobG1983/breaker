//! Archetype registry — maps archetype names to definitions.

use std::collections::HashMap;

use bevy::prelude::*;

use super::definition::ArchetypeDefinition;

/// Registry of all loaded archetype definitions, keyed by name.
#[derive(Resource, Debug, Default)]
pub struct ArchetypeRegistry {
    /// Map from archetype name to its definition.
    pub archetypes: HashMap<String, ArchetypeDefinition>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_registry_is_empty() {
        let registry = ArchetypeRegistry::default();
        assert!(registry.archetypes.is_empty());
    }

    #[test]
    fn insert_and_lookup() {
        let mut registry = ArchetypeRegistry::default();
        let ron_str = include_str!("../../../assets/archetypes/aegis.archetype.ron");
        let def: ArchetypeDefinition = ron::de::from_str(ron_str).expect("aegis RON should parse");
        registry.archetypes.insert(def.name.clone(), def);
        assert!(registry.archetypes.contains_key("Aegis"));
    }
}
