//! Shared test helpers for node definition tests.

use crate::cells::{CellTypeDefinition, CellTypeRegistry};

pub(super) fn test_registry() -> CellTypeRegistry {
    let mut registry = CellTypeRegistry::default();
    registry.insert(
        "S".to_owned(),
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
        },
    );
    registry.insert(
        "T".to_owned(),
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
        },
    );
    registry.insert(
        "Gu".to_owned(),
        CellTypeDefinition {
            id: "guarded".to_owned(),
            alias: "Gu".to_owned(),
            hp: 30.0,
            color_rgb: [0.8, 0.3, 4.0],
            required_to_clear: true,
            damage_hdr_base: 4.0,
            damage_green_min: 0.2,
            damage_blue_range: 0.4,
            damage_blue_base: 0.2,
            behaviors: Some(vec![crate::cells::definition::CellBehavior::Guarded(
                crate::cells::definition::GuardedBehavior {
                    guardian_hp: 15.0,
                    guardian_color_rgb: [0.5, 0.2, 3.0],
                    slide_speed: 60.0,
                },
            )]),
            effects: None,
        },
    );
    registry.insert(
        "gu".to_owned(),
        CellTypeDefinition {
            id: "guardian".to_owned(),
            alias: "gu".to_owned(),
            hp: 15.0,
            color_rgb: [0.5, 0.2, 3.0],
            required_to_clear: false,
            damage_hdr_base: 3.0,
            damage_green_min: 0.1,
            damage_blue_range: 0.3,
            damage_blue_base: 0.2,
            behaviors: None,
            effects: None,
        },
    );
    registry
}
