//! Breaker state machine components.

use bevy::prelude::*;

/// The breaker's movement state machine.
///
/// Transitions: Idle → Dashing → Braking → Settling → Idle.
/// Normal movement is available in Idle and Settling states.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum BreakerState {
    /// Neutral state — normal horizontal movement available.
    #[default]
    Idle,
    /// Burst horizontal speed — breaker tilts in movement direction.
    Dashing,
    /// Rapid deceleration — breaker tilts opposite to movement direction.
    Braking,
    /// Returning to neutral — tilt returns to flat.
    Settling,
}

/// Tracks the remaining time in timed breaker states (Dashing, Settling).
#[derive(Component, Debug, Default)]
pub struct BreakerStateTimer {
    /// Remaining time in the current timed state.
    pub remaining: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn breaker_state_default_is_idle() {
        assert_eq!(BreakerState::default(), BreakerState::Idle);
    }
}
