//! Messages sent by the node subdomain.

use bevy::prelude::*;

/// Sent when all target cells in a node have been destroyed.
///
/// Consumed by the run state machine and UI.
#[derive(Message, Clone, Debug)]
pub struct NodeCleared;

/// Sent when the node timer reaches zero.
///
/// Consumed by the run state machine.
#[derive(Message, Clone, Debug)]
pub struct TimerExpired;

/// Sent by the breaker behavior system to subtract time from the node timer.
///
/// Consumed by `apply_time_penalty` in the node subdomain.
#[derive(Message, Clone, Debug)]
pub struct ApplyTimePenalty {
    /// Seconds to subtract from the node timer.
    pub seconds: f32,
}

/// Sent by effect reversal to add time back to the node timer.
///
/// Consumed by `reverse_time_penalty` in the node subdomain.
#[derive(Message, Clone, Debug)]
pub struct ReverseTimePenalty {
    /// Seconds to add back to the node timer.
    pub seconds: f32,
}

/// Sent by `spawn_cells_from_layout` after all cells are spawned.
///
/// Consumed by the spawn coordinator.
#[derive(Message, Clone, Debug)]
pub struct CellsSpawned;

/// Sent by the spawn coordinator after all domain spawn signals have been received.
///
/// Indicates the game world is fully set up and gameplay can begin. Consumed by
/// the scenario runner for baseline entity count sampling.
#[derive(Message, Clone, Debug)]
pub struct SpawnNodeComplete;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_debug_format() {
        assert!(format!("{NodeCleared:?}").contains("NodeCleared"));
        assert!(format!("{TimerExpired:?}").contains("TimerExpired"));
        let penalty = ApplyTimePenalty { seconds: 5.0 };
        assert!(format!("{penalty:?}").contains("ApplyTimePenalty"));
        assert!(format!("{CellsSpawned:?}").contains("CellsSpawned"));
        assert!(format!("{SpawnNodeComplete:?}").contains("SpawnNodeComplete"));
    }

    #[test]
    fn reverse_time_penalty_debug_format() {
        let reverse = ReverseTimePenalty { seconds: 5.0 };
        assert!(
            (reverse.seconds - 5.0).abs() < f32::EPSILON,
            "expected seconds to be 5.0, got {}",
            reverse.seconds
        );
        assert!(format!("{reverse:?}").contains("ReverseTimePenalty"));
    }
}
