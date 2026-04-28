//! Handles keyboard input on the chip selection screen.

use bevy::{ecs::system::SystemParam, prelude::*};
use rantzsoft_stateflow::ChangeState;

use crate::{
    chips::inventory::ChipInventory,
    input::InputConfig,
    mutators::protocols::{
        definition::ProtocolKind,
        messages::ProtocolSelected,
        resources::{ActiveProtocols, ProtocolOffer},
    },
    prelude::*,
    state::run::chip_select::{
        ChipSelectConfig,
        messages::ChipOfferSkipped,
        resources::{ChipOffering, ChipOffers, ChipSelectSelection, SelectionRow},
    },
};

/// Bundled system parameters for chip input response actions.
#[derive(SystemParam)]
pub(crate) struct ChipInputActions<'w> {
    /// Current chip selection state (row + chip index).
    selection:        ResMut<'w, ChipSelectSelection>,
    /// State transition control.
    state_writer:     MessageWriter<'w, ChangeState<ChipSelectState>>,
    /// Message writer for chip selection events.
    chip_writer:      MessageWriter<'w, ChipSelected>,
    /// Message writer for protocol selection events.
    protocol_writer:  MessageWriter<'w, ProtocolSelected>,
    /// Message writer for the Greed skip event.
    skip_writer:      MessageWriter<'w, ChipOfferSkipped>,
    /// Inventory for recording decay on non-selected chips.
    inventory:        ResMut<'w, ChipInventory>,
    /// Chip select configuration (decay factor, etc.).
    chip_config:      Res<'w, ChipSelectConfig>,
    /// The offered protocol for this chip-select visit (if any).
    offer:            Res<'w, ProtocolOffer>,
    /// Active protocols — drives whether the Skip row is reachable.
    active_protocols: Res<'w, ActiveProtocols>,
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
    let greed_active = actions.active_protocols.contains(ProtocolKind::Greed);

    // No cards AND no protocol offer AND Greed inactive — confirm just exits.
    // (When Greed is active, the Skip row is reachable even with zero offers.)
    if card_count == 0 && !has_offer && !greed_active {
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
    let down_pressed = config.menu_down.iter().any(|k| keys.just_pressed(*k));
    let up_pressed = config.menu_up.iter().any(|k| keys.just_pressed(*k));

    if down_pressed {
        actions.selection.row = match (actions.selection.row, has_offer, greed_active) {
            (SelectionRow::Chip, true, _) => SelectionRow::Protocol,
            (SelectionRow::Chip, false, true) | (SelectionRow::Protocol, _, true) => {
                SelectionRow::Skip
            }
            (row, ..) => row,
        };
    }

    if up_pressed {
        // Up from Skip with no offer → Chip; from Protocol → Chip; from Chip → Chip (no-op).
        // All three collapse to Chip; the wildcard is the cleanest expression
        // (clippy rejects spelling them out separately).
        actions.selection.row = match actions.selection.row {
            SelectionRow::Skip if has_offer => SelectionRow::Protocol,
            _ => SelectionRow::Chip,
        };
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
                decay_all_offers(
                    &mut actions.inventory,
                    &offers,
                    actions.chip_config.seen_decay_factor,
                );
                actions.state_writer.write(ChangeState::new());
            }
            SelectionRow::Skip => {
                // Intentionally unconditional. Reachability is gated upstream
                // (the Skip row only spawns when Greed is active, and the
                // vertical-nav match only routes here when `greed_active`).
                // The protocol-layer gate is `greed_on_skip` (downstream),
                // which discards the message when Greed is inactive — a
                // stray emit from this arm is harmless.
                actions.skip_writer.write(ChipOfferSkipped);
                decay_all_offers(
                    &mut actions.inventory,
                    &offers,
                    actions.chip_config.seen_decay_factor,
                );
                actions.state_writer.write(ChangeState::new());
            }
        }
    }
}

/// Apply `record_offered` with `decay_factor` to every chip in `offers`.
/// Used by the Protocol and Skip confirm arms (the player took no chip, so
/// every offer decays). The Chip arm uses its own filtered loop because it
/// must skip the index the player selected.
fn decay_all_offers(inventory: &mut ChipInventory, offers: &ChipOffers, decay_factor: f32) {
    for offer in &offers.0 {
        inventory.record_offered(offer.name(), decay_factor);
    }
}

#[cfg(test)]
mod tests;
