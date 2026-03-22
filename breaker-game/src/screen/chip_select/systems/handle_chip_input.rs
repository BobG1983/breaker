//! Handles keyboard input on the chip selection screen.

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{
    chips::inventory::ChipInventory,
    input::InputConfig,
    screen::chip_select::{
        ChipSelectConfig,
        resources::{ChipOffers, ChipSelectSelection},
    },
    shared::GameState,
    ui::messages::ChipSelected,
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
        let chip = &offers.0[actions.selection.index];
        actions.writer.write(ChipSelected {
            name: chip.name.clone(),
        });

        // Record decay for non-selected chips
        for (i, offer) in offers.0.iter().enumerate() {
            if i != actions.selection.index {
                actions
                    .inventory
                    .record_offered(&offer.name, actions.chip_config.seen_decay_factor);
            }
        }

        actions.next_state.set(GameState::TransitionIn);
    }
}

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;

    use super::*;
    use crate::chips::{
        ChipDefinition,
        definition::{AmpEffect, ChipEffect},
    };

    #[derive(Resource, Default)]
    struct ReceivedChips(Vec<ChipSelected>);

    fn collect_chips(mut reader: MessageReader<ChipSelected>, mut received: ResMut<ReceivedChips>) {
        for msg in reader.read() {
            received.0.push(msg.clone());
        }
    }

    fn make_offers(count: usize) -> ChipOffers {
        let all = vec![
            ChipDefinition::test("Piercing Shot", ChipEffect::Amp(AmpEffect::Piercing(1)), 3),
            ChipDefinition::test_simple("Wide Breaker"),
            ChipDefinition::test_simple("Surge"),
        ];
        ChipOffers(all.into_iter().take(count).collect())
    }

    fn test_app() -> App {
        test_app_with_offers(make_offers(3))
    }

    fn test_app_with_offers(offers: ChipOffers) -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_resource::<ButtonInput<KeyCode>>()
            .insert_resource(InputConfig::default())
            .init_state::<GameState>()
            .insert_resource(ChipSelectSelection { index: 0 })
            .insert_resource(offers)
            .init_resource::<ReceivedChips>()
            .init_resource::<ChipInventory>()
            .insert_resource(ChipSelectConfig::default())
            .add_message::<ChipSelected>()
            .add_systems(Update, (handle_chip_input, collect_chips).chain());
        app
    }

    fn press_key(app: &mut App, key: KeyCode) {
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(key);
        app.update();
    }

    #[test]
    fn right_advances_selection() {
        let mut app = test_app();
        press_key(&mut app, KeyCode::ArrowRight);

        let selection = app.world().resource::<ChipSelectSelection>();
        assert_eq!(selection.index, 1);
    }

    #[test]
    fn left_wraps_selection() {
        let mut app = test_app();
        press_key(&mut app, KeyCode::ArrowLeft);

        let selection = app.world().resource::<ChipSelectSelection>();
        assert_eq!(selection.index, 2); // wraps from 0 to last (2)
    }

    #[test]
    fn confirm_transitions_to_transition_in() {
        let mut app = test_app();
        press_key(&mut app, KeyCode::Enter);

        let next = app.world().resource::<NextState<GameState>>();
        assert!(
            format!("{next:?}").contains("TransitionIn"),
            "expected TransitionIn, got: {next:?}"
        );
    }

    #[test]
    fn confirm_sends_chip_selected_message() {
        let mut app = test_app();
        press_key(&mut app, KeyCode::Enter);

        let received = app.world().resource::<ReceivedChips>();
        assert_eq!(received.0.len(), 1);
        assert_eq!(received.0[0].name, "Piercing Shot");
    }

    #[test]
    fn confirm_second_card_sends_correct_chip() {
        let mut app = test_app();
        // Navigate right once to select index 1
        press_key(&mut app, KeyCode::ArrowRight);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(KeyCode::ArrowRight);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();

        press_key(&mut app, KeyCode::Enter);

        let received = app.world().resource::<ReceivedChips>();
        assert_eq!(received.0.len(), 1);
        assert_eq!(received.0[0].name, "Wide Breaker");
    }

    #[test]
    fn right_wraps_around() {
        let mut app = test_app();
        // Go right 3 times to wrap around
        for _ in 0..3 {
            press_key(&mut app, KeyCode::ArrowRight);
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .release(KeyCode::ArrowRight);
            app.world_mut()
                .resource_mut::<ButtonInput<KeyCode>>()
                .clear();
        }

        let selection = app.world().resource::<ChipSelectSelection>();
        assert_eq!(selection.index, 0); // wraps back to 0
    }

    #[test]
    fn no_input_no_change() {
        let mut app = test_app();
        app.update();

        let selection = app.world().resource::<ChipSelectSelection>();
        assert_eq!(selection.index, 0);

        let next = app.world().resource::<NextState<GameState>>();
        assert!(
            !format!("{next:?}").contains("TransitionIn"),
            "expected no transition, got: {next:?}"
        );
    }

    #[test]
    fn empty_offers_confirm_transitions_without_message() {
        let mut app = test_app_with_offers(make_offers(0));
        press_key(&mut app, KeyCode::Enter);

        let next = app.world().resource::<NextState<GameState>>();
        assert!(
            format!("{next:?}").contains("TransitionIn"),
            "expected TransitionIn, got: {next:?}"
        );

        let received = app.world().resource::<ReceivedChips>();
        assert!(received.0.is_empty(), "expected no ChipSelected messages");
    }

    #[test]
    fn two_card_navigation_wraps_correctly() {
        let mut app = test_app_with_offers(make_offers(2));

        // Right once → index 1
        press_key(&mut app, KeyCode::ArrowRight);
        let selection = app.world().resource::<ChipSelectSelection>();
        assert_eq!(selection.index, 1);

        // Right again → wraps to 0
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(KeyCode::ArrowRight);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        press_key(&mut app, KeyCode::ArrowRight);
        let selection = app.world().resource::<ChipSelectSelection>();
        assert_eq!(selection.index, 0);
    }

    #[test]
    fn confirm_records_decay_for_non_selected_chips() {
        // Offers: index 0 = "Piercing Shot", 1 = "Wide Breaker", 2 = "Surge"
        // Selection at index 0 → confirms "Piercing Shot"
        // Non-selected: "Wide Breaker" and "Surge" should get decay 0.8
        let mut app = test_app();
        press_key(&mut app, KeyCode::Enter);

        let inventory = app.world().resource::<ChipInventory>();

        // Selected chip should NOT have decay applied
        let selected_decay = inventory.weight_decay("Piercing Shot");
        assert!(
            (selected_decay - 1.0).abs() < f32::EPSILON,
            "selected chip 'Piercing Shot' should not have decay, got {selected_decay}"
        );

        // Non-selected chips should have decay = 0.8
        let wb_decay = inventory.weight_decay("Wide Breaker");
        assert!(
            (wb_decay - 0.8).abs() < f32::EPSILON,
            "non-selected 'Wide Breaker' should have decay 0.8, got {wb_decay}"
        );

        let surge_decay = inventory.weight_decay("Surge");
        assert!(
            (surge_decay - 0.8).abs() < f32::EPSILON,
            "non-selected 'Surge' should have decay 0.8, got {surge_decay}"
        );
    }

    #[test]
    fn single_chip_confirm_applies_no_decay() {
        // Only 1 chip offered — no non-selected chips to decay
        let mut app = test_app_with_offers(make_offers(1));
        press_key(&mut app, KeyCode::Enter);

        let inventory = app.world().resource::<ChipInventory>();

        // The only chip was selected — no decay should be applied
        let decay = inventory.weight_decay("Piercing Shot");
        assert!(
            (decay - 1.0).abs() < f32::EPSILON,
            "single offered + selected chip should have no decay, got {decay}"
        );
    }
}
