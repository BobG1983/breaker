//! Handles keyboard input on the breaker selection screen.

use bevy::{ecs::system::SystemParam, prelude::*};
use rantzsoft_stateflow::ChangeState;

use crate::{
    breaker::{BreakerRegistry, SelectedBreaker},
    input::InputConfig,
    prelude::*,
    shared::RunSeed,
    state::menu::start_game::{
        components::BreakerCard,
        resources::{RunSetupSelection, SeedEntry},
    },
};

/// Bundled parameters for run confirmation (breaker, state transition, seed).
#[derive(SystemParam)]
pub(crate) struct RunConfirmation<'w> {
    breaker:      ResMut<'w, SelectedBreaker>,
    state_writer: MessageWriter<'w, ChangeState<MenuState>>,
    seed:         ResMut<'w, RunSeed>,
}

/// Handles keyboard navigation and confirmation on the run setup screen.
///
/// Reads `ButtonInput<KeyCode>` directly (same pattern as main menu) because
/// this screen runs in `Update` while `InputActions` clears in `FixedPostUpdate`.
pub(crate) fn handle_run_setup_input(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<InputConfig>,
    registry: Res<BreakerRegistry>,
    mut selection: ResMut<RunSetupSelection>,
    mut confirm: RunConfirmation,
    seed_entry: Res<SeedEntry>,
    cards: Query<&BreakerCard>,
) {
    let card_count = cards.iter().count();
    if card_count == 0 {
        return;
    }

    // Navigate down
    if config.menu_down.iter().any(|k| keys.just_pressed(*k)) {
        selection.index = (selection.index + 1) % card_count;
    }

    // Navigate up
    if config.menu_up.iter().any(|k| keys.just_pressed(*k)) {
        selection.index = if selection.index == 0 {
            card_count - 1
        } else {
            selection.index - 1
        };
    }

    // Confirm selection
    if config.menu_confirm.iter().any(|k| keys.just_pressed(*k)) {
        let mut sorted_names: Vec<&String> = registry.names().collect();
        sorted_names.sort();

        if let Some(name) = sorted_names.get(selection.index) {
            confirm.breaker.0.clone_from(name);
        }

        // Parse seed entry: empty → None (random), non-empty → Some(parsed)
        if seed_entry.value.is_empty() {
            confirm.seed.0 = None;
        } else {
            confirm.seed.0 = Some(seed_entry.value.parse::<u64>().unwrap_or(0));
        }

        confirm.state_writer.write(ChangeState::new());
    }
}

#[cfg(test)]
mod tests {
    use bevy::ecs::message::Messages;
    use rantzsoft_stateflow::ChangeState;

    use super::*;
    use crate::breaker::definition::BreakerDefinition;

    fn make_breaker(name: &str) -> BreakerDefinition {
        ron::de::from_str(&format!(
            r#"(name: "{name}", life_pool: None, bolt_lost: Stamp(Breaker, When(BoltLostOccurred, Fire(LoseLife(())))), projectile_hit: Stamp(Breaker, When(Impacted(Salvo), Fire(LoseLife(())))), effects: [])"#,
        ))
        .expect("test RON should parse")
    }

    fn test_breaker_registry(names: &[&str]) -> BreakerRegistry {
        let mut registry = BreakerRegistry::default();
        for name in names {
            registry.insert((*name).to_owned(), make_breaker(name));
        }
        registry
    }

    fn test_app() -> App {
        let registry = test_breaker_registry(&["Aegis", "Chrono"]);
        let mut app = TestAppBuilder::new()
            .with_state_hierarchy()
            .with_resource::<ButtonInput<KeyCode>>()
            .insert_resource(InputConfig::default())
            .with_message::<ChangeState<MenuState>>()
            .insert_resource(SelectedBreaker::default())
            .with_resource::<RunSeed>()
            .with_resource::<SeedEntry>()
            .insert_resource(registry)
            .insert_resource(RunSetupSelection { index: 0 })
            .with_system(Update, handle_run_setup_input)
            .build();

        // Spawn cards matching registry
        app.world_mut().spawn(BreakerCard {
            breaker_name: "Aegis".to_owned(),
        });
        app.world_mut().spawn(BreakerCard {
            breaker_name: "Chrono".to_owned(),
        });

        // Navigate to MenuState
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Game);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Menu);
        app.update();
        app
    }

    fn press_key(app: &mut App, key: KeyCode) {
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .press(key);
        app.update();
    }

    #[test]
    fn down_press_advances_selection() {
        let mut app = test_app();
        press_key(&mut app, KeyCode::ArrowDown);

        let selection = app.world().resource::<RunSetupSelection>();
        assert_eq!(selection.index, 1);
    }

    #[test]
    fn up_press_wraps_selection() {
        let mut app = test_app();
        press_key(&mut app, KeyCode::ArrowUp);

        let selection = app.world().resource::<RunSetupSelection>();
        assert_eq!(selection.index, 1); // wraps from 0 to last (1)
    }

    #[test]
    fn confirm_transitions_to_teardown() {
        let mut app = test_app();
        press_key(&mut app, KeyCode::Enter);

        let msgs = app.world().resource::<Messages<ChangeState<MenuState>>>();
        assert!(
            msgs.iter_current_update_messages().count() > 0,
            "expected ChangeState<MenuState> message"
        );
    }

    #[test]
    fn confirm_sets_selected_breaker() {
        let mut app = test_app();
        // Index 0 = "Aegis" (sorted alphabetically)
        press_key(&mut app, KeyCode::Enter);

        let selected = app.world().resource::<SelectedBreaker>();
        assert_eq!(selected.0, "Aegis");
    }

    #[test]
    fn confirm_after_navigation_selects_correct_breaker() {
        let mut app = test_app();
        press_key(&mut app, KeyCode::ArrowDown);

        // Clear and re-press
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(KeyCode::ArrowDown);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();

        press_key(&mut app, KeyCode::Enter);

        let selected = app.world().resource::<SelectedBreaker>();
        assert_eq!(selected.0, "Chrono");
    }

    #[test]
    fn confirm_sets_run_seed_none_when_entry_empty() {
        let mut app = test_app();
        // SeedEntry default is empty
        press_key(&mut app, KeyCode::Enter);

        let seed = app.world().resource::<RunSeed>();
        assert_eq!(seed.0, None);
    }

    #[test]
    fn confirm_sets_run_seed_some_when_entry_has_value() {
        let mut app = test_app();
        app.world_mut().resource_mut::<SeedEntry>().value = "42".to_owned();
        press_key(&mut app, KeyCode::Enter);

        let seed = app.world().resource::<RunSeed>();
        assert_eq!(seed.0, Some(42));
    }

    #[test]
    fn confirm_sets_run_seed_zero_on_parse_failure() {
        let mut app = test_app();
        // u64 overflow — larger than u64::MAX
        app.world_mut().resource_mut::<SeedEntry>().value = "99999999999999999999999".to_owned();
        press_key(&mut app, KeyCode::Enter);

        let seed = app.world().resource::<RunSeed>();
        assert_eq!(seed.0, Some(0));
    }

    #[test]
    fn navigation_wraps_down() {
        let mut app = test_app();
        press_key(&mut app, KeyCode::ArrowDown);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .release(KeyCode::ArrowDown);
        app.world_mut()
            .resource_mut::<ButtonInput<KeyCode>>()
            .clear();

        press_key(&mut app, KeyCode::ArrowDown);

        let selection = app.world().resource::<RunSetupSelection>();
        assert_eq!(selection.index, 0); // wraps from 1 back to 0
    }
}
