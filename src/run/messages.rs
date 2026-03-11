//! Messages sent by the run domain.

use bevy::prelude::*;

/// Sent when all target cells in a node have been destroyed.
///
/// Consumed by the state machine and UI.
#[derive(Message, Clone, Debug)]
pub struct NodeCleared;

/// Sent when the node timer reaches zero.
///
/// Consumed by the state machine.
#[derive(Message, Clone, Debug)]
pub struct TimerExpired;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn messages_debug_format() {
        let a = NodeCleared;
        assert!(format!("{a:?}").contains("NodeCleared"));

        let b = TimerExpired;
        assert!(format!("{b:?}").contains("TimerExpired"));
    }
}
