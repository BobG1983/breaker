//! Breaker systems — one file per system function.

mod breaker_cell_collision;
mod breaker_wall_collision;
mod bump;
mod bump_feedback;
mod bump_visual;
mod dash;
mod move_breaker;
mod spawn_breaker;
pub(crate) mod sync_breaker_scale;
mod tilt_visual;

pub(crate) use breaker_cell_collision::breaker_cell_collision;
pub(crate) use breaker_wall_collision::breaker_wall_collision;
pub use bump::perfect_bump_dash_cancel;
pub(crate) use bump::{grade_bump, update_bump};
pub use bump_feedback::{spawn_bump_grade_text, spawn_whiff_text};
pub use bump_visual::{animate_bump_visual, trigger_bump_visual};
pub(crate) use dash::update_breaker_state;
pub(crate) use move_breaker::move_breaker;
pub(crate) use spawn_breaker::spawn_or_reuse_breaker;
pub(crate) use sync_breaker_scale::sync_breaker_scale;
pub use tilt_visual::animate_tilt_visual;
