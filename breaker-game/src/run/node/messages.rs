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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_debug_format() {
        let a = NodeCleared;
        assert!(format!("{a:?}").contains("NodeCleared"));

        let b = TimerExpired;
        assert!(format!("{b:?}").contains("TimerExpired"));

        let c = ApplyTimePenalty { seconds: 5.0 };
        assert!(format!("{c:?}").contains("ApplyTimePenalty"));
    }
}
