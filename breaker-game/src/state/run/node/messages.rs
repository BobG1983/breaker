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

/// Sent by hazards (e.g. Decay) to subtract time from the node timer.
///
/// Consumed by `apply_reduce_node_timer` in the node subdomain.
#[derive(Message, Clone, Debug)]
pub struct ReduceNodeTimer {
    /// Seconds to subtract from the node timer.
    pub delta: f32,
}

/// Sent by protocols (e.g. Siphon) to add time back to the node timer.
///
/// Consumed by `apply_increase_node_timer` in the node subdomain. Read (in the
/// scenario runner) by `check_timer_monotonically_decreasing` for exemption logic.
#[derive(Message, Clone, Debug)]
pub struct IncreaseNodeTimer {
    /// Seconds to add back to the node timer.
    pub delta: f32,
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
        let reduce = ReduceNodeTimer { delta: 5.0 };
        assert!(format!("{reduce:?}").contains("ReduceNodeTimer"));
        assert!(format!("{CellsSpawned:?}").contains("CellsSpawned"));
        assert!(format!("{SpawnNodeComplete:?}").contains("SpawnNodeComplete"));
    }

    #[test]
    fn increase_node_timer_debug_format() {
        let increase = IncreaseNodeTimer { delta: 5.0 };
        assert!(
            (increase.delta - 5.0).abs() < f32::EPSILON,
            "expected delta to be 5.0, got {}",
            increase.delta
        );
        assert!(format!("{increase:?}").contains("IncreaseNodeTimer"));
    }
}
