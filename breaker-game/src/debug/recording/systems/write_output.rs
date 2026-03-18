//! System that serialises the recording buffer to a RON file on `AppExit`.

use std::{
    fmt::Write as _,
    io::Write as IoWrite,
    time::{SystemTime, UNIX_EPOCH},
};

use bevy::{ecs::message::Messages, prelude::*};
use tracing::{info, warn};

use crate::{
    debug::recording::resources::{RecordedFrame, RecordingBuffer, RecordingConfig},
    input::resources::GameAction,
};

/// Writes the recording buffer to `recordings/recording_<unix_secs>.scripted.ron`.
///
/// Runs in the `Last` schedule every frame, but only writes when an [`AppExit`]
/// message is present and the buffer is non-empty.
pub(crate) fn write_recording_on_exit(
    config: Res<RecordingConfig>,
    buffer: Res<RecordingBuffer>,
    exit_messages: Res<Messages<AppExit>>,
) {
    if exit_messages
        .iter_current_update_messages()
        .next()
        .is_none()
    {
        return;
    }

    if !config.enabled || buffer.0.is_empty() {
        return;
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());

    let dir = std::path::PathBuf::from("recordings");
    if let Err(e) = std::fs::create_dir_all(&dir) {
        warn!("RecordingPlugin: could not create recordings/ dir: {e}");
        return;
    }

    let path = dir.join(format!("recording_{timestamp}.scripted.ron"));
    let ron_str = serialise_buffer(&buffer.0);

    match std::fs::File::create(&path) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(ron_str.as_bytes()) {
                warn!("RecordingPlugin: failed to write recording: {e}");
            } else {
                info!(
                    "RecordingPlugin: saved {} frames to {}",
                    buffer.0.len(),
                    path.display()
                );
            }
        }
        Err(e) => {
            warn!("RecordingPlugin: could not create {}: {e}", path.display());
        }
    }
}

/// Serialises the buffer into a `ScriptedInput`-compatible RON string.
#[must_use]
pub(super) fn serialise_buffer(frames: &[RecordedFrame]) -> String {
    let mut out = String::from("Scripted(actions: [\n");
    for entry in frames {
        let _ = write!(out, "    (frame: {}, actions: [", entry.frame);
        let action_strs: Vec<&str> = entry.actions.iter().copied().map(action_name).collect();
        out.push_str(&action_strs.join(", "));
        out.push_str("]),\n");
    }
    out.push_str("])");
    out
}

const fn action_name(action: GameAction) -> &'static str {
    match action {
        GameAction::MoveLeft => "MoveLeft",
        GameAction::MoveRight => "MoveRight",
        GameAction::Bump => "Bump",
        GameAction::DashLeft => "DashLeft",
        GameAction::DashRight => "DashRight",
        GameAction::MenuUp => "MenuUp",
        GameAction::MenuDown => "MenuDown",
        GameAction::MenuLeft => "MenuLeft",
        GameAction::MenuRight => "MenuRight",
        GameAction::MenuConfirm => "MenuConfirm",
        GameAction::TogglePause => "TogglePause",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialise_buffer_produces_scripted_input_ron_format() {
        let frames = vec![
            RecordedFrame {
                frame: 42,
                actions: vec![GameAction::MoveLeft],
            },
            RecordedFrame {
                frame: 43,
                actions: vec![GameAction::MoveLeft, GameAction::Bump],
            },
        ];
        let ron = serialise_buffer(&frames);
        assert!(ron.starts_with("Scripted(actions: ["));
        assert!(ron.contains("(frame: 42, actions: [MoveLeft])"));
        assert!(ron.contains("(frame: 43, actions: [MoveLeft, Bump])"));
        assert!(ron.ends_with("])"));
    }

    #[test]
    fn serialise_buffer_empty_produces_empty_actions_list() {
        let ron = serialise_buffer(&[]);
        assert_eq!(ron, "Scripted(actions: [\n])");
    }

    #[test]
    fn serialised_output_has_correct_variant_and_field_structure() {
        let frames = vec![
            RecordedFrame {
                frame: 10,
                actions: vec![GameAction::Bump],
            },
            RecordedFrame {
                frame: 20,
                actions: vec![GameAction::MoveRight],
            },
        ];
        let ron = serialise_buffer(&frames);

        // Must start with the `Scripted` variant and actions field
        assert!(ron.starts_with("Scripted(actions: ["));
        // Must contain both frames
        assert!(ron.contains("(frame: 10, actions: [Bump])"));
        assert!(ron.contains("(frame: 20, actions: [MoveRight])"));
        // Must close the list
        assert!(ron.ends_with("])"));
    }
}
