//! Seeds `ArchetypeRegistry` from loaded `ArchetypeDefinition` assets.

use bevy::prelude::*;
use iyes_progress::prelude::*;

use crate::{
    breaker::behaviors::{ArchetypeDefinition, ArchetypeRegistry},
    screen::loading::resources::DefaultsCollection,
};

/// Iterates loaded `ArchetypeDefinition` assets, validates names,
/// and builds the `ArchetypeRegistry` resource.
pub fn seed_archetype_registry(
    collection: Option<Res<DefaultsCollection>>,
    archetype_assets: Res<Assets<ArchetypeDefinition>>,
    mut commands: Commands,
    mut seeded: Local<bool>,
) -> Progress {
    if *seeded {
        return Progress { done: 1, total: 1 };
    }

    let Some(collection) = collection else {
        return Progress { done: 0, total: 1 };
    };

    let mut registry = ArchetypeRegistry::default();
    for handle in &collection.archetypes {
        let Some(def) = archetype_assets.get(handle) else {
            return Progress { done: 0, total: 1 };
        };
        assert!(
            !registry.archetypes.contains_key(&def.name),
            "duplicate archetype name '{}'",
            def.name
        );
        registry.archetypes.insert(def.name.clone(), def.clone());
    }

    commands.insert_resource(registry);
    *seeded = true;
    Progress { done: 1, total: 1 }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<ArchetypeDefinition>();
        app.add_systems(Update, seed_archetype_registry.map(drop));
        app
    }

    fn make_archetype(name: &str) -> ArchetypeDefinition {
        use crate::breaker::behaviors::definition::BreakerStatOverrides;
        ArchetypeDefinition {
            name: name.to_owned(),
            stat_overrides: BreakerStatOverrides::default(),
            life_pool: Some(3),
            behaviors: vec![],
        }
    }

    fn make_collection(archetypes: Vec<Handle<ArchetypeDefinition>>) -> DefaultsCollection {
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
            archetypes,
            upgradeselect: Handle::default(),
        }
    }

    #[test]
    fn returns_zero_progress_without_collection() {
        let mut app = test_app();
        app.update();
        assert!(app.world().get_resource::<ArchetypeRegistry>().is_none());
    }

    #[test]
    fn builds_registry_from_archetypes() {
        let mut app = test_app();

        let mut assets = app
            .world_mut()
            .resource_mut::<Assets<ArchetypeDefinition>>();
        let h1 = assets.add(make_archetype("Aegis"));
        let h2 = assets.add(make_archetype("Flux"));

        app.world_mut()
            .insert_resource(make_collection(vec![h1, h2]));

        app.update();

        let registry = app.world().resource::<ArchetypeRegistry>();
        assert_eq!(registry.archetypes.len(), 2);
        assert!(registry.archetypes.contains_key("Aegis"));
        assert!(registry.archetypes.contains_key("Flux"));
    }

    #[test]
    #[should_panic(expected = "duplicate archetype name")]
    fn panics_on_duplicate_name() {
        let mut app = test_app();

        let mut assets = app
            .world_mut()
            .resource_mut::<Assets<ArchetypeDefinition>>();
        let h1 = assets.add(make_archetype("Aegis"));
        let h2 = assets.add(make_archetype("Aegis"));

        app.world_mut()
            .insert_resource(make_collection(vec![h1, h2]));

        app.update();
    }
}
