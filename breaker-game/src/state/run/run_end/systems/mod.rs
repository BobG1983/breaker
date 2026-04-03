//! Run end screen systems.

mod detect_most_powerful_evolution;
mod handle_run_end_input;
mod spawn_run_end_screen;

pub(crate) use detect_most_powerful_evolution::detect_most_powerful_evolution;
pub(super) use handle_run_end_input::handle_run_end_input;
pub(super) use spawn_run_end_screen::spawn_run_end_screen;
