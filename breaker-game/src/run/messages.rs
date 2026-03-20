//! Messages sent by the run domain (parent level).

use bevy::prelude::*;

/// Sent when the run is lost (e.g. all lives depleted).
///
/// Consumed by the run state machine to set [`RunOutcome::LivesDepleted`] and transition
/// to [`GameState::RunEnd`].
#[derive(Message, Clone, Debug)]
pub struct RunLost;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_lost_debug_format() {
        let msg = RunLost;
        assert!(format!("{msg:?}").contains("RunLost"));
    }
}
