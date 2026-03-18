//! System to propagate `CellTypeDefinition` asset changes to live cell entities.

use bevy::prelude::*;

use crate::{
    cells::{
        components::*,
        resources::{CellTypeDefinition, CellTypeRegistry},
    },
    screen::loading::resources::DefaultsCollection,
};

/// Detects `AssetEvent::Modified` on any `CellTypeDefinition`, rebuilds
/// `CellTypeRegistry`, and updates matching live cell entities.
///
/// Updated per-cell: `CellHealth.max` (clamped), `CellDamageVisuals`, material color.
pub(crate) fn propagate_cell_type_changes(
    mut events: MessageReader<AssetEvent<CellTypeDefinition>>,
    collection: Res<DefaultsCollection>,
    assets: Res<Assets<CellTypeDefinition>>,
    mut registry: ResMut<CellTypeRegistry>,
    mut query: Query<
        (
            &CellTypeAlias,
            &mut CellHealth,
            &mut CellDamageVisuals,
            &MeshMaterial2d<ColorMaterial>,
        ),
        With<Cell>,
    >,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Check if any cell type definition was modified
    let any_modified = events.read().any(|event| {
        collection
            .cell_types
            .iter()
            .any(|h| event.is_modified(h.id()))
    });

    if !any_modified {
        return;
    }

    // Rebuild registry from current asset state
    registry.types.clear();
    for handle in &collection.cell_types {
        if let Some(def) = assets.get(handle.id()) {
            registry.types.insert(def.alias, def.clone());
        }
    }

    // Update matching live cell entities
    for (alias, mut health, mut visuals, mat_handle) in &mut query {
        let Some(def) = registry.types.get(&alias.0) else {
            continue;
        };

        health.max = def.hp;
        health.current = health.current.min(def.hp);

        visuals.hdr_base = def.damage_hdr_base;
        visuals.green_min = def.damage_green_min;
        visuals.blue_range = def.damage_blue_range;
        visuals.blue_base = def.damage_blue_base;

        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.color = def.color();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_standard_def() -> CellTypeDefinition {
        CellTypeDefinition {
            id: "standard".to_owned(),
            alias: 'S',
            hp: 1,
            color_rgb: [4.0, 0.2, 0.5],
            required_to_clear: true,
            damage_hdr_base: 4.0,
            damage_green_min: 0.2,
            damage_blue_range: 0.4,
            damage_blue_base: 0.2,
        }
    }

    fn make_tough_def() -> CellTypeDefinition {
        CellTypeDefinition {
            id: "tough".to_owned(),
            alias: 'T',
            hp: 3,
            color_rgb: [2.5, 0.2, 4.0],
            required_to_clear: true,
            damage_hdr_base: 4.0,
            damage_green_min: 0.2,
            damage_blue_range: 0.4,
            damage_blue_base: 0.2,
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<CellTypeDefinition>()
            .init_asset::<ColorMaterial>()
            .init_resource::<CellTypeRegistry>()
            .init_resource::<Assets<ColorMaterial>>()
            .add_systems(Update, propagate_cell_type_changes);
        app
    }

    fn make_collection(cell_types: Vec<Handle<CellTypeDefinition>>) -> DefaultsCollection {
        DefaultsCollection {
            bolt: Handle::default(),
            breaker: Handle::default(),
            cells: Handle::default(),
            playfield: Handle::default(),
            input: Handle::default(),
            mainmenu: Handle::default(),
            timerui: Handle::default(),
            chipselect: Handle::default(),
            cell_types,
            layouts: vec![],
            archetypes: vec![],
            amps: vec![],
            augments: vec![],
            overclocks: vec![],
        }
    }

    #[test]
    fn registry_rebuilt_and_cell_health_max_updated() {
        let mut app = test_app();

        // Add a standard cell type asset
        let def = make_standard_def();
        let handle = {
            let mut assets = app.world_mut().resource_mut::<Assets<CellTypeDefinition>>();
            assets.add(def.clone())
        };

        // Seed the registry manually
        {
            let mut registry = app.world_mut().resource_mut::<CellTypeRegistry>();
            registry.types.insert('S', def);
        }

        // Spawn a cell entity with alias 'S' and health matching old definition
        let material_handle = {
            let mut mats = app.world_mut().resource_mut::<Assets<ColorMaterial>>();
            mats.add(ColorMaterial::from_color(Color::WHITE))
        };
        let entity = app
            .world_mut()
            .spawn((
                Cell,
                CellTypeAlias('S'),
                CellHealth::new(1),
                CellDamageVisuals {
                    hdr_base: 4.0,
                    green_min: 0.2,
                    blue_range: 0.4,
                    blue_base: 0.2,
                },
                MeshMaterial2d(material_handle),
            ))
            .id();

        app.world_mut()
            .insert_resource(make_collection(vec![handle.clone()]));

        // Flush Added event
        app.update();
        app.update();

        // Modify the cell type: increase HP to 5
        {
            let mut assets = app.world_mut().resource_mut::<Assets<CellTypeDefinition>>();
            let asset = assets.get_mut(handle.id()).expect("asset should exist");
            asset.hp = 5;
            asset.damage_hdr_base = 8.0;
        }

        // Flush Modified event
        app.update();
        app.update();

        let health = app.world().get::<CellHealth>(entity).unwrap();
        assert_eq!(health.max, 5, "CellHealth.max should be updated to 5");
        assert_eq!(
            health.current, 1,
            "CellHealth.current should be clamped to new max (but was already <= 5)"
        );

        let visuals = app.world().get::<CellDamageVisuals>(entity).unwrap();
        assert!(
            (visuals.hdr_base - 8.0).abs() < f32::EPSILON,
            "CellDamageVisuals.hdr_base should be updated to 8.0"
        );
    }

    #[test]
    fn cells_with_different_alias_unchanged() {
        let mut app = test_app();

        let s_def = make_standard_def();
        let t_def = make_tough_def();
        let s_handle = {
            let mut assets = app.world_mut().resource_mut::<Assets<CellTypeDefinition>>();
            assets.add(s_def.clone())
        };
        let t_handle = {
            let mut assets = app.world_mut().resource_mut::<Assets<CellTypeDefinition>>();
            assets.add(t_def.clone())
        };

        {
            let mut registry = app.world_mut().resource_mut::<CellTypeRegistry>();
            registry.types.insert('S', s_def);
            registry.types.insert('T', t_def);
        }

        let material_handle = {
            let mut mats = app.world_mut().resource_mut::<Assets<ColorMaterial>>();
            mats.add(ColorMaterial::from_color(Color::WHITE))
        };

        // Spawn a 'T' cell
        let t_entity = app
            .world_mut()
            .spawn((
                Cell,
                CellTypeAlias('T'),
                CellHealth::new(3),
                CellDamageVisuals {
                    hdr_base: 4.0,
                    green_min: 0.2,
                    blue_range: 0.4,
                    blue_base: 0.2,
                },
                MeshMaterial2d(material_handle),
            ))
            .id();

        app.world_mut()
            .insert_resource(make_collection(vec![s_handle.clone(), t_handle]));

        app.update();
        app.update();

        // Modify only 'S' cell type (not 'T')
        {
            let mut assets = app.world_mut().resource_mut::<Assets<CellTypeDefinition>>();
            let asset = assets.get_mut(s_handle.id()).expect("asset should exist");
            asset.hp = 10;
        }

        app.update();
        app.update();

        // 'T' cell should be unchanged
        let health = app.world().get::<CellHealth>(t_entity).unwrap();
        assert_eq!(
            health.max, 3,
            "Tough cell max HP should remain 3 since only standard was modified"
        );
    }

    #[test]
    fn current_health_clamped_to_new_max() {
        let mut app = test_app();

        let def = make_tough_def(); // hp=3
        let handle = {
            let mut assets = app.world_mut().resource_mut::<Assets<CellTypeDefinition>>();
            assets.add(def.clone())
        };

        {
            let mut registry = app.world_mut().resource_mut::<CellTypeRegistry>();
            registry.types.insert('T', def);
        }

        let material_handle = {
            let mut mats = app.world_mut().resource_mut::<Assets<ColorMaterial>>();
            mats.add(ColorMaterial::from_color(Color::WHITE))
        };

        // Spawn cell with current=3, max=3
        let entity = app
            .world_mut()
            .spawn((
                Cell,
                CellTypeAlias('T'),
                CellHealth { current: 3, max: 3 },
                CellDamageVisuals {
                    hdr_base: 4.0,
                    green_min: 0.2,
                    blue_range: 0.4,
                    blue_base: 0.2,
                },
                MeshMaterial2d(material_handle),
            ))
            .id();

        app.world_mut()
            .insert_resource(make_collection(vec![handle.clone()]));

        app.update();
        app.update();

        // Reduce HP from 3 to 1 — current should clamp
        {
            let mut assets = app.world_mut().resource_mut::<Assets<CellTypeDefinition>>();
            let asset = assets.get_mut(handle.id()).expect("asset should exist");
            asset.hp = 1;
        }

        app.update();
        app.update();

        let health = app.world().get::<CellHealth>(entity).unwrap();
        assert_eq!(health.max, 1);
        assert_eq!(
            health.current, 1,
            "current health should be clamped to new max"
        );
    }
}
