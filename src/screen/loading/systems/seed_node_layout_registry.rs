//! Seeds `NodeLayoutRegistry` from loaded `NodeLayout` assets.
//!
//! Depends on `CellTypeRegistry` being available (inserted by
//! `seed_cell_type_registry`). Uses `Option<Res>` to wait until it exists,
//! returning zero progress until then.

use bevy::prelude::*;
use iyes_progress::prelude::*;

use crate::{
    cells::CellTypeRegistry,
    run::{NodeLayout, NodeLayoutRegistry},
    screen::loading::resources::DefaultsCollection,
};

/// Validates loaded `NodeLayout` assets against the `CellTypeRegistry`
/// and builds the `NodeLayoutRegistry` resource.
pub fn seed_node_layout_registry(
    collection: Option<Res<DefaultsCollection>>,
    cell_type_registry: Option<Res<CellTypeRegistry>>,
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

    let Some(registry) = cell_type_registry else {
        return Progress { done: 0, total: 1 };
    };

    let mut node_layout_registry = NodeLayoutRegistry::default();
    for handle in &collection.layouts {
        let Some(layout) = node_layout_assets.get(handle) else {
            return Progress { done: 0, total: 1 };
        };
        if let Err(e) = layout.validate(&registry) {
            panic!("invalid node layout: {e}");
        }
        node_layout_registry.layouts.push(layout.clone());
    }

    commands.insert_resource(node_layout_registry);
    *seeded = true;
    Progress { done: 1, total: 1 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cells::CellTypeDefinition;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<NodeLayout>();
        app.add_systems(Update, seed_node_layout_registry.map(drop));
        app
    }

    fn make_registry() -> CellTypeRegistry {
        let mut registry = CellTypeRegistry::default();
        registry.types.insert(
            'S',
            CellTypeDefinition {
                id: "standard".to_owned(),
                alias: 'S',
                hp: 1,
                color_rgb: [1.0, 1.0, 1.0],
                required_to_clear: true,
                damage_hdr_base: 1.0,
                damage_green_min: 0.3,
                damage_blue_range: 0.5,
                damage_blue_base: 0.2,
            },
        );
        registry
    }

    fn make_layout() -> NodeLayout {
        NodeLayout {
            name: "test".to_owned(),
            timer_secs: 60.0,
            cols: 2,
            rows: 1,
            grid_top_offset: 0.0,
            grid: vec![vec!['S', '.']],
        }
    }

    #[test]
    fn returns_zero_progress_without_collection() {
        let mut app = test_app();
        app.update();
        assert!(app.world().get_resource::<NodeLayoutRegistry>().is_none());
    }

    #[test]
    fn returns_zero_progress_without_cell_type_registry() {
        let mut app = test_app();

        let mut assets = app.world_mut().resource_mut::<Assets<NodeLayout>>();
        let handle = assets.add(make_layout());

        app.world_mut().insert_resource(DefaultsCollection {
            playfield: Handle::default(),
            bolt: Handle::default(),
            breaker: Handle::default(),
            cells: Handle::default(),
            input: Handle::default(),
            mainmenu: Handle::default(),
            timerui: Handle::default(),
            cell_types: vec![],
            layouts: vec![handle],
            archetypes: vec![],
            upgradeselect: Handle::default(),
            amps: vec![],
            augments: vec![],
            overclocks: vec![],
        });

        app.update();

        assert!(app.world().get_resource::<NodeLayoutRegistry>().is_none());
    }

    #[test]
    fn builds_registry_when_dependencies_ready() {
        let mut app = test_app();

        let mut assets = app.world_mut().resource_mut::<Assets<NodeLayout>>();
        let handle = assets.add(make_layout());

        app.world_mut().insert_resource(DefaultsCollection {
            playfield: Handle::default(),
            bolt: Handle::default(),
            breaker: Handle::default(),
            cells: Handle::default(),
            input: Handle::default(),
            mainmenu: Handle::default(),
            timerui: Handle::default(),
            cell_types: vec![],
            layouts: vec![handle],
            archetypes: vec![],
            upgradeselect: Handle::default(),
            amps: vec![],
            augments: vec![],
            overclocks: vec![],
        });
        app.world_mut().insert_resource(make_registry());

        app.update();

        let registry = app.world().resource::<NodeLayoutRegistry>();
        assert_eq!(registry.layouts.len(), 1);
        assert_eq!(registry.layouts[0].name, "test");
    }
}
