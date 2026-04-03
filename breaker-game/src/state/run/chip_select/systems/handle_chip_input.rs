//! Handles keyboard input on the chip selection screen.

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{
    chips::inventory::ChipInventory,
    input::InputConfig,
    shared::GameState,
    state::run::chip_select::{
        ChipSelectConfig,
        messages::ChipSelected,
        resources::{ChipOffering, ChipOffers, ChipSelectSelection},
    },
};

/// Bundled system parameters for chip input response actions.
#[derive(SystemParam)]
pub(crate) struct ChipInputActions<'w> {
    /// Current chip selection index.
    selection: ResMut<'w, ChipSelectSelection>,
    /// State transition control.
    next_state: ResMut<'w, NextState<GameState>>,
    /// Message writer for chip selection events.
    writer: MessageWriter<'w, ChipSelected>,
    /// Inventory for recording decay on non-selected chips.
    inventory: ResMut<'w, ChipInventory>,
    /// Chip select configuration (decay factor, etc.).
    chip_config: Res<'w, ChipSelectConfig>,
}

/// Handles left/right card navigation and confirmation.
///
/// Reads `ButtonInput<KeyCode>` directly (same pattern as other menus).
/// On confirm, sends `ChipSelected` with the chosen chip's identity
/// before transitioning to `TransitionIn`. Also records decay on
/// non-selected chips via [`ChipInventory`].
pub(crate) fn handle_chip_input(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<InputConfig>,
    offers: Res<ChipOffers>,
    mut actions: ChipInputActions,
) {
    let card_count = offers.0.len();

    // No cards — nothing to navigate or confirm
    if card_count == 0 {
        if config.menu_confirm.iter().any(|k| keys.just_pressed(*k)) {
            actions.next_state.set(GameState::TransitionIn);
        }
        return;
    }

    // Navigate left
    if config.menu_left.iter().any(|k| keys.just_pressed(*k)) {
        actions.selection.index = if actions.selection.index == 0 {
            card_count - 1
        } else {
            actions.selection.index - 1
        };
    }

    // Navigate right
    if config.menu_right.iter().any(|k| keys.just_pressed(*k)) {
        actions.selection.index = (actions.selection.index + 1) % card_count;
    }

    // Confirm selection
    if config.menu_confirm.iter().any(|k| keys.just_pressed(*k)) {
        let offering = &offers.0[actions.selection.index];
        actions.writer.write(ChipSelected {
            name: offering.name().to_owned(),
        });

        // Consume ingredient stacks for evolution offerings
        if let ChipOffering::Evolution { ingredients, .. } = offering {
            for ingredient in ingredients {
                actions
                    .inventory
                    .remove_by_template(&ingredient.chip_name, ingredient.stacks_required);
            }
        }

        // Record decay for non-selected chips
        for (i, offer) in offers.0.iter().enumerate() {
            if i != actions.selection.index {
                actions
                    .inventory
                    .record_offered(offer.name(), actions.chip_config.seen_decay_factor);
            }
        }

        actions.next_state.set(GameState::TransitionIn);
    }
}

#[cfg(test)]
mod tests;
