//! Bolt systems — one file per system function.

pub(crate) mod begin_node_birthing;
mod bolt_breaker_collision;
mod bolt_cell_collision;
mod bolt_lost;
mod bolt_lost_feedback;
mod bolt_wall_collision;
mod clamp_bolt_to_playfield;
pub(crate) mod dispatch_bolt_effects;
mod hover_bolt;
mod launch_bolt;
mod normalize_speed_after_constraints;
pub(crate) mod sync_bolt_scale;
pub(crate) mod tick_birthing;
mod tick_bolt_lifespan;

pub(crate) use begin_node_birthing::begin_node_birthing;
pub(crate) use bolt_breaker_collision::bolt_breaker_collision;
pub(crate) use bolt_cell_collision::bolt_cell_collision;
pub(crate) use bolt_lost::bolt_lost;
pub use bolt_lost_feedback::spawn_bolt_lost_text;
pub(crate) use bolt_wall_collision::bolt_wall_collision;
pub(crate) use clamp_bolt_to_playfield::clamp_bolt_to_playfield;
pub(crate) use dispatch_bolt_effects::dispatch_bolt_effects;
pub use hover_bolt::hover_bolt;
pub(crate) use launch_bolt::launch_bolt;
pub(crate) use normalize_speed_after_constraints::normalize_bolt_speed_after_constraints;
pub(crate) use sync_bolt_scale::sync_bolt_scale;
pub(crate) use tick_birthing::tick_birthing;
pub(crate) use tick_bolt_lifespan::tick_bolt_lifespan;
