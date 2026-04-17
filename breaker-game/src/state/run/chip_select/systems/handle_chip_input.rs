//! Handles keyboard input on the chip selection screen.

use bevy::{ecs::system::SystemParam, prelude::*};
use rantzsoft_stateflow::ChangeState;

use crate::{
    chips::inventory::ChipInventory,
    input::InputConfig,
    prelude::*,
    protocol::{messages::ProtocolSelected, resources::ProtocolOffer},
    state::run::chip_select::{
        ChipSelectConfig,
        resources::{ChipOffering, ChipOffers, ChipSelectSelection, SelectionRow},
    },
};

/// Bundled system parameters for chip input response actions.
#[derive(SystemParam)]
pub(crate) struct ChipInputActions<'w> {
    /// Current chip selection state (row + chip index).
    selection:       ResMut<'w, ChipSelectSelection>,
    /// State transition control.
    state_writer:    MessageWriter<'w, ChangeState<ChipSelectState>>,
    /// Message writer for chip selection events.
    chip_writer:     MessageWriter<'w, ChipSelected>,
    /// Message writer for protocol selection events.
    protocol_writer: MessageWriter<'w, ProtocolSelected>,
    /// Inventory for recording decay on non-selected chips.
    inventory:       ResMut<'w, ChipInventory>,
    /// Chip select configuration (decay factor, etc.).
    chip_config:     Res<'w, ChipSelectConfig>,
    /// The offered protocol for this chip-select visit (if any).
    offer:           Res<'w, ProtocolOffer>,
}

/// Handles left/right card navigation, up/down row navigation, and confirmation.
///
/// Reads `ButtonInput<KeyCode>` directly (same pattern as other menus).
/// On chip-row confirm, sends `ChipSelected` with the chosen chip's identity
/// before transitioning. On protocol-row confirm, sends `ProtocolSelected`
/// with the offered protocol kind. Both confirm branches record decay on
/// offered chips via [`ChipInventory`] and request a state change.
pub(crate) fn handle_chip_input(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<InputConfig>,
    offers: Res<ChipOffers>,
    mut actions: ChipInputActions,
) {
    let card_count = offers.0.len();
    let has_offer = actions.offer.0.is_some();

    // No cards AND no protocol offer — confirm just exits.
    if card_count == 0 && !has_offer {
        if config.menu_confirm.iter().any(|k| keys.just_pressed(*k)) {
            actions.state_writer.write(ChangeState::new());
        }
        return;
    }

    // Horizontal navigation — chip row only.
    if actions.selection.row == SelectionRow::Chip && card_count > 0 {
        if config.menu_left.iter().any(|k| keys.just_pressed(*k)) {
            actions.selection.chip_index = if actions.selection.chip_index == 0 {
                card_count - 1
            } else {
                actions.selection.chip_index - 1
            };
        }

        if config.menu_right.iter().any(|k| keys.just_pressed(*k)) {
            actions.selection.chip_index = (actions.selection.chip_index + 1) % card_count;
        }
    }

    // Vertical navigation between rows.
    if config.menu_down.iter().any(|k| keys.just_pressed(*k))
        && actions.selection.row == SelectionRow::Chip
        && has_offer
    {
        actions.selection.row = SelectionRow::Protocol;
    }
    if config.menu_up.iter().any(|k| keys.just_pressed(*k))
        && actions.selection.row == SelectionRow::Protocol
    {
        actions.selection.row = SelectionRow::Chip;
    }

    // Confirm.
    if config.menu_confirm.iter().any(|k| keys.just_pressed(*k)) {
        match actions.selection.row {
            SelectionRow::Chip => {
                if card_count == 0 {
                    // Nothing to confirm on empty chip row — still exit.
                    actions.state_writer.write(ChangeState::new());
                    return;
                }

                let offering = &offers.0[actions.selection.chip_index];
                actions.chip_writer.write(ChipSelected {
                    name: offering.name().to_owned(),
                });

                // Consume ingredient stacks for evolution offerings.
                if let ChipOffering::Evolution { ingredients, .. } = offering {
                    for ingredient in ingredients {
                        actions
                            .inventory
                            .remove_by_template(&ingredient.chip_name, ingredient.stacks_required);
                    }
                }

                // Record decay for non-selected chips.
                for (i, offer) in offers.0.iter().enumerate() {
                    if i != actions.selection.chip_index {
                        actions
                            .inventory
                            .record_offered(offer.name(), actions.chip_config.seen_decay_factor);
                    }
                }

                actions.state_writer.write(ChangeState::new());
            }
            SelectionRow::Protocol => {
                if let Some(def) = actions.offer.0.as_ref() {
                    actions
                        .protocol_writer
                        .write(ProtocolSelected { kind: def.kind() });
                }

                // Decay every offered chip — the player skipped the chip row.
                for offer in &offers.0 {
                    actions
                        .inventory
                        .record_offered(offer.name(), actions.chip_config.seen_decay_factor);
                }

                actions.state_writer.write(ChangeState::new());
            }
        }
    }
}

#[cfg(test)]
mod tests;
