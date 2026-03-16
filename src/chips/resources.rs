//! Chip registry — flat pool of all loaded chip definitions.

use bevy::prelude::*;

use super::definition::ChipDefinition;

/// Flat pool of all loaded chip definitions.
///
/// The selection screen picks from this pool. Populated during loading
/// by `seed_chip_registry`.
#[derive(Resource, Debug, Default)]
pub struct ChipRegistry {
    /// All available chips (amps + augments + overclocks combined).
    pub chips: Vec<ChipDefinition>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_registry_is_empty() {
        let registry = ChipRegistry::default();
        assert!(registry.chips.is_empty());
    }
}
