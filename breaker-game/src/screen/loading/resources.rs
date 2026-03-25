//! Loading screen resources.

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::{
    behaviors::ArchetypeDefinition,
    bolt::BoltDefaults,
    breaker::BreakerDefaults,
    cells::{CellDefaults, CellTypeDefinition},
    chips::{ChipDefinition, definition::ChipTemplate},
    input::InputDefaults,
    run::{NodeLayout, definition::DifficultyCurveDefaults},
    screen::{chip_select::ChipSelectDefaults, main_menu::MainMenuDefaults},
    shared::PlayfieldDefaults,
    ui::TimerUiDefaults,
};

/// Asset collection for all defaults — automatically loaded during
/// [`GameState::Loading`] by `bevy_asset_loader`.
#[derive(AssetCollection, Resource)]
pub(crate) struct DefaultsCollection {
    /// Handle for playfield defaults.
    #[asset(path = "config/defaults.playfield.ron")]
    pub playfield: Handle<PlayfieldDefaults>,
    /// Handle for bolt defaults.
    #[asset(path = "config/defaults.bolt.ron")]
    pub bolt: Handle<BoltDefaults>,
    /// Handle for breaker defaults.
    #[asset(path = "config/defaults.breaker.ron")]
    pub breaker: Handle<BreakerDefaults>,
    /// Handle for cells defaults.
    #[asset(path = "config/defaults.cells.ron")]
    pub cell_defaults: Handle<CellDefaults>,
    /// Handle for input defaults.
    #[asset(path = "config/defaults.input.ron")]
    pub input: Handle<InputDefaults>,
    /// Handle for main menu defaults.
    #[asset(path = "config/defaults.mainmenu.ron")]
    pub main_menu: Handle<MainMenuDefaults>,
    /// Handle for timer UI defaults.
    #[asset(path = "config/defaults.timerui.ron")]
    pub timer_ui: Handle<TimerUiDefaults>,
    /// All cell type definition handles.
    #[asset(path = "cells", collection(typed))]
    pub cells: Vec<Handle<CellTypeDefinition>>,
    /// All node layout handles.
    #[asset(path = "nodes", collection(typed))]
    pub nodes: Vec<Handle<NodeLayout>>,
    /// All breaker archetype definition handles.
    #[asset(path = "breakers", collection(typed))]
    pub breakers: Vec<Handle<ArchetypeDefinition>>,
    /// Handle for chip select defaults.
    #[asset(path = "config/defaults.chipselect.ron")]
    pub chip_select: Handle<ChipSelectDefaults>,
    /// All chip definitions (evolutions only) — recurses through rarity subdirectories.
    #[asset(path = "chips", collection(typed))]
    pub chips: Vec<Handle<ChipDefinition>>,
    /// All chip template handles (`.chip.ron` files).
    #[asset(path = "chips", collection(typed))]
    pub chip_templates: Vec<Handle<ChipTemplate>>,
    /// Handle for difficulty curve defaults.
    #[asset(path = "config/defaults.difficulty.ron")]
    pub difficulty: Handle<DifficultyCurveDefaults>,
}
