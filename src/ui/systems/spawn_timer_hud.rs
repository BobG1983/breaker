//! System to spawn the timer HUD on node entry.

use bevy::prelude::*;

use crate::{
    run::node::NodeTimer,
    shared::CleanupOnNodeExit,
    ui::{components::NodeTimerDisplay, resources::TimerUiConfig},
};

/// Spawns the timer display UI at the top of the screen.
pub fn spawn_timer_hud(
    mut commands: Commands,
    config: Res<TimerUiConfig>,
    timer: Res<NodeTimer>,
    asset_server: Res<AssetServer>,
) {
    let font: Handle<Font> = asset_server.load(&config.font_path);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let secs = timer.remaining.ceil().max(0.0) as u32;

    commands.spawn((
        NodeTimerDisplay,
        CleanupOnNodeExit,
        Text::new(format!("{secs}")),
        TextFont {
            font,
            font_size: config.font_size,
            ..default()
        },
        TextColor(config.color_for_fraction(1.0)),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(16.0),
            left: Val::Percent(50.0),
            ..default()
        },
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AssetPlugin::default()));
        app.init_asset::<Font>();
        app.insert_resource(TimerUiConfig::default());
        app.insert_resource(NodeTimer {
            remaining: 60.0,
            total: 60.0,
        });
        app.add_systems(Update, spawn_timer_hud);
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
    fn entity_has_cleanup_marker() {
        let mut app = test_app();
        app.update();

        let count = app
            .world_mut()
            .query_filtered::<Entity, (With<NodeTimerDisplay>, With<CleanupOnNodeExit>)>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1);
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
