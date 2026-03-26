//! Seeds `BreakerRegistry` from loaded `BreakerDefinition` assets.

use bevy::prelude::*;
use iyes_progress::prelude::*;

use crate::{
    effect::{BreakerDefinition, BreakerRegistry},
    screen::loading::resources::DefaultsCollection,
};

/// Iterates loaded `BreakerDefinition` assets, validates names,
/// and builds the `BreakerRegistry` resource.
pub(crate) fn seed_breaker_registry(
    collection: Option<Res<DefaultsCollection>>,
    breaker_assets: Res<Assets<BreakerDefinition>>,
    mut commands: Commands,
    mut seeded: Local<bool>,
) -> Progress {
    if *seeded {
        return Progress { done: 1, total: 1 };
    }

    let Some(collection) = collection else {
        return Progress { done: 0, total: 1 };
    };

    let mut registry = BreakerRegistry::default();
    for handle in &collection.breakers {
        let Some(def) = breaker_assets.get(handle) else {
            return Progress { done: 0, total: 1 };
        };
        // Invariant: asset pipeline enforces unique IDs before loading; a duplicate here is a data authoring error, not a recoverable runtime condition.
        assert!(
            !registry.contains(&def.name),
            "duplicate breaker name '{}'",
            def.name
        );
        registry.insert(def.name.clone(), def.clone());
    }

    commands.insert_resource(registry);
    *seeded = true;
    Progress { done: 1, total: 1 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect::definition::BreakerStatOverrides;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<BreakerDefinition>()
            .add_systems(Update, seed_breaker_registry.map(drop));
        app
    }

    fn make_breaker(name: &str) -> BreakerDefinition {
        BreakerDefinition {
            name: name.to_owned(),
            stat_overrides: BreakerStatOverrides::default(),
            life_pool: Some(3),
            effects: vec![],
        }
    }

    fn make_collection(breakers: Vec<Handle<BreakerDefinition>>) -> DefaultsCollection {
        DefaultsCollection {
            playfield: Handle::default(),
            bolt: Handle::default(),
            breaker: Handle::default(),
            cell_defaults: Handle::default(),
            input: Handle::default(),
            main_menu: Handle::default(),
            timer_ui: Handle::default(),
            cells: vec![],
            nodes: vec![],
            breakers,
            chip_select: Handle::default(),
            chips: vec![],
            chip_templates: vec![],
            difficulty: Handle::default(),
        }
    }

    #[test]
    fn returns_zero_progress_without_collection() {
        let mut app = test_app();
        app.update();
        assert!(app.world().get_resource::<BreakerRegistry>().is_none());
    }

    #[test]
    fn builds_registry_from_breakers() {
        let mut app = test_app();

        let mut assets = app.world_mut().resource_mut::<Assets<BreakerDefinition>>();
        let h1 = assets.add(make_breaker("Aegis"));
        let h2 = assets.add(make_breaker("Vortex"));

        app.world_mut()
            .insert_resource(make_collection(vec![h1, h2]));

        app.update();

        let registry = app.world().resource::<BreakerRegistry>();
        assert_eq!(registry.len(), 2);
        assert!(registry.contains("Aegis"));
        assert!(registry.contains("Vortex"));
    }

    #[test]
    #[should_panic(expected = "duplicate breaker name")]
    fn panics_on_duplicate_name() {
        let mut app = test_app();

        let mut assets = app.world_mut().resource_mut::<Assets<BreakerDefinition>>();
        let h1 = assets.add(make_breaker("Aegis"));
        let h2 = assets.add(make_breaker("Aegis"));

        app.world_mut()
            .insert_resource(make_collection(vec![h1, h2]));

        app.update();
    }
}
