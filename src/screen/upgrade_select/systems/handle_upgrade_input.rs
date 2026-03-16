//! Handles keyboard input on the upgrade selection screen.

use bevy::prelude::*;

use crate::{
    input::InputConfig,
    screen::upgrade_select::resources::{UpgradeOffers, UpgradeSelectSelection},
    shared::GameState,
    ui::messages::UpgradeSelected,
};

/// Handles left/right card navigation and confirmation.
///
/// Reads `ButtonInput<KeyCode>` directly (same pattern as other menus).
/// On confirm, sends `UpgradeSelected` with the chosen upgrade's identity
/// before transitioning to `NodeTransition`.
pub fn handle_upgrade_input(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<InputConfig>,
    offers: Res<UpgradeOffers>,
    mut selection: ResMut<UpgradeSelectSelection>,
    mut next_state: ResMut<NextState<GameState>>,
    mut writer: MessageWriter<UpgradeSelected>,
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
    if config.move_left.iter().any(|k| keys.just_pressed(*k)) {
        selection.index = if selection.index == 0 {
            card_count - 1
        } else {
            selection.index - 1
        };
    }

    // Navigate right
    if config.move_right.iter().any(|k| keys.just_pressed(*k)) {
        selection.index = (selection.index + 1) % card_count;
    }

    // Confirm selection
    if config.menu_confirm.iter().any(|k| keys.just_pressed(*k)) {
        let upgrade = &offers.0[selection.index];
        writer.write(UpgradeSelected {
            name: upgrade.name.clone(),
            kind: upgrade.kind,
        });
        next_state.set(GameState::NodeTransition);
    }
}

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;

    use super::*;
    use crate::upgrades::{UpgradeDefinition, UpgradeKind};

    #[derive(Resource, Default)]
    struct ReceivedUpgrades(Vec<UpgradeSelected>);

    fn collect_upgrades(
        mut reader: MessageReader<UpgradeSelected>,
        mut received: ResMut<ReceivedUpgrades>,
    ) {
        for msg in reader.read() {
            received.0.push(msg.clone());
        }
    }

    fn make_offers(count: usize) -> UpgradeOffers {
        let all = vec![
            UpgradeDefinition {
                name: "Piercing Shot".to_owned(),
                kind: UpgradeKind::Amp,
                description: "Bolt passes through".to_owned(),
            },
            UpgradeDefinition {
                name: "Wide Breaker".to_owned(),
                kind: UpgradeKind::Augment,
                description: "Breaker width increased".to_owned(),
            },
            UpgradeDefinition {
                name: "Surge".to_owned(),
                kind: UpgradeKind::Overclock,
                description: "Shockwave damage".to_owned(),
            },
        ];
        UpgradeOffers(all.into_iter().take(count).collect())
    }

    fn test_app() -> App {
        test_app_with_offers(make_offers(3))
    }

    fn test_app_with_offers(offers: UpgradeOffers) -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_resource::<ButtonInput<KeyCode>>();
        app.insert_resource(InputConfig::default());
        app.init_state::<GameState>();
        app.insert_resource(UpgradeSelectSelection { index: 0 });
        app.insert_resource(offers);
        app.init_resource::<ReceivedUpgrades>();
        app.add_message::<UpgradeSelected>();
        app.add_systems(Update, (handle_upgrade_input, collect_upgrades).chain());
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

        let selection = app.world().resource::<UpgradeSelectSelection>();
        assert_eq!(selection.index, 1);
    }

    #[test]
    fn left_wraps_selection() {
        let mut app = test_app();
        press_key(&mut app, KeyCode::ArrowLeft);

        let selection = app.world().resource::<UpgradeSelectSelection>();
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
    fn confirm_sends_upgrade_selected_message() {
        let mut app = test_app();
        press_key(&mut app, KeyCode::Enter);

        let received = app.world().resource::<ReceivedUpgrades>();
        assert_eq!(received.0.len(), 1);
        assert_eq!(received.0[0].name, "Piercing Shot");
        assert_eq!(received.0[0].kind, UpgradeKind::Amp);
    }

    #[test]
    fn confirm_second_card_sends_correct_upgrade() {
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

        let received = app.world().resource::<ReceivedUpgrades>();
        assert_eq!(received.0.len(), 1);
        assert_eq!(received.0[0].name, "Wide Breaker");
        assert_eq!(received.0[0].kind, UpgradeKind::Augment);
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

        let selection = app.world().resource::<UpgradeSelectSelection>();
        assert_eq!(selection.index, 0); // wraps back to 0
    }

    #[test]
    fn no_input_no_change() {
        let mut app = test_app();
        app.update();

        let selection = app.world().resource::<UpgradeSelectSelection>();
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

        let received = app.world().resource::<ReceivedUpgrades>();
        assert!(
            received.0.is_empty(),
            "expected no UpgradeSelected messages"
        );
    }

    #[test]
    fn two_card_navigation_wraps_correctly() {
        let mut app = test_app_with_offers(make_offers(2));

        // Right once → index 1
        press_key(&mut app, KeyCode::ArrowRight);
        let selection = app.world().resource::<UpgradeSelectSelection>();
        assert_eq!(selection.index, 1);

        // Right again → wraps to 0
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(KeyCode::ArrowRight);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();
        press_key(&mut app, KeyCode::ArrowRight);
        let selection = app.world().resource::<UpgradeSelectSelection>();
        assert_eq!(selection.index, 0);
    }
}
