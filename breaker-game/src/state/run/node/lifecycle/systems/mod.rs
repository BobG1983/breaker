//! Node lifecycle systems.

mod handle_node_cleared;
mod handle_run_lost;
mod handle_timer_expired;
mod reset_highlight_tracker;
mod spawn_highlight_text;

pub(crate) use handle_node_cleared::handle_node_cleared;
pub(crate) use handle_run_lost::handle_run_lost;
pub(crate) use handle_timer_expired::handle_timer_expired;
pub(crate) use reset_highlight_tracker::reset_highlight_tracker;
pub(crate) use spawn_highlight_text::spawn_highlight_text;
