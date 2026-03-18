//! Input recording — captures [`InputActions`] frames to a RON file.
//!
//! When `--record` is passed on the CLI, `RecordingPlugin` appends each frame's
//! non-empty action list to an in-memory buffer. On exit the buffer is serialised
//! to `recordings/recording_<unix_seconds>.scripted.ron`.

mod plugin;
mod resources;
mod systems;

pub(crate) use plugin::RecordingPlugin;
pub(crate) use resources::RecordingConfig;
