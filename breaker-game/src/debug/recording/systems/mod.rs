//! Recording systems.

mod capture_frame;
mod write_output;

pub(super) use capture_frame::capture_frame;
pub(super) use write_output::write_recording_on_exit;
