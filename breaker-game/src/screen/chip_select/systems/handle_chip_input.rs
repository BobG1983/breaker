//! Handles keyboard input on the chip selection screen.

use bevy::prelude::*;

use crate::{
    input::InputConfig,
    screen::chip_select::resources::{ChipOffers, ChipSelectSelection},
    shared::GameState,
    ui::messages::ChipSelected,
};

/// Handles left/right card navigation and confirmation.
///
/// Reads `ButtonInput<KeyCode>` directly (same pattern as other menus).
/// On confirm, sends `ChipSelected` with the chosen chip's identity
/// before transitioning to `NodeTransition`.
pub(crate) fn handle_chip_input(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<InputConfig>,
    offers: Res<ChipOffers>,
    mut selection: ResMut<ChipSelectSelection>,
    mut next_state: ResMut<NextState<GameState>>,
    mut writer: MessageWriter<ChipSelected>,
) {
    let card_count = offers.0.len();

    // No cards — nothing to navigate or confirm
    if card_count == 0 {
        if config.menu_confirm.iter().any(|k| keys.just_pressed(*k)) {
            next_state.set(GameState::NodeTransition);
        }
        return;
    }

    // Navigate left
    if config.menu_left.iter().any(|k| keys.just_pressed(*k)) {
        selection.index = if selection.index == 0 {
            card_count - 1
        } else {
            selection.index - 1
        };
    }

    // Navigate right
    if config.menu_right.iter().any(|k| keys.just_pressed(*k)) {
        selection.index = (selection.index + 1) % card_count;
    }

    // Confirm selection
    if config.menu_confirm.iter().any(|k| keys.just_pressed(*k)) {
        let chip = &offers.0[selection.index];
        writer.write(ChipSelected {
            name: chip.name.clone(),
            kind: chip.kind,
        });
        next_state.set(GameState::NodeTransition);
    }
}

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;

    use super::*;
    use crate::chips::{ChipDefinition, ChipKind};

    #[derive(Resource, Default)]
    struct ReceivedChips(Vec<ChipSelected>);

    fn collect_chips(mut reader: MessageReader<ChipSelected>, mut received: ResMut<ReceivedChips>) {
        for msg in reader.read() {
            received.0.push(msg.clone());
        }
    }

    fn make_offers(count: usize) -> ChipOffers {
        let all = vec![
            ChipDefinition {
                name: "Piercing Shot".to_owned(),
                kind: ChipKind::Amp,
                description: "Bolt passes through".to_owned(),
            },
            ChipDefinition {
                name: "Wide Breaker".to_owned(),
                kind: ChipKind::Augment,
                description: "Breaker width increased".to_owned(),
            },
            ChipDefinition {
                name: "Surge".to_owned(),
                kind: ChipKind::Overclock,
                description: "Shockwave damage".to_owned(),
            },
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
    fn confirm_transitions_to_node_transition() {
        let mut app = test_app();
        press_key(&mut app, KeyCode::Enter);

        let next = app.world().resource::<NextState<GameState>>();
        assert!(
            format!("{next:?}").contains("NodeTransition"),
            "expected NodeTransition, got: {next:?}"
        );
    }

    #[test]
    fn confirm_sends_chip_selected_message() {
        let mut app = test_app();
        press_key(&mut app, KeyCode::Enter);

        let received = app.world().resource::<ReceivedChips>();
        assert_eq!(received.0.len(), 1);
        assert_eq!(received.0[0].name, "Piercing Shot");
        assert_eq!(received.0[0].kind, ChipKind::Amp);
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
        assert_eq!(received.0[0].kind, ChipKind::Augment);
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
            !format!("{next:?}").contains("NodeTransition"),
            "expected no transition, got: {next:?}"
        );
    }

    #[test]
    fn empty_offers_confirm_transitions_without_message() {
        let mut app = test_app_with_offers(make_offers(0));
        press_key(&mut app, KeyCode::Enter);

        let next = app.world().resource::<NextState<GameState>>();
        assert!(
            format!("{next:?}").contains("NodeTransition"),
            "expected NodeTransition, got: {next:?}"
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
}
