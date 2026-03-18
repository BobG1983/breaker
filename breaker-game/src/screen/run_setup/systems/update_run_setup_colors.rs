//! Updates breaker card colors based on the current selection.

use bevy::prelude::*;

use crate::screen::run_setup::{components::BreakerCard, resources::RunSetupSelection};

const SELECTED_COLOR: Color = Color::srgb(0.4, 0.8, 1.0);
const NORMAL_COLOR: Color = Color::srgb(0.6, 0.6, 0.7);

/// Updates breaker card text colors based on the current selection index.
pub(crate) fn update_run_setup_colors(
    selection: Res<RunSetupSelection>,
    cards: Query<(Entity, &BreakerCard)>,
    children_query: Query<&Children>,
    mut text_colors: Query<&mut TextColor>,
) {
    // Sort cards by archetype name to match selection index
    let mut sorted_cards: Vec<(Entity, &BreakerCard)> = cards.iter().collect();
    sorted_cards.sort_by(|a, b| a.1.archetype_name.cmp(&b.1.archetype_name));

    for (i, (card_entity, _)) in sorted_cards.iter().enumerate() {
        let color = if i == selection.index {
            SELECTED_COLOR
        } else {
            NORMAL_COLOR
        };

        // Update text colors on card's children
        if let Ok(children) = children_query.get(*card_entity) {
            for child in children.iter() {
                if let Ok(mut text_color) = text_colors.get_mut(child) {
                    *text_color = TextColor(color);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app(selection_index: usize) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(RunSetupSelection {
                index: selection_index,
            })
            .add_systems(Update, update_run_setup_colors);
        app
    }

    /// Spawns a card with a text child. Returns (card, text) entities.
    fn spawn_card(app: &mut App, name: &str) -> (Entity, Entity) {
        let text_entity = app.world_mut().spawn(TextColor(Color::BLACK)).id();
        let card_entity = app
            .world_mut()
            .spawn(BreakerCard {
                archetype_name: name.to_owned(),
            })
            .id();
        app.world_mut()
            .entity_mut(card_entity)
            .add_child(text_entity);
        (card_entity, text_entity)
    }

    #[test]
    fn selected_card_gets_selected_color() {
        let mut app = test_app(0);
        let (_, text) = spawn_card(&mut app, "Aegis");
        spawn_card(&mut app, "Chrono");
        app.update();

        let color = app.world().get::<TextColor>(text).unwrap();
        assert_eq!(color.0, SELECTED_COLOR);
    }

    #[test]
    fn unselected_card_gets_normal_color() {
        let mut app = test_app(0);
        spawn_card(&mut app, "Aegis");
        let (_, text) = spawn_card(&mut app, "Chrono");
        app.update();

        let color = app.world().get::<TextColor>(text).unwrap();
        assert_eq!(color.0, NORMAL_COLOR);
    }

    #[test]
    fn selection_change_updates_colors() {
        let mut app = test_app(0);
        let (_, text_a) = spawn_card(&mut app, "Aegis");
        let (_, text_c) = spawn_card(&mut app, "Chrono");
        app.update();

        // Change selection to index 1 (Chrono)
        app.world_mut().resource_mut::<RunSetupSelection>().index = 1;
        app.update();

        let color_a = app.world().get::<TextColor>(text_a).unwrap();
        let color_c = app.world().get::<TextColor>(text_c).unwrap();
        assert_eq!(color_a.0, NORMAL_COLOR);
        assert_eq!(color_c.0, SELECTED_COLOR);
    }

    #[test]
    fn cards_sorted_alphabetically() {
        // Spawn in reverse alphabetical order
        let mut app = test_app(0);
        let (_, text_c) = spawn_card(&mut app, "Chrono");
        let (_, text_a) = spawn_card(&mut app, "Aegis");
        app.update();

        // Index 0 = first alphabetically = Aegis
        let color_a = app.world().get::<TextColor>(text_a).unwrap();
        let color_c = app.world().get::<TextColor>(text_c).unwrap();
        assert_eq!(color_a.0, SELECTED_COLOR);
        assert_eq!(color_c.0, NORMAL_COLOR);
    }
}
