//! Handles keyboard input on the breaker selection screen.

use bevy::prelude::*;

use crate::{
    behaviors::ArchetypeRegistry,
    input::InputConfig,
    screen::run_setup::{components::BreakerCard, resources::RunSetupSelection},
    shared::{GameState, SelectedArchetype},
};

/// Handles keyboard navigation and confirmation on the run setup screen.
///
/// Reads `ButtonInput<KeyCode>` directly (same pattern as main menu) because
/// this screen runs in `Update` while `InputActions` clears in `FixedPostUpdate`.
pub(crate) fn handle_run_setup_input(
    keys: Res<ButtonInput<KeyCode>>,
    config: Res<InputConfig>,
    registry: Res<ArchetypeRegistry>,
    mut selection: ResMut<RunSetupSelection>,
    mut selected_archetype: ResMut<SelectedArchetype>,
    mut next_state: ResMut<NextState<GameState>>,
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
        let mut sorted_names: Vec<&String> = registry.archetypes.keys().collect();
        sorted_names.sort();

        if let Some(name) = sorted_names.get(selection.index) {
            selected_archetype.0.clone_from(name);
        }

        next_state.set(GameState::Playing);
    }
}

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;

    use super::*;
    use crate::behaviors::definition::{ArchetypeDefinition, BreakerStatOverrides};

    fn make_archetype(name: &str) -> ArchetypeDefinition {
        ArchetypeDefinition {
            name: name.to_owned(),
            stat_overrides: BreakerStatOverrides::default(),
            life_pool: None,
            behaviors: vec![],
        }
    }

    fn test_registry(names: &[&str]) -> ArchetypeRegistry {
        let mut registry = ArchetypeRegistry::default();
        for name in names {
            registry
                .archetypes
                .insert((*name).to_owned(), make_archetype(name));
        }
        registry
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_resource::<ButtonInput<KeyCode>>()
            .insert_resource(InputConfig::default())
            .init_state::<GameState>()
            .insert_resource(SelectedArchetype::default());

        let registry = test_registry(&["Aegis", "Chrono"]);
        app.insert_resource(registry)
            .insert_resource(RunSetupSelection { index: 0 });

        // Spawn cards matching registry
        app.world_mut().spawn(BreakerCard {
            archetype_name: "Aegis".to_owned(),
        });
        app.world_mut().spawn(BreakerCard {
            archetype_name: "Chrono".to_owned(),
        });

        app.add_systems(Update, handle_run_setup_input);
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
    fn confirm_transitions_to_playing() {
        let mut app = test_app();
        press_key(&mut app, KeyCode::Enter);

        let next = app.world().resource::<NextState<GameState>>();
        assert!(
            format!("{next:?}").contains("Playing"),
            "expected Playing, got: {next:?}"
        );
    }

    #[test]
    fn confirm_sets_selected_archetype() {
        let mut app = test_app();
        // Index 0 = "Aegis" (sorted alphabetically)
        press_key(&mut app, KeyCode::Enter);

        let selected = app.world().resource::<SelectedArchetype>();
        assert_eq!(selected.0, "Aegis");
    }

    #[test]
    fn confirm_after_navigation_selects_correct_archetype() {
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

        let selected = app.world().resource::<SelectedArchetype>();
        assert_eq!(selected.0, "Chrono");
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
