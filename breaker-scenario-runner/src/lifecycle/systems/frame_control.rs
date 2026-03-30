//! Frame counting, frame limit checking, exit/restart, and game-state mapping.

use bevy::prelude::*;
use breaker::{run::node::messages::SpawnNodeComplete, shared::GameState};

use super::types::ScenarioConfig;
use crate::{
    invariants::{ScenarioFrame, ScenarioStats},
    types::ForcedGameState,
};

/// Increments [`ScenarioFrame`] by 1 each fixed-update tick.
///
/// Also updates [`ScenarioStats::max_frame`] when the stats resource is present.
pub fn tick_scenario_frame(
    mut frame: ResMut<ScenarioFrame>,
    mut stats: Option<ResMut<ScenarioStats>>,
) {
    frame.0 += 1;
    if let Some(ref mut s) = stats {
        s.max_frame = frame.0;
    }
}

/// Sends [`AppExit::Success`] when [`ScenarioFrame`] reaches `max_frames`.
pub fn check_frame_limit(
    frame: Res<ScenarioFrame>,
    config: Res<ScenarioConfig>,
    mut exits: MessageWriter<AppExit>,
) {
    if frame.0 >= config.definition.max_frames {
        exits.write(AppExit::Success);
    }
}

/// Sends [`AppExit::Success`] when the run ends naturally.
///
/// Runs every frame while in `RunEnd` (not as a one-shot `OnEnter`) so that
/// the Winit event loop reliably sees the exit message on macOS.
pub fn exit_on_run_end(mut exits: MessageWriter<AppExit>) {
    exits.write(AppExit::Success);
}

/// Redirects `RunEnd` back to `MainMenu` (which `bypass_menu_to_playing`
/// sends to `Playing`). Used when `allow_early_end` is false so the
/// scenario runs for the full `max_frames` frame budget.
pub fn restart_run_on_end(mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::MainMenu);
}

/// Maps a [`ForcedGameState`] to the game crate's [`GameState`].
///
/// Used by [`super::debug_setup::apply_debug_setup`] to translate the RON-serializable enum
/// into the Bevy state enum.
#[must_use]
pub const fn map_forced_game_state(forced: ForcedGameState) -> GameState {
    match forced {
        ForcedGameState::Loading => GameState::Loading,
        ForcedGameState::MainMenu => GameState::MainMenu,
        ForcedGameState::RunSetup => GameState::RunSetup,
        ForcedGameState::Playing => GameState::Playing,
        ForcedGameState::TransitionOut => GameState::TransitionOut,
        ForcedGameState::ChipSelect => GameState::ChipSelect,
        ForcedGameState::TransitionIn => GameState::TransitionIn,
        ForcedGameState::RunEnd => GameState::RunEnd,
        ForcedGameState::MetaProgression => GameState::MetaProgression,
    }
}

/// Sets [`ScenarioStats::entered_playing`] to `true` when [`SpawnNodeComplete`]
/// fires, indicating all game entities are spawned and ready.
///
/// Frame counting and invariant checking are gated on `entered_playing`, so
/// no scenario frames advance until the node is fully loaded and spawned.
pub fn mark_entered_playing_on_spawn_complete(
    mut spawn_reader: MessageReader<SpawnNodeComplete>,
    mut stats: Option<ResMut<ScenarioStats>>,
) {
    let spawned = spawn_reader.read().next().is_some();
    if spawned && let Some(ref mut s) = stats {
        s.entered_playing = true;
    }
}

/// Run condition: returns `true` when [`ScenarioStats::entered_playing`] is `true`.
///
/// Used as a `run_if` guard to prevent frame counting and frame-limit
/// checking from running before the game has entered `Playing`.
#[must_use]
pub fn entered_playing(stats: Option<Res<ScenarioStats>>) -> bool {
    stats.is_some_and(|s| s.entered_playing)
}
