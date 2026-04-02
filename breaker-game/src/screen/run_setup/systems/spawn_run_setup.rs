//! System to spawn the breaker selection screen UI.

use bevy::prelude::*;

use crate::{
    breaker::BreakerRegistry,
    screen::run_setup::{
        components::{BreakerCard, RunSetupScreen, SeedDisplay},
        resources::{RunSetupSelection, SeedEntry},
    },
};

/// Spawns the breaker selection UI.
///
/// Reads [`BreakerRegistry`] to display one card per available breaker.
pub(crate) fn spawn_run_setup(mut commands: Commands, registry: Res<BreakerRegistry>) {
    let mut names: Vec<&String> = registry.names().collect();
    names.sort();

    commands.insert_resource(RunSetupSelection { index: 0 });
    commands.insert_resource(SeedEntry::default());

    commands
        .spawn((
            RunSetupScreen,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(40.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new("SELECT BREAKER"),
                TextFont {
                    font_size: 72.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            // Cards container
            parent
                .spawn(Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Px(16.0),
                    ..default()
                })
                .with_children(|cards| {
                    for name in &names {
                        let description = description_for(name);

                        cards
                            .spawn((
                                BreakerCard {
                                    breaker_name: (*name).clone(),
                                },
                                Button,
                                Node {
                                    padding: UiRect::axes(Val::Px(40.0), Val::Px(16.0)),
                                    flex_direction: FlexDirection::Column,
                                    align_items: AlignItems::Center,
                                    row_gap: Val::Px(8.0),
                                    ..default()
                                },
                                BackgroundColor(Color::NONE),
                            ))
                            .with_children(|card| {
                                card.spawn((
                                    Text::new(name.to_uppercase()),
                                    TextFont {
                                        font_size: 48.0,
                                        ..default()
                                    },
                                    TextColor(Color::WHITE),
                                ));

                                card.spawn((
                                    Text::new(description),
                                    TextFont {
                                        font_size: 24.0,
                                        ..default()
                                    },
                                    TextColor(Color::srgba(0.6, 0.6, 0.7, 1.0)),
                                ));
                            });
                    }
                });

            // Seed display
            parent.spawn((
                SeedDisplay,
                Text::new("SEED: Random"),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::srgba(0.5, 0.5, 0.6, 1.0)),
            ));

            // Prompt
            parent.spawn((
                Text::new("Press Enter to confirm  |  Tab to edit seed"),
                TextFont {
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgba(0.5, 0.5, 0.5, 1.0)),
            ));
        });
}

/// Returns a brief description for a known breaker name.
// TODO(phase-7): load descriptions from BreakerDefinition RON field
fn description_for(name: &str) -> &'static str {
    match name {
        "Aegis" => "Lives-based. Lose a life on bolt-loss. Bump boosts bolt speed.",
        "Chrono" => "Time-penalty. Bolt-loss costs time. Bump boosts bolt speed.",
        "Prism" => "Multi-bolt. Perfect bump spawns extra bolts.",
        _ => "Unknown breaker.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::breaker::definition::BreakerDefinition;

    fn make_breaker(name: &str) -> BreakerDefinition {
        ron::de::from_str(&format!(
            r#"(name: "{name}", life_pool: None, effects: [])"#,
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

    fn test_app(registry: BreakerRegistry) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(registry)
            .add_systems(Update, spawn_run_setup);
        app
    }

    #[test]
    fn spawn_creates_screen_entity() {
        let mut app = test_app(test_breaker_registry(&["Aegis"]));
        app.update();

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<RunSetupScreen>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn cards_match_registry_count() {
        let mut app = test_app(test_breaker_registry(&["Aegis", "Chrono"]));
        app.update();

        let count = app
            .world_mut()
            .query::<&BreakerCard>()
            .iter(app.world())
            .count();
        assert_eq!(count, 2);
    }

    #[test]
    fn selection_resource_inserted() {
        let mut app = test_app(test_breaker_registry(&["Aegis"]));
        app.update();

        let selection = app.world().resource::<RunSetupSelection>();
        assert_eq!(selection.index, 0);
    }

    #[test]
    fn empty_registry_spawns_no_cards() {
        let mut app = test_app(BreakerRegistry::default());
        app.update();

        let count = app
            .world_mut()
            .query::<&BreakerCard>()
            .iter(app.world())
            .count();
        assert_eq!(count, 0);

        // Screen still spawns
        let screen_count = app
            .world_mut()
            .query_filtered::<Entity, With<RunSetupScreen>>()
            .iter(app.world())
            .count();
        assert_eq!(screen_count, 1);
    }
}
