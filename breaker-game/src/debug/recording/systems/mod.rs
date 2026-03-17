//! Recording systems.

mod capture_frame;
mod write_output;

pub use capture_frame::capture_frame;
pub use write_output::write_recording_on_exit;
