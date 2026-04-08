//! System to propagate `CellTypeRegistry` changes to live cell entities.

use bevy::prelude::*;

use crate::cells::{components::*, resources::CellTypeRegistry};

/// Detects when `propagate_registry` has rebuilt the `CellTypeRegistry`
/// and updates matching live cell entities.
///
/// Updated per-cell: `CellHealth.max` (clamped), `CellDamageVisuals`, material color.
pub(crate) fn propagate_cell_type_changes(
    registry: Res<CellTypeRegistry>,
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
    if !registry.is_changed() || registry.is_added() {
        return;
    }

    // Update matching live cell entities
    for (alias, mut health, mut visuals, mat_handle) in &mut query {
        let Some(def) = registry.get(&alias.0) else {
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
    use crate::cells::CellTypeDefinition;

    fn make_standard_def() -> CellTypeDefinition {
        CellTypeDefinition {
            id: "standard".to_owned(),
            alias: "S".to_owned(),
            hp: 1.0,
            color_rgb: [4.0, 0.2, 0.5],
            required_to_clear: true,
            damage_hdr_base: 4.0,
            damage_green_min: 0.2,
            damage_blue_range: 0.4,
            damage_blue_base: 0.2,
            behaviors: None,

            effects: None,
        }
    }

    fn make_tough_def() -> CellTypeDefinition {
        CellTypeDefinition {
            id: "tough".to_owned(),
            alias: "T".to_owned(),
            hp: 3.0,
            color_rgb: [2.5, 0.2, 4.0],
            required_to_clear: true,
            damage_hdr_base: 4.0,
            damage_green_min: 0.2,
            damage_blue_range: 0.4,
            damage_blue_base: 0.2,
            behaviors: None,

            effects: None,
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<ColorMaterial>()
            .init_resource::<CellTypeRegistry>()
            .add_systems(Update, propagate_cell_type_changes);
        app
    }

    #[test]
    fn registry_change_updates_cell_health_max() {
        let mut app = test_app();

        let def = make_standard_def();
        {
            let mut registry = app.world_mut().resource_mut::<CellTypeRegistry>();
            registry.insert("S".to_owned(), def);
        }

        let material_handle = {
            let mut mats = app.world_mut().resource_mut::<Assets<ColorMaterial>>();
            mats.add(ColorMaterial::from_color(Color::WHITE))
        };
        let entity = app
            .world_mut()
            .spawn((
                Cell,
                CellTypeAlias("S".to_owned()),
                CellHealth::new(1.0),
                CellDamageVisuals {
                    hdr_base: 4.0,
                    green_min: 0.2,
                    blue_range: 0.4,
                    blue_base: 0.2,
                },
                MeshMaterial2d(material_handle),
            ))
            .id();

        // Flush Added change detection (system should skip Added)
        app.update();
        app.update();

        // Mutate registry directly — simulates propagate_registry rebuild
        {
            let mut registry = app.world_mut().resource_mut::<CellTypeRegistry>();
            let mut updated_def = make_standard_def();
            updated_def.hp = 5.0;
            updated_def.damage_hdr_base = 8.0;
            registry.insert("S".to_owned(), updated_def);
        }

        app.update();

        let health = app.world().get::<CellHealth>(entity).unwrap();
        assert!(
            (health.max - 5.0).abs() < f32::EPSILON,
            "CellHealth.max should be updated to 5.0"
        );
        assert!(
            (health.current - 1.0).abs() < f32::EPSILON,
            "CellHealth.current should be clamped to new max (but was already <= 5.0)"
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
        {
            let mut registry = app.world_mut().resource_mut::<CellTypeRegistry>();
            registry.insert("S".to_owned(), s_def);
            registry.insert("T".to_owned(), t_def);
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
                CellTypeAlias("T".to_owned()),
                CellHealth::new(3.0),
                CellDamageVisuals {
                    hdr_base: 4.0,
                    green_min: 0.2,
                    blue_range: 0.4,
                    blue_base: 0.2,
                },
                MeshMaterial2d(material_handle),
            ))
            .id();

        // Flush Added
        app.update();
        app.update();

        // Modify only 'S' in the registry (but registry still reports Changed)
        {
            let mut registry = app.world_mut().resource_mut::<CellTypeRegistry>();
            let mut updated_s = make_standard_def();
            updated_s.hp = 10.0;
            registry.insert("S".to_owned(), updated_s);
        }

        app.update();

        // 'T' cell should be unchanged — registry updated 'T' with same values
        let health = app.world().get::<CellHealth>(t_entity).unwrap();
        assert!(
            (health.max - 3.0).abs() < f32::EPSILON,
            "Tough cell max HP should remain 3.0 since only standard was modified"
        );
    }

    #[test]
    fn current_health_clamped_to_new_max() {
        let mut app = test_app();

        let def = make_tough_def(); // hp=3
        {
            let mut registry = app.world_mut().resource_mut::<CellTypeRegistry>();
            registry.insert("T".to_owned(), def);
        }

        let material_handle = {
            let mut mats = app.world_mut().resource_mut::<Assets<ColorMaterial>>();
            mats.add(ColorMaterial::from_color(Color::WHITE))
        };

        // Spawn cell with current=3.0, max=3.0
        let entity = app
            .world_mut()
            .spawn((
                Cell,
                CellTypeAlias("T".to_owned()),
                CellHealth {
                    current: 3.0,
                    max: 3.0,
                },
                CellDamageVisuals {
                    hdr_base: 4.0,
                    green_min: 0.2,
                    blue_range: 0.4,
                    blue_base: 0.2,
                },
                MeshMaterial2d(material_handle),
            ))
            .id();

        // Flush Added
        app.update();
        app.update();

        // Reduce HP from 3.0 to 1.0 — current should clamp
        {
            let mut registry = app.world_mut().resource_mut::<CellTypeRegistry>();
            let mut updated_def = make_tough_def();
            updated_def.hp = 1.0;
            registry.insert("T".to_owned(), updated_def);
        }

        app.update();

        let health = app.world().get::<CellHealth>(entity).unwrap();
        assert!((health.max - 1.0).abs() < f32::EPSILON);
        assert!(
            (health.current - 1.0).abs() < f32::EPSILON,
            "current health should be clamped to new max"
        );
    }
}
