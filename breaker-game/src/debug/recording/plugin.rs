//! `RecordingPlugin` — wires recording systems into the app.

use bevy::prelude::*;

use super::{
    resources::{RecordingBuffer, RecordingFrame},
    systems::{capture_frame, write_recording_on_exit},
};

/// Dev-only plugin that records input actions to a RON file.
///
/// Add this plugin when `--record` is passed on the CLI. It captures each
/// frame's [`crate::input::resources::InputActions`] and writes a
/// `recordings/recording_<timestamp>.scripted.ron` file on exit.
pub struct RecordingPlugin;

impl Plugin for RecordingPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RecordingBuffer>()
            .init_resource::<RecordingFrame>()
            .add_systems(FixedUpdate, capture_frame)
            .add_systems(Last, write_recording_on_exit);
    }
}
