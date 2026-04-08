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
            shield: None,
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
            shield: None,
            effects: None,
        },
    );
    registry
}
