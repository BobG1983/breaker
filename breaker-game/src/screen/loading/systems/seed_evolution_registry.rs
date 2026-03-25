//! Seeds `EvolutionRegistry` from loaded evolution `ChipDefinition` assets.

use bevy::prelude::*;
use iyes_progress::prelude::*;

use crate::{
    chips::{
        ChipDefinition, EvolutionRegistry,
        definition::{EvolutionRecipe, Rarity},
    },
    screen::loading::resources::DefaultsCollection,
};

/// Iterates loaded evolution `ChipDefinition` assets from the evolutions
/// collection, converts each to an `EvolutionRecipe`, and builds the
/// `EvolutionRegistry` resource.
pub(crate) fn seed_evolution_registry(
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

    let mut registry = EvolutionRegistry::default();

    for handle in &collection.chips {
        let Some(def) = chip_assets.get(handle) else {
            return Progress { done: 0, total: 1 };
        };
        // Only evolution-rarity chips become recipes
        if def.rarity != Rarity::Evolution {
            continue;
        }
        let recipe = EvolutionRecipe {
            ingredients: def.ingredients.clone().unwrap_or_default(),
            result_definition: def.clone(),
        };
        registry.insert(recipe);
    }

    commands.insert_resource(registry);
    *seeded = true;
    Progress { done: 1, total: 1 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chips::definition::{EvolutionIngredient, Rarity, TriggerChain};

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<ChipDefinition>()
            .add_systems(Update, seed_evolution_registry.map(drop));
        app
    }

    fn make_collection(evolutions: Vec<Handle<ChipDefinition>>) -> DefaultsCollection {
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
            breakers: vec![],
            chip_select: Handle::default(),
            chips: evolutions,
            chip_templates: vec![],
            difficulty: Handle::default(),
        }
    }

    /// Build a minimal evolution `ChipDefinition` for testing.
    fn test_evolution_def(name: &str) -> ChipDefinition {
        ChipDefinition {
            name: name.to_owned(),
            description: format!("{name} description"),
            rarity: Rarity::Evolution,
            max_stacks: 1,
            effects: vec![TriggerChain::Piercing(5)],
            ingredients: Some(vec![EvolutionIngredient {
                chip_name: "Test Chip".to_owned(),
                stacks_required: 2,
            }]),
            template_name: None,
        }
    }

    #[test]
    fn returns_zero_progress_without_collection() {
        let mut app = test_app();
        app.update();
        assert!(
            app.world().get_resource::<EvolutionRegistry>().is_none(),
            "EvolutionRegistry should not be inserted without DefaultsCollection"
        );
    }

    #[test]
    fn builds_registry_from_loaded_evolution_recipes() {
        let mut app = test_app();

        let mut assets = app.world_mut().resource_mut::<Assets<ChipDefinition>>();
        let h1 = assets.add(test_evolution_def("Piercing Barrage"));
        let h2 = assets.add(test_evolution_def("Chain Lightning"));

        app.world_mut()
            .insert_resource(make_collection(vec![h1, h2]));

        app.update();

        let registry = app.world().resource::<EvolutionRegistry>();
        assert_eq!(
            registry.recipes().len(),
            2,
            "registry should contain 2 recipes"
        );
    }

    #[test]
    fn empty_evolutions_collection_produces_empty_registry() {
        let mut app = test_app();

        app.world_mut().insert_resource(make_collection(vec![]));

        app.update();

        let registry = app.world().resource::<EvolutionRegistry>();
        assert_eq!(
            registry.recipes().len(),
            0,
            "registry should be empty when no evolution handles provided"
        );
    }

    #[test]
    fn only_seeds_once() {
        let mut app = test_app();

        // First update: seed with empty collection.
        app.world_mut().insert_resource(make_collection(vec![]));
        app.update();

        // Add an evolution AFTER seeding -- if the guard works, it won't be picked up.
        let mut assets = app.world_mut().resource_mut::<Assets<ChipDefinition>>();
        let handle = assets.add(test_evolution_def("Late Addition"));
        app.world_mut()
            .insert_resource(make_collection(vec![handle]));
        app.update();

        let registry = app.world().resource::<EvolutionRegistry>();
        assert_eq!(
            registry.recipes().len(),
            0,
            "guard should prevent re-seeding"
        );
    }
}
