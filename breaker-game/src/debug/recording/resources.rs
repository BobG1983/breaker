//! Resources for the input recording system.

use bevy::prelude::*;

use crate::input::resources::GameAction;

/// Configuration inserted by `main.rs` when `--record` is passed.
#[derive(Resource, Debug, Default)]
pub struct RecordingConfig {
    /// Whether recording is active.
    pub enabled: bool,
    /// If set, only record while `ActiveNodeLayout.name` equals this string.
    pub level_filter: Option<String>,
}

/// A single recorded frame — frame index and the actions that were active.
#[derive(Debug, Clone)]
pub struct RecordedFrame {
    /// Fixed-update frame index.
    pub frame: u32,
    /// Actions that were active on this frame.
    pub actions: Vec<GameAction>,
}

/// Accumulates recorded frames during the run.
#[derive(Resource, Default)]
pub struct RecordingBuffer(pub Vec<RecordedFrame>);

/// Monotonic fixed-update frame counter for recording.
#[derive(Resource, Default)]
pub struct RecordingFrame(pub u32);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recording_config_default_disabled() {
        let config = RecordingConfig::default();
        assert!(!config.enabled);
        assert!(config.level_filter.is_none());
    }

    #[test]
    fn recording_buffer_default_is_empty() {
        let buffer = RecordingBuffer::default();
        assert!(buffer.0.is_empty());
    }

    #[test]
    fn recording_frame_default_is_zero() {
        let frame = RecordingFrame::default();
        assert_eq!(frame.0, 0);
    }
}
