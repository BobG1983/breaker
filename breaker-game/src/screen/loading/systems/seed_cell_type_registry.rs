//! Seeds `CellTypeRegistry` from loaded `CellTypeDefinition` assets.

use bevy::prelude::*;
use iyes_progress::prelude::*;

use crate::{
    cells::{CellTypeDefinition, CellTypeRegistry},
    screen::loading::resources::DefaultsCollection,
};

/// Iterates loaded `CellTypeDefinition` assets, validates aliases,
/// and builds the `CellTypeRegistry` resource.
pub(crate) fn seed_cell_type_registry(
    collection: Option<Res<DefaultsCollection>>,
    cell_type_assets: Res<Assets<CellTypeDefinition>>,
    mut commands: Commands,
    mut seeded: Local<bool>,
) -> Progress {
    if *seeded {
        return Progress { done: 1, total: 1 };
    }

    let Some(collection) = collection else {
        return Progress { done: 0, total: 1 };
    };

    let mut registry = CellTypeRegistry::default();
    for handle in &collection.cell_types {
        let Some(def) = cell_type_assets.get(handle) else {
            return Progress { done: 0, total: 1 };
        };
        if let Err(e) = def.validate() {
            warn!("Skipping cell type '{}': {e}", def.id);
            continue;
        }
        // Invariant: asset pipeline enforces unique IDs before loading; a duplicate here is a data authoring error, not a recoverable runtime condition.
        assert!(
            def.alias != '.',
            "cell type '{}' uses reserved alias '.'",
            def.id
        );
        // Invariant: asset pipeline enforces unique IDs before loading; a duplicate here is a data authoring error, not a recoverable runtime condition.
        assert!(
            !registry.contains(def.alias),
            "duplicate cell type alias '{}' from '{}'",
            def.alias,
            def.id
        );
        registry.insert(def.alias, def.clone());
    }

    commands.insert_resource(registry);
    *seeded = true;
    Progress { done: 1, total: 1 }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cells::definition::CellBehavior;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<CellTypeDefinition>()
            .add_systems(Update, seed_cell_type_registry.map(drop));
        app
    }

    fn make_cell_type(id: &str, alias: char) -> CellTypeDefinition {
        CellTypeDefinition {
            id: id.to_owned(),
            alias,
            hp: 1.0,
            color_rgb: [1.0, 1.0, 1.0],
            required_to_clear: true,
            damage_hdr_base: 1.0,
            damage_green_min: 0.3,
            damage_blue_range: 0.5,
            damage_blue_base: 0.2,
            behavior: CellBehavior::default(),
        }
    }

    #[test]
    fn returns_zero_progress_without_collection() {
        let mut app = test_app();
        app.update();
        assert!(app.world().get_resource::<CellTypeRegistry>().is_none());
    }

    #[test]
    fn builds_registry_from_cell_types() {
        let mut app = test_app();

        let mut assets = app.world_mut().resource_mut::<Assets<CellTypeDefinition>>();
        let h1 = assets.add(make_cell_type("standard", 'S'));
        let h2 = assets.add(make_cell_type("tough", 'T'));

        app.world_mut().insert_resource(DefaultsCollection {
            playfield: Handle::default(),
            bolt: Handle::default(),
            breaker: Handle::default(),
            cells: Handle::default(),
            input: Handle::default(),
            mainmenu: Handle::default(),
            timerui: Handle::default(),
            cell_types: vec![h1, h2],
            layouts: vec![],
            archetypes: vec![],
            chipselect: Handle::default(),
            amps: vec![],
            augments: vec![],
            overclocks: vec![],
            difficulty: Handle::default(),
            evolutions: vec![],
        });

        app.update();

        let registry = app.world().resource::<CellTypeRegistry>();
        assert_eq!(registry.len(), 2);
        assert!(registry.contains('S'));
        assert!(registry.contains('T'));
    }

    #[test]
    fn skips_invalid_definition_and_keeps_valid_ones() {
        let mut app = test_app();

        let mut invalid_def = make_cell_type("broken", 'B');
        invalid_def.hp = 0.0; // hp <= 0.0 fails validate()

        let mut assets = app.world_mut().resource_mut::<Assets<CellTypeDefinition>>();
        let h_invalid = assets.add(invalid_def);
        let h_before = assets.add(make_cell_type("before", 'A'));
        let h_after = assets.add(make_cell_type("after", 'C'));

        app.world_mut().insert_resource(DefaultsCollection {
            playfield: Handle::default(),
            bolt: Handle::default(),
            breaker: Handle::default(),
            cells: Handle::default(),
            input: Handle::default(),
            mainmenu: Handle::default(),
            timerui: Handle::default(),
            cell_types: vec![h_before, h_invalid, h_after],
            layouts: vec![],
            archetypes: vec![],
            chipselect: Handle::default(),
            amps: vec![],
            augments: vec![],
            overclocks: vec![],
            difficulty: Handle::default(),
            evolutions: vec![],
        });

        app.update();

        let registry = app.world().resource::<CellTypeRegistry>();
        assert_eq!(
            registry.len(),
            2,
            "invalid definition should be skipped, leaving 2 valid entries"
        );
        assert!(
            !registry.contains('B'),
            "invalid cell type 'B' (hp=0.0) should not be in registry"
        );
        assert!(
            registry.contains('A'),
            "valid cell type 'A' before the invalid one should be in registry"
        );
        assert!(
            registry.contains('C'),
            "valid cell type 'C' after the invalid one should be in registry"
        );
    }

    #[test]
    #[should_panic(expected = "reserved alias '.'")]
    fn panics_on_dot_alias() {
        let mut app = test_app();

        let mut assets = app.world_mut().resource_mut::<Assets<CellTypeDefinition>>();
        let handle = assets.add(make_cell_type("bad", '.'));

        app.world_mut().insert_resource(DefaultsCollection {
            playfield: Handle::default(),
            bolt: Handle::default(),
            breaker: Handle::default(),
            cells: Handle::default(),
            input: Handle::default(),
            mainmenu: Handle::default(),
            timerui: Handle::default(),
            cell_types: vec![handle],
            layouts: vec![],
            archetypes: vec![],
            chipselect: Handle::default(),
            amps: vec![],
            augments: vec![],
            overclocks: vec![],
            difficulty: Handle::default(),
            evolutions: vec![],
        });

        app.update();
    }

    #[test]
    #[should_panic(expected = "duplicate cell type alias")]
    fn panics_on_duplicate_alias() {
        let mut app = test_app();

        let mut assets = app.world_mut().resource_mut::<Assets<CellTypeDefinition>>();
        let h1 = assets.add(make_cell_type("first", 'A'));
        let h2 = assets.add(make_cell_type("second", 'A'));

        app.world_mut().insert_resource(DefaultsCollection {
            playfield: Handle::default(),
            bolt: Handle::default(),
            breaker: Handle::default(),
            cells: Handle::default(),
            input: Handle::default(),
            mainmenu: Handle::default(),
            timerui: Handle::default(),
            cell_types: vec![h1, h2],
            layouts: vec![],
            archetypes: vec![],
            chipselect: Handle::default(),
            amps: vec![],
            augments: vec![],
            overclocks: vec![],
            difficulty: Handle::default(),
            evolutions: vec![],
        });

        app.update();
    }
}
