//! Updates the seed display text based on `SeedEntry` state.

use bevy::prelude::*;

use crate::state::menu::start_game::{components::SeedDisplay, resources::SeedEntry};

/// Syncs the [`SeedDisplay`] text with the current [`SeedEntry`] value and focus state.
///
/// Shows `[Random]` or `[12345]` when focused, `SEED: Random` or `SEED: 12345` when unfocused.
pub(crate) fn update_seed_display(
    seed_entry: Res<SeedEntry>,
    mut query: Query<(&mut Text, &mut TextColor), With<SeedDisplay>>,
) {
    for (mut text, mut color) in &mut query {
        let display_value = if seed_entry.value.is_empty() {
            "Random".to_owned()
        } else {
            seed_entry.value.clone()
        };

        if seed_entry.focused {
            **text = format!("SEED: [{display_value}]");
            *color = TextColor(Color::srgba(0.4, 0.8, 1.0, 1.0));
        } else {
            **text = format!("SEED: {display_value}");
            *color = TextColor(Color::srgba(0.5, 0.5, 0.6, 1.0));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        use crate::shared::test_utils::TestAppBuilder;
        TestAppBuilder::new()
            .with_resource::<SeedEntry>()
            .with_system(Update, update_seed_display)
            .build()
    }

    fn spawn_display(app: &mut App) -> Entity {
        app.world_mut()
            .spawn((
                SeedDisplay,
                Text::new("placeholder"),
                TextColor(Color::BLACK),
            ))
            .id()
    }

    #[test]
    fn shows_random_when_empty_unfocused() {
        let mut app = test_app();
        let entity = spawn_display(&mut app);
        app.update();

        let text = app.world().get::<Text>(entity).unwrap();
        assert_eq!(**text, "SEED: Random");
    }

    #[test]
    fn shows_value_when_typed_unfocused() {
        let mut app = test_app();
        let entity = spawn_display(&mut app);
        app.world_mut().resource_mut::<SeedEntry>().value = "42".to_owned();
        app.update();

        let text = app.world().get::<Text>(entity).unwrap();
        assert_eq!(**text, "SEED: 42");
    }

    #[test]
    fn shows_brackets_when_focused() {
        let mut app = test_app();
        let entity = spawn_display(&mut app);
        app.world_mut().resource_mut::<SeedEntry>().focused = true;
        app.update();

        let text = app.world().get::<Text>(entity).unwrap();
        assert_eq!(**text, "SEED: [Random]");
    }

    #[test]
    fn shows_value_with_brackets_when_focused() {
        let mut app = test_app();
        let entity = spawn_display(&mut app);
        app.world_mut().resource_mut::<SeedEntry>().focused = true;
        app.world_mut().resource_mut::<SeedEntry>().value = "999".to_owned();
        app.update();

        let text = app.world().get::<Text>(entity).unwrap();
        assert_eq!(**text, "SEED: [999]");
    }

    #[test]
    fn focused_color_is_highlight() {
        let mut app = test_app();
        let entity = spawn_display(&mut app);
        app.world_mut().resource_mut::<SeedEntry>().focused = true;
        app.update();

        let color = app.world().get::<TextColor>(entity).unwrap();
        assert_eq!(color.0, Color::srgba(0.4, 0.8, 1.0, 1.0));
    }

    #[test]
    fn unfocused_color_is_dim() {
        let mut app = test_app();
        let entity = spawn_display(&mut app);
        app.update();

        let color = app.world().get::<TextColor>(entity).unwrap();
        assert_eq!(color.0, Color::srgba(0.5, 0.5, 0.6, 1.0));
    }
}
