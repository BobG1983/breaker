//! Updates pause menu item colors based on the current selection.

use bevy::prelude::*;

use crate::state::pause::{components::PauseMenuItem, resources::PauseMenuSelection};

/// The highlight color applied to the currently selected pause menu item.
pub(super) const SELECTED_COLOR: Color = Color::srgb(0.4, 0.8, 1.0);
/// The normal color applied to unselected pause menu items.
pub(super) const NORMAL_COLOR: Color = Color::srgb(0.6, 0.6, 0.7);

/// Syncs `PauseMenuSelection` to `TextColor` on `PauseMenuItem` entities.
///
/// Selected item gets the highlight color; all others get the normal color.
pub(crate) fn update_pause_menu_colors(
    selection: Res<PauseMenuSelection>,
    mut query: Query<(&PauseMenuItem, &mut TextColor)>,
) {
    for (item, mut text_color) in &mut query {
        let color = if *item == selection.selected {
            SELECTED_COLOR
        } else {
            NORMAL_COLOR
        };
        *text_color = TextColor(color);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(PauseMenuSelection {
                selected: PauseMenuItem::Resume,
            })
            .add_systems(Update, update_pause_menu_colors);
        app
    }

    /// Spawns two pause menu item entities (Resume and Quit) with a default
    /// `TextColor::WHITE` so that any color change is observable.
    fn spawn_menu_items(app: &mut App) -> (Entity, Entity) {
        let resume = app
            .world_mut()
            .spawn((PauseMenuItem::Resume, TextColor(Color::WHITE)))
            .id();
        let quit = app
            .world_mut()
            .spawn((PauseMenuItem::Quit, TextColor(Color::WHITE)))
            .id();
        (resume, quit)
    }

    #[test]
    fn selected_resume_gets_highlight_color() {
        let mut app = test_app();
        let (resume, quit) = spawn_menu_items(&mut app);

        // Selection defaults to Resume
        app.update();

        let resume_color = app.world().entity(resume).get::<TextColor>().unwrap().0;
        let quit_color = app.world().entity(quit).get::<TextColor>().unwrap().0;

        assert_eq!(
            resume_color, SELECTED_COLOR,
            "Resume should have the selected highlight color"
        );
        assert_eq!(
            quit_color, NORMAL_COLOR,
            "Quit should have the normal color when Resume is selected"
        );
    }

    #[test]
    fn highlight_follows_selection_change_to_quit() {
        let mut app = test_app();
        let (resume, quit) = spawn_menu_items(&mut app);

        // Change selection to Quit before running the system
        app.world_mut()
            .resource_mut::<PauseMenuSelection>()
            .selected = PauseMenuItem::Quit;

        app.update();

        let resume_color = app.world().entity(resume).get::<TextColor>().unwrap().0;
        let quit_color = app.world().entity(quit).get::<TextColor>().unwrap().0;

        assert_eq!(
            quit_color, SELECTED_COLOR,
            "Quit should have the selected highlight color after selection change"
        );
        assert_eq!(
            resume_color, NORMAL_COLOR,
            "Resume should have the normal color when Quit is selected"
        );
    }

    #[test]
    fn only_selected_item_gets_highlight_color() {
        let mut app = test_app();
        let (resume, quit) = spawn_menu_items(&mut app);

        // Selection is Resume (default)
        app.update();

        let resume_color = app.world().entity(resume).get::<TextColor>().unwrap().0;
        let quit_color = app.world().entity(quit).get::<TextColor>().unwrap().0;

        // Resume is selected, so it gets the highlight
        assert_eq!(resume_color, SELECTED_COLOR);
        // Quit is NOT selected, so it gets the normal color
        assert_eq!(quit_color, NORMAL_COLOR);

        // Verify they are not the same
        assert_ne!(
            resume_color, quit_color,
            "Selected and unselected items must have different colors"
        );
    }
}
