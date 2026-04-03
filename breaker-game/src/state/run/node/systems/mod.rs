//! Node subdomain systems — setup + runtime.

// Setup systems moved from bolt/breaker/cells/wall domains
pub(crate) mod apply_node_scale_to_bolt;
pub(crate) mod apply_node_scale_to_breaker;
pub(crate) mod dispatch_cell_effects;
mod reset_bolt;
mod reset_breaker;
pub(crate) mod spawn_walls;

// Node runtime systems
mod apply_time_penalty;
mod check_spawn_complete;
mod init_clear_remaining;
mod init_node_timer;
mod reverse_time_penalty;
mod set_active_layout;
mod spawn_cells_from_layout;
mod tick_node_timer;
mod track_node_completion;

pub(crate) use apply_node_scale_to_bolt::apply_node_scale_to_bolt;
pub(crate) use apply_node_scale_to_breaker::apply_node_scale_to_breaker;
pub use apply_time_penalty::apply_time_penalty;
pub(crate) use check_spawn_complete::check_spawn_complete;
pub(crate) use dispatch_cell_effects::dispatch_cell_effects;
pub use init_clear_remaining::init_clear_remaining;
pub use init_node_timer::init_node_timer;
pub(crate) use reset_bolt::reset_bolt;
pub(crate) use reset_breaker::reset_breaker;
pub use reverse_time_penalty::reverse_time_penalty;
pub use set_active_layout::set_active_layout;
#[cfg(feature = "dev")]
pub(crate) use spawn_cells_from_layout::RenderAssets;
#[cfg(feature = "dev")]
pub(crate) use spawn_cells_from_layout::spawn_cells_from_grid;
pub(crate) use spawn_cells_from_layout::spawn_cells_from_layout;
pub(crate) use spawn_walls::spawn_walls;
pub use tick_node_timer::tick_node_timer;
pub(crate) use track_node_completion::track_node_completion;
