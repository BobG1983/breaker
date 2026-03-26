//! Breaker domain system sets for cross-domain ordering.

use bevy::prelude::*;

/// System sets exported by the breaker domain for cross-domain ordering.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum BreakerSystems {
    /// The `move_breaker` system — updates breaker position from input.
    Move,
    /// The `init_breaker_params` system — stamps config-derived components on the breaker entity.
    InitParams,
    /// The `reset_breaker` system — resets breaker position and state on node entry.
    Reset,
    /// The `grade_bump` system — grades bump timing and writes `BumpPerformed`/`BumpWhiffed`.
    GradeBump,
    /// The `update_breaker_state` system — updates breaker state machine each tick.
    UpdateState,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_params_variant_exists() {
        // Ensures BreakerSystems::InitParams is a valid variant
        let set = BreakerSystems::InitParams;
        assert_ne!(set, BreakerSystems::Move);
    }
}
