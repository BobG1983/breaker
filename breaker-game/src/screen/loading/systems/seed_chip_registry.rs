//! Seeds `ChipRegistry` from loaded `ChipDefinition` assets.

use bevy::prelude::*;
use iyes_progress::prelude::*;

use crate::{
    chips::{ChipDefinition, ChipRegistry},
    screen::loading::resources::DefaultsCollection,
};

/// Iterates loaded `ChipDefinition` assets from all three collections
/// (amps, augments, overclocks) and builds the `ChipRegistry` resource.
pub(crate) fn seed_chip_registry(
    collection: Option<Res<DefaultsCollection>>,
    chip_assets: Res<Assets<ChipDefinition>>,
    mut commands: Commands,
    mut seeded: Local<bool>,
) -> Progress {
    if *seeded {
        return Progress { done: 1, total: 1 };
    }

    let Some(collection) = collection else {
        return Progress { done: 0, total: 1 };
    };

    let mut registry = ChipRegistry::default();

    let all_handles = collection
        .amps
        .iter()
        .chain(collection.augments.iter())
        .chain(collection.overclocks.iter());

    for handle in all_handles {
        let Some(def) = chip_assets.get(handle) else {
            return Progress { done: 0, total: 1 };
        };
        registry.chips.push(def.clone());
    }

    commands.insert_resource(registry);
    *seeded = true;
    Progress { done: 1, total: 1 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chips::ChipKind;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<ChipDefinition>()
            .add_systems(Update, seed_chip_registry.map(drop));
        app
    }

    fn make_chip(name: &str, kind: ChipKind) -> ChipDefinition {
        ChipDefinition {
            name: name.to_owned(),
            kind,
            description: format!("{name} description"),
        }
    }

    fn make_collection(
        amps: Vec<Handle<ChipDefinition>>,
        augments: Vec<Handle<ChipDefinition>>,
        overclocks: Vec<Handle<ChipDefinition>>,
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
            chipselect: Handle::default(),
            amps,
            augments,
            overclocks,
        }
    }

    #[test]
    fn returns_zero_progress_without_collection() {
        let mut app = test_app();
        app.update();
        assert!(app.world().get_resource::<ChipRegistry>().is_none());
    }

    #[test]
    fn builds_registry_from_all_three_collections() {
        let mut app = test_app();

        let mut assets = app.world_mut().resource_mut::<Assets<ChipDefinition>>();
        let amp = assets.add(make_chip("Piercing Shot", ChipKind::Amp));
        let augment = assets.add(make_chip("Wide Breaker", ChipKind::Augment));
        let overclock = assets.add(make_chip("Surge", ChipKind::Overclock));

        app.world_mut()
            .insert_resource(make_collection(vec![amp], vec![augment], vec![overclock]));

        app.update();

        let registry = app.world().resource::<ChipRegistry>();
        assert_eq!(registry.chips.len(), 3);
        assert_eq!(registry.chips[0].kind, ChipKind::Amp);
        assert_eq!(registry.chips[1].kind, ChipKind::Augment);
        assert_eq!(registry.chips[2].kind, ChipKind::Overclock);
    }

    #[test]
    fn empty_collections_produce_empty_registry() {
        let mut app = test_app();

        app.world_mut()
            .insert_resource(make_collection(vec![], vec![], vec![]));

        app.update();

        let registry = app.world().resource::<ChipRegistry>();
        assert!(registry.chips.is_empty());
    }

    #[test]
    fn only_seeds_once() {
        let mut app = test_app();

        // First update: seed with empty collection
        app.world_mut()
            .insert_resource(make_collection(vec![], vec![], vec![]));
        app.update();

        // Add a chip AFTER seeding — if the guard works, it won't be picked up
        let mut assets = app.world_mut().resource_mut::<Assets<ChipDefinition>>();
        let handle = assets.add(make_chip("Late Addition", ChipKind::Amp));
        app.world_mut()
            .insert_resource(make_collection(vec![handle], vec![], vec![]));
        app.update();

        let registry = app.world().resource::<ChipRegistry>();
        assert!(
            registry.chips.is_empty(),
            "guard should prevent re-seeding; got {} chips",
            registry.chips.len()
        );
    }
}
