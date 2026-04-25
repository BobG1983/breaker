//! Handles keyboard input on the hazard selection screen.

use bevy::{ecs::system::SystemParam, prelude::*};
use rantzsoft_stateflow::ChangeState;

use crate::{
    input::InputConfig,
    mutators::hazards::{messages::HazardSelected, resources::HazardOffers},
    prelude::*,
    state::run::hazard_select::resources::HazardSelectSelection,
};

/// Bundled system parameters for hazard input response actions.
#[derive(SystemParam)]
pub(crate) struct HazardInputActions<'w> {
    /// Current hazard selection state (which card index is highlighted).
    selection:     ResMut<'w, HazardSelectSelection>,
    /// State transition control.
    state_writer:  MessageWriter<'w, ChangeState<HazardSelectState>>,
    /// Message writer for hazard selection events.
    hazard_writer: MessageWriter<'w, HazardSelected>,
}

/// Handles left/right card navigation (with wrap) and Enter/Space confirmation.
///
/// On confirm, writes a single `HazardSelected` with the chosen kind and a
/// single `ChangeState<HazardSelectState>` to advance toward `AnimateOut`.
/// If offers is empty, confirm still emits `ChangeState` (no `HazardSelected`).
pub(crate) fn handle_hazard_input(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<InputConfig>,
    offers: Res<HazardOffers>,
    mut actions: HazardInputActions,
) {
    let card_count = offers.0.len();

    // No cards — confirm just exits the screen without emitting HazardSelected.
    if card_count == 0 {
        if config.menu_confirm.iter().any(|k| keys.just_pressed(*k)) {
            actions.state_writer.write(ChangeState::new());
        }
        return;
    }

    // Horizontal navigation (single row, wraps).
    if config.menu_left.iter().any(|k| keys.just_pressed(*k)) {
        actions.selection.card_index = if actions.selection.card_index == 0 {
            card_count - 1
        } else {
            actions.selection.card_index - 1
        };
    }
    if config.menu_right.iter().any(|k| keys.just_pressed(*k)) {
        actions.selection.card_index = (actions.selection.card_index + 1) % card_count;
    }

    // Confirm — write HazardSelected and request state change.
    if config.menu_confirm.iter().any(|k| keys.just_pressed(*k)) {
        let kind = offers.0[actions.selection.card_index].kind();
        actions.hazard_writer.write(HazardSelected { kind });
        actions.state_writer.write(ChangeState::new());
    }
}

#[cfg(test)]
mod tests;
