//! Screen domain systems.

mod cleanup;
mod loading;
mod start_game;

pub use cleanup::{cleanup_on_node_exit, cleanup_on_run_end};
pub use loading::{
    DefaultsCollection, cleanup_loading_screen, seed_configs_from_defaults, spawn_loading_screen,
    update_loading_bar,
};
pub use start_game::start_game_on_input;
