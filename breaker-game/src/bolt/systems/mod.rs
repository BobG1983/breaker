//! Bolt systems — one file per system function.

pub(crate) mod apply_entity_scale_to_bolt;
mod bolt_lost_feedback;
pub(crate) mod bolt_scale_visual;
mod hover_bolt;
mod init_bolt_params;
mod launch_bolt;
mod prepare_bolt_velocity;
mod reset_bolt;
mod spawn_additional_bolt;
mod spawn_bolt;

pub(crate) use apply_entity_scale_to_bolt::apply_entity_scale_to_bolt;
pub use bolt_lost_feedback::spawn_bolt_lost_text;
pub(crate) use bolt_scale_visual::bolt_scale_visual;
pub use hover_bolt::hover_bolt;
pub use init_bolt_params::init_bolt_params;
pub use launch_bolt::launch_bolt;
pub(crate) use prepare_bolt_velocity::prepare_bolt_velocity;
pub(crate) use reset_bolt::reset_bolt;
pub use spawn_additional_bolt::spawn_additional_bolt;
pub(crate) use spawn_bolt::spawn_bolt;
