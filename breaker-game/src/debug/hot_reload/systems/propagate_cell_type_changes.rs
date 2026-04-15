//! System to propagate `CellTypeRegistry` changes to live cell entities.

use bevy::prelude::*;

use crate::{
    cells::{components::*, resources::CellTypeRegistry},
    shared::death_pipeline::hp::Hp,
};

/// Detects when `propagate_registry` has rebuilt the `CellTypeRegistry`
/// and updates matching live cell entities.
///
/// Updated per-cell: `Hp.starting` (and clamps `current`), `CellDamageVisuals`,
/// material color.
pub(crate) fn propagate_cell_type_changes(
    registry: Res<CellTypeRegistry>,
    mut query: Query<
        (
            &CellTypeAlias,
            &mut Hp,
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
    for (alias, mut hp, mut visuals, mat_handle) in &mut query {
        let Some(def) = registry.get(&alias.0) else {
            continue;
        };

        let new_base = def.toughness.default_base_hp();
        hp.starting = new_base;
        hp.current = hp.current.min(new_base);

        visuals.hdr_base = def.damage_hdr_base;
        visuals.green_min = def.damage_green_min;
        visuals.blue_range = def.damage_blue_range;
        visuals.blue_base = def.damage_blue_base;

        if let Some(mat) = materials.get_mut(&mat_handle.0) {
            mat.color = crate::shared::color_from_rgb(def.color_rgb);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cells::{CellTypeDefinition, definition::Toughness};

    fn make_standard_def() -> CellTypeDefinition {
        CellTypeDefinition {
            id:                "standard".to_owned(),
            alias:             "S".to_owned(),
            toughness:         Toughness::default(),
            color_rgb:         [4.0, 0.2, 0.5],
            required_to_clear: true,
            damage_hdr_base:   4.0,
            damage_green_min:  0.2,
            damage_blue_range: 0.4,
            damage_blue_base:  0.2,
            behaviors:         None,

            effects: None,
        }
    }

    fn make_tough_def() -> CellTypeDefinition {
        CellTypeDefinition {
            id:                "tough".to_owned(),
            alias:             "T".to_owned(),
            toughness:         Toughness::Tough,
            color_rgb:         [2.5, 0.2, 4.0],
            required_to_clear: true,
            damage_hdr_base:   4.0,
            damage_green_min:  0.2,
            damage_blue_range: 0.4,
            damage_blue_base:  0.2,
            behaviors:         None,

            effects: None,
        }
    }

    fn test_app() -> App {
        use crate::shared::test_utils::TestAppBuilder;

        let mut app = TestAppBuilder::new()
            .with_resource::<CellTypeRegistry>()
            .with_system(Update, propagate_cell_type_changes)
            .build();
        app.add_plugins(AssetPlugin::default());
        app.init_asset::<ColorMaterial>();
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
                Hp::new(1.0),
                CellDamageVisuals {
                    hdr_base:   4.0,
                    green_min:  0.2,
                    blue_range: 0.4,
                    blue_base:  0.2,
                },
                MeshMaterial2d(material_handle),
            ))
            .id();

        // Flush Added change detection (system should skip Added)
        app.update();
        app.update();

        // Mutate registry: change toughness from Standard (20.0) to Weak (10.0)
        {
            let mut registry = app.world_mut().resource_mut::<CellTypeRegistry>();
            let mut updated_def = make_standard_def();
            updated_def.toughness = Toughness::Weak;
            updated_def.damage_hdr_base = 8.0;
            registry.insert("S".to_owned(), updated_def);
        }

        app.update();

        let hp = app.world().get::<Hp>(entity).unwrap();
        assert!(
            (hp.starting - 10.0).abs() < f32::EPSILON,
            "Hp.starting should update to Weak base 10.0, got {}",
            hp.starting
        );
        assert!(
            (hp.current - 1.0).abs() < f32::EPSILON,
            "Hp.current should remain 1.0 (already <= 10.0)"
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

        // Spawn a 'T' cell with Tough base HP
        let t_entity = app
            .world_mut()
            .spawn((
                Cell,
                CellTypeAlias("T".to_owned()),
                Hp::new(30.0),
                CellDamageVisuals {
                    hdr_base:   4.0,
                    green_min:  0.2,
                    blue_range: 0.4,
                    blue_base:  0.2,
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
            let updated_s = make_standard_def();
            registry.insert("S".to_owned(), updated_s);
        }

        app.update();

        // 'T' cell recalculated from Tough toughness — starting stays 30.0
        let hp = app.world().get::<Hp>(t_entity).unwrap();
        assert!(
            (hp.starting - 30.0).abs() < f32::EPSILON,
            "Tough cell Hp.starting should be Tough base 30.0, got {}",
            hp.starting
        );
    }

    #[test]
    fn current_health_clamped_to_new_max() {
        let mut app = test_app();

        let def = make_tough_def(); // Tough toughness, base 30.0
        {
            let mut registry = app.world_mut().resource_mut::<CellTypeRegistry>();
            registry.insert("T".to_owned(), def);
        }

        let material_handle = {
            let mut mats = app.world_mut().resource_mut::<Assets<ColorMaterial>>();
            mats.add(ColorMaterial::from_color(Color::WHITE))
        };

        // Spawn cell with current=30.0, starting=30.0 (Tough base)
        let entity = app
            .world_mut()
            .spawn((
                Cell,
                CellTypeAlias("T".to_owned()),
                Hp {
                    current:  30.0,
                    starting: 30.0,
                    max:      Some(30.0),
                },
                CellDamageVisuals {
                    hdr_base:   4.0,
                    green_min:  0.2,
                    blue_range: 0.4,
                    blue_base:  0.2,
                },
                MeshMaterial2d(material_handle),
            ))
            .id();

        // Flush Added
        app.update();
        app.update();

        // Change toughness from Tough (30.0) to Weak (10.0) — current should clamp
        {
            let mut registry = app.world_mut().resource_mut::<CellTypeRegistry>();
            let mut updated_def = make_tough_def();
            updated_def.toughness = Toughness::Weak;
            registry.insert("T".to_owned(), updated_def);
        }

        app.update();

        let hp = app.world().get::<Hp>(entity).unwrap();
        assert!(
            (hp.starting - 10.0).abs() < f32::EPSILON,
            "Hp.starting should be Weak base 10.0, got {}",
            hp.starting
        );
        assert!(
            (hp.current - 10.0).abs() < f32::EPSILON,
            "current (30.0) should clamp to new starting (10.0), got {}",
            hp.current
        );
    }
}
