//! Loading screen resources.

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::{
    cells::CellTypeDefinition,
    chips::{ChipDefinition, definition::ChipTemplate},
    effect::BreakerDefinition,
    run::{NodeLayout, definition::DifficultyCurveDefaults},
};

/// Asset collection for all defaults — automatically loaded during
/// [`GameState::Loading`] by `bevy_asset_loader`.
#[derive(AssetCollection, Resource)]
pub(crate) struct DefaultsCollection {
    /// All cell type definition handles.
    #[asset(path = "cells", collection(typed))]
    pub cells: Vec<Handle<CellTypeDefinition>>,
    /// All node layout handles.
    #[asset(path = "nodes", collection(typed))]
    pub nodes: Vec<Handle<NodeLayout>>,
    /// All breaker definition handles.
    #[asset(path = "breakers", collection(typed))]
    pub breakers: Vec<Handle<BreakerDefinition>>,
    /// All chip definitions (evolutions only) — only scans the evolution subdirectory.
    #[asset(path = "chips/evolution", collection(typed))]
    pub chips: Vec<Handle<ChipDefinition>>,
    /// All chip template handles (`.chip.ron` files).
    #[asset(path = "chips/templates", collection(typed))]
    pub chip_templates: Vec<Handle<ChipTemplate>>,
    /// Handle for difficulty curve defaults.
    #[asset(path = "config/defaults.difficulty.ron")]
    pub difficulty: Handle<DifficultyCurveDefaults>,
}
