//! `update_previous_dash_state` — copies each entity's current [`DashState`]
//! into its [`PreviousDashState`] once per `FixedUpdate` tick.
//!
//! Scheduled `.after(BreakerSystems::UpdateState)` and placed in
//! `BreakerSystems::UpdatePreviousState`. Systems sitting between these two
//! sets see the new `DashState` AND the old `PreviousDashState`, enabling
//! transition detection.

use bevy::prelude::*;

use crate::breaker::components::{DashState, PreviousDashState};

/// Copies every entity's current [`DashState`] into its [`PreviousDashState`].
///
/// Iterates entities that have BOTH components and overwrites
/// `PreviousDashState.0` with the current `DashState`. Runs once per
/// `FixedUpdate` tick after `BreakerSystems::UpdateState`, inside
/// `BreakerSystems::UpdatePreviousState`.
pub(crate) fn update_previous_dash_state(mut query: Query<(&DashState, &mut PreviousDashState)>) {
    for (state, mut prev) in &mut query {
        prev.0 = *state;
    }
}
