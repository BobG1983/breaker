//! Upgrade registry — flat pool of all loaded upgrade definitions.

use bevy::prelude::*;

use super::definition::UpgradeDefinition;

/// Flat pool of all loaded upgrade definitions.
///
/// The selection screen picks from this pool. Populated during loading
/// by `seed_upgrade_registry`.
#[derive(Resource, Debug, Default)]
pub struct UpgradeRegistry {
    /// All available upgrades (amps + augments + overclocks combined).
    pub upgrades: Vec<UpgradeDefinition>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_registry_is_empty() {
        let registry = UpgradeRegistry::default();
        assert!(registry.upgrades.is_empty());
    }
}
