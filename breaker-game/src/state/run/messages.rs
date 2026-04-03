//! Messages sent by the run domain (parent level).

use bevy::prelude::*;

use crate::state::run::resources::HighlightKind;

/// Sent when the run is lost (e.g. all lives depleted).
///
/// Consumed by the run state machine to set [`RunOutcome::LivesDepleted`] and transition
/// to [`GameState::RunEnd`].
#[derive(Message, Clone, Debug)]
pub struct RunLost;

/// Emitted by highlight detection systems when a memorable moment is detected.
///
/// Consumed by the juice feedback system to spawn in-game text popups.
#[derive(Message, Clone, Debug)]
pub struct HighlightTriggered {
    /// Which kind of memorable moment was detected.
    pub kind: HighlightKind,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_lost_debug_format() {
        let msg = RunLost;
        assert!(format!("{msg:?}").contains("RunLost"));
    }
}
