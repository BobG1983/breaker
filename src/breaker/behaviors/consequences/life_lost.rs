//! Life-lost consequence — observer, component, and HUD.

use bevy::prelude::*;

use crate::{run::messages::RunLost, ui::components::StatusPanel};

/// Consequence event triggered by bridge systems when a life should be lost.
#[derive(Event)]
pub struct LoseLifeRequested;

/// Number of remaining lives on the breaker entity.
#[derive(Component, Debug, Clone, Copy)]
pub struct LivesCount(pub u32);

/// Marker for the lives HUD display entity.
#[derive(Component, Debug)]
pub struct LivesDisplay;

/// Observer that handles life loss — decrements `LivesCount`, sends [`RunLost`]
/// when lives reach zero.
pub fn handle_life_lost(
    _trigger: On<LoseLifeRequested>,
    mut lives_query: Query<&mut LivesCount>,
    mut writer: MessageWriter<RunLost>,
) {
    for mut lives in &mut lives_query {
        if lives.0 == 0 {
            continue;
        }
        lives.0 -= 1;

        if lives.0 == 0 {
            writer.write(RunLost);
        }
    }
}

/// Spawns the lives display as a child of the [`StatusPanel`].
pub fn spawn_lives_display(
    mut commands: Commands,
    lives_query: Query<&LivesCount>,
    existing: Query<(), With<LivesDisplay>>,
    status_panel: Query<Entity, With<StatusPanel>>,
) {
    if !existing.is_empty() {
        return;
    }

    let Ok(lives) = lives_query.single() else {
        return;
    };

    let Ok(panel) = status_panel.single() else {
        return;
    };

    commands.entity(panel).with_children(|parent| {
        parent
            .spawn((
                Node {
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                    border_radius: BorderRadius::all(Val::Px(6.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            ))
            .with_child((
                LivesDisplay,
                Text::new(format_lives(lives.0)),
                TextFont {
                    font_size: 24.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
    });
}

/// Updates the lives display text to match the current `LivesCount`.
pub fn update_lives_display(
    lives_query: Query<&LivesCount>,
    mut display_query: Query<&mut Text, With<LivesDisplay>>,
) {
    let Ok(lives) = lives_query.single() else {
        return;
    };

    for mut text in &mut display_query {
        text.0 = format_lives(lives.0);
    }
}

/// Returns the HUD display string for the current life count.
///
/// Format: `"Lives: {count}"`. Both `spawn_lives_display` and
/// `update_lives_display` call this function — change the format here only.
fn format_lives(count: u32) -> String {
    format!("Lives: {count}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ui::components::StatusPanel;

    #[derive(Resource, Default)]
    struct CapturedRunLost(u32);

    fn capture_run_lost(mut reader: MessageReader<RunLost>, mut captured: ResMut<CapturedRunLost>) {
        for _msg in reader.read() {
            captured.0 += 1;
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<RunLost>();
        app.init_resource::<CapturedRunLost>();
        app.add_observer(handle_life_lost);
        app.add_systems(FixedUpdate, capture_run_lost);
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    #[test]
    fn lose_life_decrements_count() {
        let mut app = test_app();
        let entity = app.world_mut().spawn(LivesCount(3)).id();

        app.world_mut().commands().trigger(LoseLifeRequested);
        app.world_mut().flush();

        let lives = app.world().get::<LivesCount>(entity).unwrap();
        assert_eq!(lives.0, 2);
    }

    #[test]
    fn lose_last_life_sends_run_lost() {
        let mut app = test_app();
        app.world_mut().spawn(LivesCount(1));

        app.world_mut().commands().trigger(LoseLifeRequested);
        app.world_mut().flush();
        tick(&mut app);

        let captured = app.world().resource::<CapturedRunLost>();
        assert_eq!(captured.0, 1, "should send RunLost when lives reach zero");
    }

    #[test]
    fn lose_life_at_zero_does_not_send_run_lost() {
        let mut app = test_app();
        app.world_mut().spawn(LivesCount(0));

        app.world_mut().commands().trigger(LoseLifeRequested);
        app.world_mut().flush();
        tick(&mut app);

        let captured = app.world().resource::<CapturedRunLost>();
        assert_eq!(
            captured.0, 0,
            "should not send RunLost when already at zero"
        );
    }

    #[test]
    fn multiple_lives_lost_sequentially() {
        let mut app = test_app();
        let entity = app.world_mut().spawn(LivesCount(3)).id();

        app.world_mut().commands().trigger(LoseLifeRequested);
        app.world_mut().flush();
        assert_eq!(app.world().get::<LivesCount>(entity).unwrap().0, 2);

        app.world_mut().commands().trigger(LoseLifeRequested);
        app.world_mut().flush();
        assert_eq!(app.world().get::<LivesCount>(entity).unwrap().0, 1);

        app.world_mut().commands().trigger(LoseLifeRequested);
        app.world_mut().flush();
        tick(&mut app);

        assert_eq!(app.world().get::<LivesCount>(entity).unwrap().0, 0);

        let captured = app.world().resource::<CapturedRunLost>();
        assert_eq!(captured.0, 1, "should send exactly one RunLost");
    }

    #[test]
    fn spawn_lives_display_creates_hud() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.world_mut().spawn(LivesCount(3));
        app.world_mut().spawn((StatusPanel, Node::default()));
        app.add_systems(Update, spawn_lives_display);
        app.update();

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<LivesDisplay>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 1);
    }

    #[test]
    fn spawn_lives_display_no_lives_no_hud() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.world_mut().spawn((StatusPanel, Node::default()));
        app.add_systems(Update, spawn_lives_display);
        app.update();

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<LivesDisplay>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 0);
    }

    #[test]
    fn spawn_lives_display_no_panel_no_hud() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.world_mut().spawn(LivesCount(3));
        app.add_systems(Update, spawn_lives_display);
        app.update();

        let count = app
            .world_mut()
            .query_filtered::<Entity, With<LivesDisplay>>()
            .iter(app.world())
            .count();
        assert_eq!(count, 0);
    }

    #[test]
    fn update_lives_display_updates_text() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let lives_entity = app.world_mut().spawn(LivesCount(3)).id();
        app.world_mut().spawn((
            LivesDisplay,
            Text::new("Lives: 3".to_owned()),
            TextFont::default(),
            TextColor(Color::WHITE),
            Node::default(),
        ));
        app.add_systems(Update, update_lives_display);
        app.update();

        // Change lives
        app.world_mut()
            .get_mut::<LivesCount>(lives_entity)
            .unwrap()
            .0 = 1;
        app.update();

        let text = app
            .world_mut()
            .query_filtered::<&Text, With<LivesDisplay>>()
            .iter(app.world())
            .next()
            .unwrap();
        assert_eq!(text.0, "Lives: 1");
    }

    #[test]
    fn lives_display_is_child_of_status_panel() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.world_mut().spawn(LivesCount(3));
        let panel = app.world_mut().spawn((StatusPanel, Node::default())).id();
        app.add_systems(Update, spawn_lives_display);
        app.update();

        let display_entity = app
            .world_mut()
            .query_filtered::<Entity, With<LivesDisplay>>()
            .iter(app.world())
            .next()
            .expect("LivesDisplay should exist");

        // LivesDisplay text → wrapper parent → StatusPanel grandparent
        let wrapper = app
            .world()
            .get::<ChildOf>(display_entity)
            .expect("LivesDisplay should have a parent");
        let grandparent = app
            .world()
            .get::<ChildOf>(wrapper.parent())
            .expect("wrapper should have a parent");
        assert_eq!(
            grandparent.parent(),
            panel,
            "lives wrapper should be a child of StatusPanel"
        );
    }
}
