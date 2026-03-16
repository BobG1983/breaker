//! Seeds `UpgradeRegistry` from loaded `UpgradeDefinition` assets.

use bevy::prelude::*;
use iyes_progress::prelude::*;

use crate::{
    screen::loading::resources::DefaultsCollection,
    upgrades::{UpgradeDefinition, UpgradeRegistry},
};

/// Iterates loaded `UpgradeDefinition` assets from all three collections
/// (amps, augments, overclocks) and builds the `UpgradeRegistry` resource.
pub fn seed_upgrade_registry(
    collection: Option<Res<DefaultsCollection>>,
    upgrade_assets: Res<Assets<UpgradeDefinition>>,
    mut commands: Commands,
    mut seeded: Local<bool>,
) -> Progress {
    if *seeded {
        return Progress { done: 1, total: 1 };
    }

    let Some(collection) = collection else {
        return Progress { done: 0, total: 1 };
    };

    let mut registry = UpgradeRegistry::default();

    let all_handles = collection
        .amps
        .iter()
        .chain(collection.augments.iter())
        .chain(collection.overclocks.iter());

    for handle in all_handles {
        let Some(def) = upgrade_assets.get(handle) else {
            return Progress { done: 0, total: 1 };
        };
        registry.upgrades.push(def.clone());
    }

    commands.insert_resource(registry);
    *seeded = true;
    Progress { done: 1, total: 1 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::upgrades::UpgradeKind;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<UpgradeDefinition>();
        app.add_systems(Update, seed_upgrade_registry.map(drop));
        app
    }

    fn make_upgrade(name: &str, kind: UpgradeKind) -> UpgradeDefinition {
        UpgradeDefinition {
            name: name.to_owned(),
            kind,
            description: format!("{name} description"),
        }
    }

    fn make_collection(
        amps: Vec<Handle<UpgradeDefinition>>,
        augments: Vec<Handle<UpgradeDefinition>>,
        overclocks: Vec<Handle<UpgradeDefinition>>,
    ) -> DefaultsCollection {
        DefaultsCollection {
            playfield: Handle::default(),
            bolt: Handle::default(),
            breaker: Handle::default(),
            cells: Handle::default(),
            input: Handle::default(),
            mainmenu: Handle::default(),
            timerui: Handle::default(),
            cell_types: vec![],
            layouts: vec![],
            archetypes: vec![],
            upgradeselect: Handle::default(),
            amps,
            augments,
            overclocks,
        }
    }

    #[test]
    fn returns_zero_progress_without_collection() {
        let mut app = test_app();
        app.update();
        assert!(app.world().get_resource::<UpgradeRegistry>().is_none());
    }

    #[test]
    fn builds_registry_from_all_three_collections() {
        let mut app = test_app();

        let mut assets = app.world_mut().resource_mut::<Assets<UpgradeDefinition>>();
        let amp = assets.add(make_upgrade("Piercing Shot", UpgradeKind::Amp));
        let augment = assets.add(make_upgrade("Wide Breaker", UpgradeKind::Augment));
        let overclock = assets.add(make_upgrade("Surge", UpgradeKind::Overclock));

        app.world_mut()
            .insert_resource(make_collection(vec![amp], vec![augment], vec![overclock]));

        app.update();

        let registry = app.world().resource::<UpgradeRegistry>();
        assert_eq!(registry.upgrades.len(), 3);
        assert_eq!(registry.upgrades[0].kind, UpgradeKind::Amp);
        assert_eq!(registry.upgrades[1].kind, UpgradeKind::Augment);
        assert_eq!(registry.upgrades[2].kind, UpgradeKind::Overclock);
    }

    #[test]
    fn empty_collections_produce_empty_registry() {
        let mut app = test_app();

        app.world_mut()
            .insert_resource(make_collection(vec![], vec![], vec![]));

        app.update();

        let registry = app.world().resource::<UpgradeRegistry>();
        assert!(registry.upgrades.is_empty());
    }

    #[test]
    fn only_seeds_once() {
        let mut app = test_app();

        // First update: seed with empty collection
        app.world_mut()
            .insert_resource(make_collection(vec![], vec![], vec![]));
        app.update();

        // Add an upgrade AFTER seeding — if the guard works, it won't be picked up
        let mut assets = app.world_mut().resource_mut::<Assets<UpgradeDefinition>>();
        let handle = assets.add(make_upgrade("Late Addition", UpgradeKind::Amp));
        app.world_mut()
            .insert_resource(make_collection(vec![handle], vec![], vec![]));
        app.update();

        let registry = app.world().resource::<UpgradeRegistry>();
        assert!(
            registry.upgrades.is_empty(),
            "guard should prevent re-seeding; got {} upgrades",
            registry.upgrades.len()
        );
    }
}
