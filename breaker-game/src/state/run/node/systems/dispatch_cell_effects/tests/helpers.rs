use bevy::prelude::*;

use super::super::system::dispatch_cell_effects;
use crate::cells::{
    definition::{CellBehavior, CellTypeDefinition},
    resources::CellTypeRegistry,
};

/// Builds a minimal cell type definition with the given id, alias, hp, and effects.
pub(super) fn make_cell_def(
    id: &str,
    alias: char,
    hp: f32,
    effects: Option<Vec<crate::effect::RootEffect>>,
) -> CellTypeDefinition {
    CellTypeDefinition {
        id: id.to_owned(),
        alias,
        hp,
        color_rgb: [1.0, 1.0, 1.0],
        required_to_clear: true,
        damage_hdr_base: 1.0,
        damage_green_min: 0.3,
        damage_blue_range: 0.5,
        damage_blue_base: 0.2,
        behavior: CellBehavior::default(),
        effects,
    }
}

/// Creates a test app with the dispatch system and the given registry.
pub(super) fn test_app(registry: CellTypeRegistry) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(registry);
    app.add_systems(Update, dispatch_cell_effects);
    app
}
