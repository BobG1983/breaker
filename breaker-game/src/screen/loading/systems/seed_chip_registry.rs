//! Seeds `ChipRegistry` from loaded `ChipDefinition` assets.

use bevy::prelude::*;
use iyes_progress::prelude::*;

use crate::{
    chips::{
        ChipDefinition, ChipRegistry, Recipe,
        definition::{ChipTemplate, Rarity, expand_template},
    },
    screen::loading::resources::DefaultsCollection,
};

/// Iterates loaded `ChipTemplate` assets from the chip templates collection,
/// expands each into `ChipDefinition`s, and builds the `ChipRegistry` resource.
///
/// Evolution chips are inserted alongside normal chips and also have their
/// recipes extracted into `ChipRegistry::insert_recipe`.
pub(crate) fn seed_chip_registry(
    collection: Option<Res<DefaultsCollection>>,
    chip_assets: Res<Assets<ChipDefinition>>,
    template_assets: Res<Assets<ChipTemplate>>,
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

    // Expand chip templates into definitions
    for handle in &collection.chip_templates {
        let Some(template) = template_assets.get(handle) else {
            return Progress { done: 0, total: 1 };
        };
        for def in expand_template(template) {
            registry.insert(def);
        }
    }

    // Also load ChipDefinition assets (for backward compatibility —
    // remove this block once all non-evolution chips are defined as ChipTemplates.)
    for handle in &collection.chips {
        let Some(def) = chip_assets.get(handle) else {
            return Progress { done: 0, total: 1 };
        };
        if def.rarity == Rarity::Evolution {
            let recipe = Recipe {
                ingredients: def.ingredients.clone().unwrap_or_default(),
                result_name: def.name.clone(),
            };
            registry.insert_recipe(recipe);
        }
        registry.insert(def.clone());
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
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<ChipDefinition>()
            .init_asset::<ChipTemplate>()
            .add_systems(Update, seed_chip_registry.map(drop));
        app
    }

    fn make_collection(
        chips: Vec<Handle<ChipDefinition>>,
        chip_templates: Vec<Handle<ChipTemplate>>,
    ) -> DefaultsCollection {
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
            chips,
            chip_templates,
            difficulty: Handle::default(),
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
        let amp = assets.add(ChipDefinition::test_simple("Piercing Shot"));
        let augment = assets.add(ChipDefinition::test_simple("Wide Breaker"));
        let overclock = assets.add(ChipDefinition::test_simple("Surge"));

        app.world_mut()
            .insert_resource(make_collection(vec![amp, augment, overclock], vec![]));

        app.update();

        let registry = app.world().resource::<ChipRegistry>();
        assert!(registry.get("Piercing Shot").is_some());
        assert!(registry.get("Wide Breaker").is_some());
        assert!(registry.get("Surge").is_some());
        assert_eq!(registry.ordered_values().count(), 3);
    }

    #[test]
    fn empty_collections_produce_empty_registry() {
        let mut app = test_app();

        app.world_mut()
            .insert_resource(make_collection(vec![], vec![]));

        app.update();

        let registry = app.world().resource::<ChipRegistry>();
        assert_eq!(registry.ordered_values().count(), 0);
    }

    #[test]
    fn only_seeds_once() {
        let mut app = test_app();

        // First update: seed with empty collection
        app.world_mut()
            .insert_resource(make_collection(vec![], vec![]));
        app.update();

        // Add a chip AFTER seeding — if the guard works, it won't be picked up
        let mut assets = app.world_mut().resource_mut::<Assets<ChipDefinition>>();
        let handle = assets.add(ChipDefinition::test_simple("Late Addition"));
        app.world_mut()
            .insert_resource(make_collection(vec![handle], vec![]));
        app.update();

        let registry = app.world().resource::<ChipRegistry>();
        assert_eq!(
            registry.ordered_values().count(),
            0,
            "guard should prevent re-seeding"
        );
    }

    // ======================================================================
    // B4 Part C: seed_chip_registry template loading (spec behaviors 10-12)
    // ======================================================================

    /// Helper: build a `ChipTemplate` with given slots.
    fn make_test_template(
        name: &str,
        max_taken: u32,
        common: Option<(&str, Vec<crate::chips::definition::TriggerChain>)>,
        uncommon: Option<(&str, Vec<crate::chips::definition::TriggerChain>)>,
    ) -> ChipTemplate {
        use crate::chips::definition::RaritySlot;
        ChipTemplate {
            name: name.to_owned(),
            max_taken,
            common: common.map(|(prefix, effects)| RaritySlot {
                prefix: prefix.to_owned(),
                effects,
            }),
            uncommon: uncommon.map(|(prefix, effects)| RaritySlot {
                prefix: prefix.to_owned(),
                effects,
            }),
            rare: None,
            legendary: None,
        }
    }

    // --- Behavior 10: seed_chip_registry expands templates into registry ---

    #[test]
    fn seed_chip_registry_expands_templates_into_registry() {
        use crate::chips::definition::TriggerChain;

        let mut app = test_app();

        let template = make_test_template(
            "Piercing",
            3,
            Some((
                "Basic",
                vec![TriggerChain::OnSelected(vec![TriggerChain::Piercing(1)])],
            )),
            Some((
                "Keen",
                vec![TriggerChain::OnSelected(vec![TriggerChain::Piercing(2)])],
            )),
        );
        let mut template_assets = app.world_mut().resource_mut::<Assets<ChipTemplate>>();
        let template_handle = template_assets.add(template);

        app.world_mut()
            .insert_resource(make_collection(vec![], vec![template_handle]));

        app.update();

        let registry = app.world().resource::<ChipRegistry>();
        assert!(
            registry.get("Basic Piercing").is_some(),
            "registry should contain 'Basic Piercing'"
        );
        assert!(
            registry.get("Keen Piercing").is_some(),
            "registry should contain 'Keen Piercing'"
        );
        assert_eq!(registry.ordered_values().count(), 2);
    }

    #[test]
    fn seed_chip_registry_empty_template_adds_zero_entries() {
        let mut app = test_app();

        let template = ChipTemplate {
            name: "Empty".to_owned(),
            max_taken: 1,
            common: None,
            uncommon: None,
            rare: None,
            legendary: None,
        };
        let mut template_assets = app.world_mut().resource_mut::<Assets<ChipTemplate>>();
        let template_handle = template_assets.add(template);

        app.world_mut()
            .insert_resource(make_collection(vec![], vec![template_handle]));

        app.update();

        let registry = app.world().resource::<ChipRegistry>();
        assert_eq!(
            registry.ordered_values().count(),
            0,
            "template with all None slots should add 0 entries"
        );
    }

    // ======================================================================
    // B12d: seed_chip_registry includes evolution chips + extracts recipes
    // ======================================================================

    // --- Behavior 8: seed_chip_registry inserts ALL chip definitions including Evolution rarity ---

    #[test]
    fn seed_chip_registry_inserts_evolution_chips_into_registry() {
        use crate::chips::definition::{EvolutionIngredient, TriggerChain};

        let mut app = test_app();

        // Add a Common chip and an Evolution chip to the chips collection
        let mut chip_assets = app.world_mut().resource_mut::<Assets<ChipDefinition>>();
        let common_handle = chip_assets.add(ChipDefinition {
            name: "Piercing Shot".to_owned(),
            description: "Common chip".to_owned(),
            rarity: Rarity::Common,
            max_stacks: 3,
            effects: vec![TriggerChain::OnSelected(vec![TriggerChain::Piercing(1)])],
            ingredients: None,
            template_name: None,
        });
        let evo_handle = chip_assets.add(ChipDefinition {
            name: "Barrage".to_owned(),
            description: "Evolution chip".to_owned(),
            rarity: Rarity::Evolution,
            max_stacks: 1,
            effects: vec![TriggerChain::Piercing(5)],
            ingredients: Some(vec![EvolutionIngredient {
                chip_name: "Piercing Shot".to_owned(),
                stacks_required: 2,
            }]),
            template_name: None,
        });

        app.world_mut()
            .insert_resource(make_collection(vec![common_handle, evo_handle], vec![]));

        app.update();

        let registry = app.world().resource::<ChipRegistry>();
        assert!(
            registry.get("Barrage").is_some(),
            "evolution chip 'Barrage' should be in the unified ChipRegistry"
        );
        assert!(
            registry.get("Piercing Shot").is_some(),
            "common chip 'Piercing Shot' should be in the unified ChipRegistry"
        );
    }

    #[test]
    fn seed_chip_registry_evolution_only_collection() {
        use crate::chips::definition::TriggerChain;

        let mut app = test_app();

        let mut chip_assets = app.world_mut().resource_mut::<Assets<ChipDefinition>>();
        let evo_handle = chip_assets.add(ChipDefinition {
            name: "Solo Evo".to_owned(),
            description: "Evolution".to_owned(),
            rarity: Rarity::Evolution,
            max_stacks: 1,
            effects: vec![TriggerChain::Piercing(5)],
            ingredients: None,
            template_name: None,
        });

        app.world_mut()
            .insert_resource(make_collection(vec![evo_handle], vec![]));

        app.update();

        let registry = app.world().resource::<ChipRegistry>();
        assert!(
            registry.get("Solo Evo").is_some(),
            "evolution-only collection should have the evolution in registry"
        );
        assert!(
            registry.ordered_values().count() >= 1,
            "registry should have at least 1 entry"
        );
    }

    // --- Behavior 9: seed_chip_registry extracts evolution recipes ---

    #[test]
    fn seed_chip_registry_extracts_recipes_from_evolution_chips() {
        use crate::chips::definition::{EvolutionIngredient, TriggerChain};

        let mut app = test_app();

        let mut chip_assets = app.world_mut().resource_mut::<Assets<ChipDefinition>>();
        let evo_handle = chip_assets.add(ChipDefinition {
            name: "Piercing Barrage".to_owned(),
            description: "Evolution chip".to_owned(),
            rarity: Rarity::Evolution,
            max_stacks: 1,
            effects: vec![TriggerChain::Piercing(5)],
            ingredients: Some(vec![EvolutionIngredient {
                chip_name: "Piercing Shot".to_owned(),
                stacks_required: 2,
            }]),
            template_name: None,
        });

        app.world_mut()
            .insert_resource(make_collection(vec![evo_handle], vec![]));

        app.update();

        let registry = app.world().resource::<ChipRegistry>();
        assert_eq!(
            registry.recipes().len(),
            1,
            "should extract 1 recipe from evolution chip"
        );
        assert_eq!(registry.recipes()[0].result_name, "Piercing Barrage");
        assert_eq!(registry.recipes()[0].ingredients.len(), 1);
        assert_eq!(
            registry.recipes()[0].ingredients[0].chip_name,
            "Piercing Shot"
        );
        assert_eq!(registry.recipes()[0].ingredients[0].stacks_required, 2);
    }

    // --- Behavior 11: non-evolution chips do not produce recipes ---

    #[test]
    fn seed_chip_registry_non_evolution_chips_no_recipes() {
        use crate::chips::definition::{EvolutionIngredient, TriggerChain};

        let mut app = test_app();

        let mut chip_assets = app.world_mut().resource_mut::<Assets<ChipDefinition>>();
        let c1 = chip_assets.add(ChipDefinition {
            name: "Common A".to_owned(),
            description: "Common".to_owned(),
            rarity: Rarity::Common,
            max_stacks: 3,
            effects: vec![TriggerChain::Piercing(1)],
            ingredients: None,
            template_name: None,
        });
        let c2 = chip_assets.add(ChipDefinition {
            name: "Common B".to_owned(),
            description: "Common".to_owned(),
            rarity: Rarity::Common,
            max_stacks: 3,
            effects: vec![TriggerChain::DamageBoost(0.5)],
            ingredients: None,
            template_name: None,
        });
        let evo = chip_assets.add(ChipDefinition {
            name: "Evo Chip".to_owned(),
            description: "Evolution".to_owned(),
            rarity: Rarity::Evolution,
            max_stacks: 1,
            effects: vec![TriggerChain::Piercing(5)],
            ingredients: Some(vec![EvolutionIngredient {
                chip_name: "Common A".to_owned(),
                stacks_required: 2,
            }]),
            template_name: None,
        });

        app.world_mut()
            .insert_resource(make_collection(vec![c1, c2, evo], vec![]));

        app.update();

        let registry = app.world().resource::<ChipRegistry>();
        assert_eq!(
            registry.recipes().len(),
            1,
            "only 1 recipe from the single Evolution chip"
        );
        assert_eq!(
            registry.ordered_values().count(),
            3,
            "all 3 chips (2 common + 1 evolution) should be in registry"
        );
    }

    #[test]
    fn seed_chip_registry_zero_evolution_chips_no_recipes() {
        use crate::chips::definition::TriggerChain;

        let mut app = test_app();

        let mut chip_assets = app.world_mut().resource_mut::<Assets<ChipDefinition>>();
        let c1 = chip_assets.add(ChipDefinition {
            name: "Common A".to_owned(),
            description: "Common".to_owned(),
            rarity: Rarity::Common,
            max_stacks: 3,
            effects: vec![TriggerChain::Piercing(1)],
            ingredients: None,
            template_name: None,
        });

        app.world_mut()
            .insert_resource(make_collection(vec![c1], vec![]));

        app.update();

        let registry = app.world().resource::<ChipRegistry>();
        assert_eq!(
            registry.recipes().len(),
            0,
            "0 evolution chips means 0 recipes"
        );
        assert!(
            registry.get("Common A").is_some(),
            "common chip should still be present"
        );
    }

    // --- Behavior 12: seed_chip_registry is idempotent ---

    #[test]
    fn seed_chip_registry_only_seeds_once_with_templates() {
        use crate::chips::definition::TriggerChain;

        let mut app = test_app();

        // First seeding with 1 template (1 slot = 1 chip)
        let template = make_test_template(
            "Piercing",
            3,
            Some((
                "Basic",
                vec![TriggerChain::OnSelected(vec![TriggerChain::Piercing(1)])],
            )),
            None,
        );
        let mut template_assets = app.world_mut().resource_mut::<Assets<ChipTemplate>>();
        let t1 = template_assets.add(template);

        app.world_mut()
            .insert_resource(make_collection(vec![], vec![t1]));
        app.update();

        // Replace collection with 2 templates and re-update
        let template2 = make_test_template(
            "Damage",
            2,
            Some(("Basic", vec![TriggerChain::DamageBoost(0.5)])),
            None,
        );
        let mut template_assets = app.world_mut().resource_mut::<Assets<ChipTemplate>>();
        let t2 = template_assets.add(template2);

        let template_redo = make_test_template(
            "Piercing",
            3,
            Some((
                "Basic",
                vec![TriggerChain::OnSelected(vec![TriggerChain::Piercing(1)])],
            )),
            None,
        );
        let t1_redo = template_assets.add(template_redo);

        app.world_mut()
            .insert_resource(make_collection(vec![], vec![t1_redo, t2]));
        app.update();

        let registry = app.world().resource::<ChipRegistry>();
        assert_eq!(
            registry.ordered_values().count(),
            1,
            "guard should prevent re-seeding — still only 1 chip from first seed"
        );
    }
}
