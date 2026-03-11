//! Bolt systems — one file per system function.

mod apply_bump_velocity;
mod move_bolt;
mod spawn_bolt;

pub use apply_bump_velocity::apply_bump_velocity;
pub use move_bolt::move_bolt;
pub use spawn_bolt::spawn_bolt;
