//! Screen domain systems.

mod cleanup;
mod loading;
mod main_menu;
mod main_menu_input;

pub use cleanup::cleanup_entities;
pub use loading::{seed_configs_from_defaults, spawn_loading_screen, update_loading_bar};
pub use main_menu::{cleanup_main_menu, spawn_main_menu, update_menu_colors};
pub use main_menu_input::handle_main_menu_input;
