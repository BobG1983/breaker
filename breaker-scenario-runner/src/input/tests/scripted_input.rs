use super::super::drivers::*;
use crate::types::{GameAction, ScriptedFrame, ScriptedParams};

// -------------------------------------------------------------------------
// ScriptedInput — fires at exact frame numbers
// -------------------------------------------------------------------------

/// `ScriptedInput` fires the configured actions only on the matching frames.
///
/// Frame 5 returns `[MoveLeft]`; frame 10 returns `[Bump, MoveRight]`;
/// all other frames 0..15 return empty `Vec`.
#[test]
fn scripted_input_fires_at_exact_frame_numbers() {
    let params = ScriptedParams {
        actions: vec![
            ScriptedFrame {
                frame: 5,
                actions: vec![GameAction::MoveLeft],
            },
            ScriptedFrame {
                frame: 10,
                actions: vec![GameAction::Bump, GameAction::MoveRight],
            },
        ],
    };
    let scripted = ScriptedInput::new(&params);

    assert_eq!(
        scripted.actions_for_frame(5),
        vec![GameAction::MoveLeft],
        "frame 5 must return [MoveLeft]"
    );
    assert_eq!(
        scripted.actions_for_frame(10),
        vec![GameAction::Bump, GameAction::MoveRight],
        "frame 10 must return [Bump, MoveRight]"
    );

    // All frames except 5 and 10 must return empty.
    for frame in 0_u32..15 {
        if frame == 5 || frame == 10 {
            continue;
        }
        let result = scripted.actions_for_frame(frame);
        assert!(
            result.is_empty(),
            "frame {frame} should return empty Vec, got {result:?}"
        );
    }
}

// -------------------------------------------------------------------------
// ScriptedInput — empty entries returns nothing
// -------------------------------------------------------------------------

/// `ScriptedInput` with no entries always returns an empty `Vec`.
#[test]
fn scripted_input_with_empty_entries_returns_nothing() {
    let params = ScriptedParams { actions: vec![] };
    let scripted = ScriptedInput::new(&params);

    for frame in [0_u32, 1, 100, u32::MAX / 2] {
        let result = scripted.actions_for_frame(frame);
        assert!(
            result.is_empty(),
            "frame {frame}: expected empty Vec from empty ScriptedInput, got {result:?}"
        );
    }
}
