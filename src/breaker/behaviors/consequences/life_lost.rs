//! Life-lost consequence — observer, component, and HUD.

use bevy::prelude::*;

use crate::{
    run::resources::{RunOutcome, RunState},
    shared::{CleanupOnRunEnd, GameState},
};

/// Consequence event triggered by bridge systems when a life should be lost.
#[derive(Event)]
pub struct LoseLifeRequested;

/// Number of remaining lives on the breaker entity.
#[derive(Component, Debug, Clone, Copy)]
pub struct LivesCount(pub u32);

/// Marker for the lives HUD display entity.
#[derive(Component, Debug)]
pub struct LivesDisplay;

/// Observer that handles life loss — decrements `LivesCount`, ends run at zero.
pub fn handle_life_lost(
    _trigger: On<LoseLifeRequested>,
    mut lives_query: Query<&mut LivesCount>,
    mut run_state: ResMut<RunState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for mut lives in &mut lives_query {
        if lives.0 == 0 {
            continue;
        }
        lives.0 -= 1;

        if lives.0 == 0 && run_state.outcome == RunOutcome::InProgress {
            run_state.outcome = RunOutcome::Lost;
            next_state.set(GameState::RunEnd);
        }
    }
}

/// Spawns the lives display HUD entity.
pub fn spawn_lives_display(
    mut commands: Commands,
    lives_query: Query<&LivesCount>,
    existing: Query<Entity, With<LivesDisplay>>,
) {
    if existing.iter().next().is_some() {
        return;
    }

    let Ok(lives) = lives_query.single() else {
        return;
    };

    commands
        .spawn((
            CleanupOnRunEnd,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(16.0),
                right: Val::Px(24.0),
                padding: UiRect::axes(Val::Px(12.0), Val::Px(4.0)),
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
        ))
        .with_child((
            LivesDisplay,
            Text::new(format_lives(lives.0)),
            TextFont {
                font_size: 28.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
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

fn format_lives(count: u32) -> String {
    format!("Lives: {count}")
}

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;

    use super::*;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<GameState>();
        app.insert_resource(RunState {
            node_index: 0,
            outcome: RunOutcome::InProgress,
        });
        app.add_observer(handle_life_lost);
        app
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
    fn lose_last_life_ends_run() {
        let mut app = test_app();
        app.world_mut().spawn(LivesCount(1));

        app.world_mut().commands().trigger(LoseLifeRequested);
        app.world_mut().flush();

        let run_state = app.world().resource::<RunState>();
        assert_eq!(run_state.outcome, RunOutcome::Lost);

        let next = app.world().resource::<NextState<GameState>>();
        assert!(
            format!("{next:?}").contains("RunEnd"),
            "expected RunEnd, got: {next:?}"
        );
    }

    #[test]
    fn lose_life_at_zero_does_not_double_end() {
        let mut app = test_app();
        app.world_mut().spawn(LivesCount(0));

        app.world_mut().commands().trigger(LoseLifeRequested);
        app.world_mut().flush();

        let run_state = app.world().resource::<RunState>();
        assert_eq!(run_state.outcome, RunOutcome::InProgress);
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
        assert_eq!(app.world().get::<LivesCount>(entity).unwrap().0, 0);

        let run_state = app.world().resource::<RunState>();
        assert_eq!(run_state.outcome, RunOutcome::Lost);
    }

    #[test]
    fn spawn_lives_display_creates_hud() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.world_mut().spawn(LivesCount(3));
        app.add_systems(Startup, spawn_lives_display);
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
        app.add_systems(Startup, spawn_lives_display);
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
    fn lives_display_parent_has_cleanup_marker() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.world_mut().spawn(LivesCount(3));
        app.add_systems(Startup, spawn_lives_display);
        app.update();

        // LivesDisplay is a child; CleanupOnRunEnd is on the parent wrapper
        let display_entity = app
            .world_mut()
            .query_filtered::<Entity, With<LivesDisplay>>()
            .iter(app.world())
            .next()
            .expect("LivesDisplay should exist");
        let parent = app
            .world()
            .get::<ChildOf>(display_entity)
            .expect("LivesDisplay should have a parent");
        assert!(
            app.world()
                .get::<CleanupOnRunEnd>(parent.parent())
                .is_some(),
            "parent wrapper should have CleanupOnRunEnd"
        );
    }
}
