//! Loading screen resources.

use bevy::prelude::*;
use bevy_asset_loader::prelude::*;

use crate::{
    bolt::BoltDefaults,
    breaker::{behaviors::ArchetypeDefinition, BreakerDefaults},
    cells::{CellDefaults, CellTypeDefinition},
    input::InputDefaults,
    run::NodeLayout,
    screen::main_menu::MainMenuDefaults,
    shared::PlayfieldDefaults,
    ui::TimerUiDefaults,
};

/// Asset collection for all defaults — automatically loaded during
/// [`GameState::Loading`] by `bevy_asset_loader`.
#[derive(AssetCollection, Resource)]
pub struct DefaultsCollection {
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
    pub cells: Handle<CellDefaults>,
    /// Handle for input defaults.
    #[asset(path = "config/defaults.input.ron")]
    pub input: Handle<InputDefaults>,
    /// Handle for main menu defaults.
    #[asset(path = "config/defaults.mainmenu.ron")]
    pub mainmenu: Handle<MainMenuDefaults>,
    /// Handle for timer UI defaults.
    #[asset(path = "config/defaults.timerui.ron")]
    pub timerui: Handle<TimerUiDefaults>,
    /// All cell type definition handles.
    #[asset(path = "cells", collection(typed))]
    pub cell_types: Vec<Handle<CellTypeDefinition>>,
    /// All node layout handles.
    #[asset(path = "nodes", collection(typed))]
    pub layouts: Vec<Handle<NodeLayout>>,
    /// All archetype definition handles.
    #[asset(path = "archetypes", collection(typed))]
    pub archetypes: Vec<Handle<ArchetypeDefinition>>,
}
