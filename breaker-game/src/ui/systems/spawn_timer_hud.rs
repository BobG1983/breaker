//! System to spawn the timer HUD on node entry.

use bevy::prelude::*;

use crate::{
    run::node::NodeTimer,
    shared::CleanupOnNodeExit,
    ui::{
        components::{NodeTimerDisplay, StatusPanel},
        resources::TimerUiConfig,
    },
};

/// Spawns the timer display as a child of the [`StatusPanel`].
pub(crate) fn spawn_timer_hud(
    mut commands: Commands,
    config: Res<TimerUiConfig>,
    timer: Res<NodeTimer>,
    asset_server: Res<AssetServer>,
    existing: Query<(), With<NodeTimerDisplay>>,
    status_panel: Query<Entity, With<StatusPanel>>,
) {
    if !existing.is_empty() {
        return;
    }

    let Ok(panel) = status_panel.single() else {
        return;
    };

    let font: Handle<Font> = asset_server.load(&config.font_path);
    let display_secs = timer.remaining.ceil().max(0.0);

    commands.entity(panel).with_children(|parent| {
        parent
            .spawn((
                CleanupOnNodeExit,
                Node {
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            ))
            .with_child((
                NodeTimerDisplay,
                Text::new(format!("{display_secs:.0}")),
                TextFont {
                    font,
                    font_size: config.font_size,
                    ..default()
                },
                TextColor(config.color_for_fraction(1.0)),
            ));
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::components::StatusPanel;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<Font>()
            .insert_resource(TimerUiConfig::default())
            .insert_resource(NodeTimer {
                remaining: 60.0,
                total: 60.0,
            });
        // Spawn a StatusPanel for the HUD to parent under
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn((StatusPanel, Node::default()));
        })
        .add_systems(Update, spawn_timer_hud);
        app
    }

    #[test]
    fn spawn_creates_timer_display_entity() {
        let mut app = test_app();
        app.update();

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<NodeTimerDisplay>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn parent_has_cleanup_marker() {
        let mut app = test_app();
        app.update();

        let display_entity = app
            .world_mut()
            .query_filtered::<Entity, With<NodeTimerDisplay>>()
            .iter(app.world())
            .next()
            .expect("NodeTimerDisplay should exist");
        let parent = app
            .world()
            .get::<ChildOf>(display_entity)
            .expect("NodeTimerDisplay should have a parent");
        assert!(
            app.world()
                .get::<CleanupOnNodeExit>(parent.parent())
                .is_some(),
            "parent wrapper should have CleanupOnNodeExit"
        );
    }

    #[test]
    fn no_status_panel_no_spawn() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()))
            .init_asset::<Font>()
            .insert_resource(TimerUiConfig::default())
            .insert_resource(NodeTimer::default())
            .add_systems(Update, spawn_timer_hud);
        app.update();

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<NodeTimerDisplay>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 0);
    }

    #[test]
    fn no_double_spawn() {
        let mut app = test_app();
        app.update();
        app.update();

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<NodeTimerDisplay>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1, "should not double-spawn timer HUD");
    }

    #[test]
    fn text_shows_remaining_time() {
        let mut app = test_app();
        app.update();

        let texts: Vec<String> = app
            .world_mut()
            .query_filtered::<&Text, With<NodeTimerDisplay>>()
            .iter(app.world())
            .map(|t| t.0.clone())
            .collect();
        assert_eq!(texts.len(), 1);
        assert_eq!(texts[0], "60");
    }
}
