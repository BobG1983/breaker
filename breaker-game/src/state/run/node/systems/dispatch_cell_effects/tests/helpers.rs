use bevy::prelude::*;

use crate::{
    cells::{
        definition::{CellTypeDefinition, Toughness},
        resources::CellTypeRegistry,
    },
    state::run::node::systems::dispatch_cell_effects::system::dispatch_cell_effects,
};

/// Builds a minimal cell type definition with the given id, alias, hp, and effects.
pub(super) fn make_cell_def(
    id: &str,
    alias: &str,
    _hp: f32,
    effects: Option<Vec<crate::effect_v3::types::RootNode>>,
) -> CellTypeDefinition {
    CellTypeDefinition {
        id: id.to_owned(),
        alias: alias.to_owned(),
        toughness: Toughness::default(),
        color_rgb: [1.0, 1.0, 1.0],
        required_to_clear: true,
        damage_hdr_base: 1.0,
        damage_green_min: 0.3,
        damage_blue_range: 0.5,
        damage_blue_base: 0.2,
        behaviors: None,

        effects,
    }
}

/// Creates a test app with the dispatch system and the given registry.
pub(super) fn test_app(registry: CellTypeRegistry) -> App {
    use crate::shared::test_utils::TestAppBuilder;
    TestAppBuilder::new()
        .insert_resource(registry)
        .with_system(Update, dispatch_cell_effects)
        .build()
}
