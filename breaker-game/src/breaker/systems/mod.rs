//! Breaker systems — one file per system function.

pub(crate) mod apply_node_scale_to_breaker;
mod breaker_cell_collision;
mod breaker_wall_collision;
mod bump;
mod bump_feedback;
mod bump_visual;
mod dash;
mod dispatch_breaker_effects;
pub(crate) mod init_breaker;
mod init_breaker_params;
mod move_breaker;
mod spawn_breaker;
mod tilt_visual;
pub(crate) mod width_boost_visual;

pub(crate) use apply_node_scale_to_breaker::apply_node_scale_to_breaker;
pub(crate) use breaker_cell_collision::breaker_cell_collision;
pub(crate) use breaker_wall_collision::breaker_wall_collision;
pub use bump::perfect_bump_dash_cancel;
pub(crate) use bump::{grade_bump, update_bump};
pub use bump_feedback::{spawn_bump_grade_text, spawn_whiff_text};
pub use bump_visual::{animate_bump_visual, trigger_bump_visual};
pub use dash::update_breaker_state;
pub(crate) use dispatch_breaker_effects::dispatch_breaker_effects;
pub(crate) use init_breaker::init_breaker;
pub use init_breaker_params::init_breaker_params;
pub(crate) use move_breaker::move_breaker;
pub use spawn_breaker::{reset_breaker, spawn_breaker};
pub use tilt_visual::animate_tilt_visual;
pub(crate) use width_boost_visual::width_boost_visual;
