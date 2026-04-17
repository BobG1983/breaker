//! System to tick the hazard selection countdown timer.
//!
//! Critical difference vs `tick_chip_timer`: on expiry this system auto-picks
//! a random offer and emits a `HazardSelected` message before transitioning.
//! The player cannot skip a hazard.

use bevy::prelude::*;
use rand::Rng;
use rantzsoft_stateflow::ChangeState;

use crate::{
    hazard::{messages::HazardSelected, resources::HazardOffers},
    prelude::*,
    state::run::hazard_select::resources::HazardSelectTimer,
};

/// Ticks the hazard selection timer and auto-picks a hazard on expiry.
///
/// On expiry:
///  - `timer.remaining` is clamped to 0.0.
///  - If `HazardOffers` is present and non-empty, a random offer is picked
///    using `GameRng` and a single `HazardSelected` is emitted.
///  - Exactly one `ChangeState<HazardSelectState>` is always emitted so the
///    state machine advances even when there is nothing to pick.
pub(crate) fn tick_hazard_timer(
    time: Res<Time>,
    mut timer: ResMut<HazardSelectTimer>,
    offers: Option<Res<HazardOffers>>,
    mut rng: ResMut<GameRng>,
    mut hazard_writer: MessageWriter<HazardSelected>,
    mut state_writer: MessageWriter<ChangeState<HazardSelectState>>,
) {
    timer.remaining -= time.delta_secs();

    if timer.remaining <= 0.0 {
        timer.remaining = 0.0;

        // Auto-pick a random offer (only if we have any).
        if let Some(offers) = offers.as_ref()
            && !offers.0.is_empty()
        {
            let idx = rng.0.random_range(0..offers.0.len());
            let kind = offers.0[idx].kind();
            hazard_writer.write(HazardSelected { kind });
        }

        state_writer.write(ChangeState::new());
    }
}

#[cfg(test)]
mod tests;
