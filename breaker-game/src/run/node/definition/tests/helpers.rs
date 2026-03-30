//! Shared test helpers for node definition tests.

use crate::cells::{CellTypeDefinition, CellTypeRegistry, definition::CellBehavior};

pub(super) fn test_registry() -> CellTypeRegistry {
    let mut registry = CellTypeRegistry::default();
    registry.insert(
        'S',
        CellTypeDefinition {
            id: "standard".to_owned(),
            alias: 'S',
            hp: 1.0,
            color_rgb: [4.0, 0.2, 0.5],
            required_to_clear: true,
            damage_hdr_base: 4.0,
            damage_green_min: 0.2,
            damage_blue_range: 0.4,
            damage_blue_base: 0.2,
            behavior: CellBehavior::default(),
            effects: None,
        },
    );
    registry.insert(
        'T',
        CellTypeDefinition {
            id: "tough".to_owned(),
            alias: 'T',
            hp: 3.0,
            color_rgb: [2.5, 0.2, 4.0],
            required_to_clear: true,
            damage_hdr_base: 4.0,
            damage_green_min: 0.2,
            damage_blue_range: 0.4,
            damage_blue_base: 0.2,
            behavior: CellBehavior::default(),
            effects: None,
        },
    );
    registry
}
