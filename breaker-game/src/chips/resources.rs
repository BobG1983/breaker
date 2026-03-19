//! Chip registry — `HashMap` pool of all loaded chip definitions.

use std::collections::HashMap;

use bevy::prelude::*;

use super::definition::ChipDefinition;

/// `HashMap` pool of all loaded chip definitions, keyed by name.
///
/// Preserves insertion order via a separate `Vec<String>` for deterministic
/// iteration (chip offer display). Populated during loading by `seed_chip_registry`.
#[derive(Resource, Debug, Default)]
pub(crate) struct ChipRegistry {
    chips: HashMap<String, ChipDefinition>,
    order: Vec<String>,
}

impl ChipRegistry {
    /// Look up a chip by name.
    #[must_use]
    pub(crate) fn get(&self, name: &str) -> Option<&ChipDefinition> {
        self.chips.get(name)
    }

    /// Iterate all chip definitions in insertion order.
    pub(crate) fn ordered_values(&self) -> impl Iterator<Item = &ChipDefinition> {
        self.order.iter().filter_map(|name| self.chips.get(name))
    }

    /// Number of registered chips.
    #[must_use]
    pub(crate) fn len(&self) -> usize {
        self.chips.len()
    }

    /// Whether the registry is empty.
    #[must_use]
    pub(crate) fn is_empty(&self) -> bool {
        self.chips.is_empty()
    }

    /// Insert a chip definition, keyed by its name.
    pub(crate) fn insert(&mut self, def: ChipDefinition) {
        let name = def.name.clone();
        self.chips.insert(name.clone(), def);
        self.order.push(name);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chips::definition::ChipKind;

    #[test]
    fn default_registry_is_empty() {
        let registry = ChipRegistry::default();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn insert_and_get() {
        let mut registry = ChipRegistry::default();
        registry.insert(ChipDefinition::test_simple("Piercing Shot", ChipKind::Amp));
        assert_eq!(registry.len(), 1);
        assert!(registry.get("Piercing Shot").is_some());
        assert!(registry.get("missing").is_none());
    }

    #[test]
    fn ordered_values_preserves_insertion_order() {
        let mut registry = ChipRegistry::default();
        registry.insert(ChipDefinition::test_simple("C", ChipKind::Overclock));
        registry.insert(ChipDefinition::test_simple("A", ChipKind::Amp));
        registry.insert(ChipDefinition::test_simple("B", ChipKind::Augment));
        let names: Vec<&str> = registry.ordered_values().map(|c| c.name.as_str()).collect();
        assert_eq!(names, vec!["C", "A", "B"]);
    }
}
