//! Config seeding system — reads loaded defaults assets and inserts config resources.

use bevy::prelude::*;
use iyes_progress::prelude::*;

use crate::{
    bolt::{BoltConfig, BoltDefaults},
    breaker::{BreakerConfig, BreakerDefaults},
    cells::{CellConfig, CellDefaults, CellTypeDefinition, CellTypeRegistry},
    input::{InputConfig, InputDefaults},
    run::{NodeLayout, NodeLayoutRegistry},
    screen::{
        loading::resources::DefaultsCollection,
        main_menu::{MainMenuConfig, MainMenuDefaults},
    },
    shared::{PlayfieldConfig, PlayfieldDefaults},
};

/// Reads loaded `*Defaults` assets and inserts the corresponding `*Config`
/// resources. Also builds `CellTypeRegistry` and `NodeLayoutRegistry` from
/// loaded asset collections. Returns [`Progress`] to block the loading state
/// transition until seeding is complete.
#[allow(clippy::too_many_arguments)]
pub fn seed_configs_from_defaults(
    collection: Option<Res<DefaultsCollection>>,
    playfield_assets: Res<Assets<PlayfieldDefaults>>,
    bolt_assets: Res<Assets<BoltDefaults>>,
    breaker_assets: Res<Assets<BreakerDefaults>>,
    cell_assets: Res<Assets<CellDefaults>>,
    input_assets: Res<Assets<InputDefaults>>,
    mainmenu_assets: Res<Assets<MainMenuDefaults>>,
    cell_type_assets: Res<Assets<CellTypeDefinition>>,
    node_layout_assets: Res<Assets<NodeLayout>>,
    mut commands: Commands,
    mut seeded: Local<bool>,
) -> Progress {
    if *seeded {
        return Progress { done: 1, total: 1 };
    }

    let Some(collection) = collection else {
        return Progress { done: 0, total: 1 };
    };

    // All assets must be loaded before we can seed
    let Some(playfield) = playfield_assets.get(&collection.playfield) else {
        return Progress { done: 0, total: 1 };
    };
    let Some(bolt) = bolt_assets.get(&collection.bolt) else {
        return Progress { done: 0, total: 1 };
    };
    let Some(breaker) = breaker_assets.get(&collection.breaker) else {
        return Progress { done: 0, total: 1 };
    };
    let Some(cells) = cell_assets.get(&collection.cells) else {
        return Progress { done: 0, total: 1 };
    };
    let Some(input) = input_assets.get(&collection.input) else {
        return Progress { done: 0, total: 1 };
    };
    let Some(mainmenu) = mainmenu_assets.get(&collection.mainmenu) else {
        return Progress { done: 0, total: 1 };
    };

    // Build CellTypeRegistry from loaded cell type definitions
    let mut cell_type_registry = CellTypeRegistry::default();
    for handle in &collection.cell_types {
        let Some(def) = cell_type_assets.get(handle) else {
            return Progress { done: 0, total: 1 };
        };
        assert!(
            def.alias != '.',
            "cell type '{}' uses reserved alias '.'",
            def.id
        );
        assert!(
            !cell_type_registry.types.contains_key(&def.alias),
            "duplicate cell type alias '{}' from '{}'",
            def.alias,
            def.id
        );
        cell_type_registry.types.insert(def.alias, def.clone());
    }

    // Build NodeLayoutRegistry from loaded node layouts
    let mut node_layout_registry = NodeLayoutRegistry::default();
    for handle in &collection.layouts {
        let Some(layout) = node_layout_assets.get(handle) else {
            return Progress { done: 0, total: 1 };
        };
        if let Err(e) = layout.validate(&cell_type_registry) {
            panic!("invalid node layout: {e}");
        }
        node_layout_registry.layouts.push(layout.clone());
    }

    commands.insert_resource::<PlayfieldConfig>(playfield.clone().into());
    commands.insert_resource::<BoltConfig>(bolt.clone().into());
    commands.insert_resource::<BreakerConfig>(breaker.clone().into());
    commands.insert_resource::<CellConfig>(cells.clone().into());
    commands.insert_resource::<InputConfig>(input.clone().into());
    commands.insert_resource::<MainMenuConfig>(mainmenu.clone().into());
    commands.insert_resource(cell_type_registry);
    commands.insert_resource(node_layout_registry);

    *seeded = true;
    Progress { done: 1, total: 1 }
}
