use bevy::prelude::*;

use super::{super::spawn_run_end_screen, helpers::*};
use crate::{
    run::resources::{RunOutcome, RunState, RunStats},
    screen::run_end::RunEndScreen,
};

#[test]
fn spawn_creates_run_end_screen_entity() {
    let mut app = test_app(RunOutcome::Won);
    app.update();

    let count = app
        .world_mut()
        .query_filtered::<Entity, With<RunEndScreen>>()
        .iter(app.world())
        .count();
    assert_eq!(count, 1);
}

#[test]
fn won_shows_complete_text() {
    let mut app = test_app(RunOutcome::Won);
    app.update();

    let texts = collect_texts(&mut app);
    assert!(
        texts.iter().any(|t| t.contains("RUN COMPLETE")),
        "expected 'RUN COMPLETE' in texts: {texts:?}"
    );
    assert!(
        texts.iter().any(|t| t.contains("The bolt obeys")),
        "expected 'The bolt obeys' subtitle in texts: {texts:?}"
    );
}

#[test]
fn timer_expired_shows_times_up_text() {
    let mut app = test_app(RunOutcome::TimerExpired);
    app.update();

    let texts = collect_texts(&mut app);
    assert!(
        texts.iter().any(|t| t.contains("TIME'S UP")),
        "expected \"TIME'S UP\" in texts: {texts:?}"
    );
}

#[test]
fn in_progress_shows_run_ended_text() {
    let mut app = test_app(RunOutcome::InProgress);
    app.update();

    let screen_count = app
        .world_mut()
        .query_filtered::<Entity, With<RunEndScreen>>()
        .iter(app.world())
        .count();
    assert_eq!(
        screen_count, 1,
        "RunEndScreen entity should be spawned for InProgress fallback"
    );

    let texts = collect_texts(&mut app);
    assert!(
        texts.iter().any(|t| t.contains("RUN ENDED")),
        "expected 'RUN ENDED' in texts: {texts:?}"
    );
}

#[test]
fn lives_depleted_shows_signal_lost_text() {
    let mut app = test_app(RunOutcome::LivesDepleted);
    app.update();

    let texts = collect_texts(&mut app);
    assert!(
        texts.iter().any(|t| t.contains("SIGNAL LOST")),
        "expected 'SIGNAL LOST' in texts: {texts:?}"
    );
}

#[test]
fn displays_seed_value() {
    let stats = RunStats {
        seed: 42,
        ..Default::default()
    };
    let mut app = test_app_with_stats(RunOutcome::Won, stats);
    app.update();

    let texts = collect_texts(&mut app);
    assert!(
        texts.iter().any(|t| t.contains("42")),
        "expected seed '42' in texts: {texts:?}"
    );
}

#[test]
fn displays_large_seed_value() {
    let stats = RunStats {
        seed: 123_456_789,
        ..Default::default()
    };
    let mut app = test_app_with_stats(RunOutcome::Won, stats);
    app.update();

    let texts = collect_texts(&mut app);
    assert!(
        texts.iter().any(|t| t.contains("123456789")),
        "expected seed '123456789' in texts: {texts:?}"
    );
}

#[test]
fn displays_chip_names_from_chips_collected() {
    let stats = RunStats {
        chips_collected: vec![
            "Piercing Shot".to_string(),
            "Wide Breaker".to_string(),
            "Piercing Shot".to_string(),
        ],
        ..Default::default()
    };
    let mut app = test_app_with_stats(RunOutcome::Won, stats);
    app.update();

    let texts = collect_texts(&mut app);
    assert!(
        texts.iter().any(|t| t.contains("Piercing Shot")),
        "expected 'Piercing Shot' in texts: {texts:?}"
    );
    assert!(
        texts.iter().any(|t| t.contains("Wide Breaker")),
        "expected 'Wide Breaker' in texts: {texts:?}"
    );
}

#[test]
fn displays_empty_chip_list_gracefully() {
    let stats = RunStats {
        chips_collected: vec![],
        ..Default::default()
    };
    let mut app = test_app_with_stats(RunOutcome::Won, stats);
    app.update();

    // Should not panic and should still have the run end screen
    let count = app
        .world_mut()
        .query_filtered::<Entity, With<RunEndScreen>>()
        .iter(app.world())
        .count();
    assert_eq!(count, 1, "RunEndScreen should exist even with no chips");
}

#[test]
fn displays_outcome_without_stats_resource() {
    // This test creates an app WITHOUT inserting RunStats.
    // The system should use Option<Res<RunStats>> and not panic.
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .insert_resource(RunState {
            node_index: 0,
            outcome: RunOutcome::Won,
            ..default()
        })
        .add_systems(Update, spawn_run_end_screen);
    app.update();

    let count = app
        .world_mut()
        .query_filtered::<Entity, With<RunEndScreen>>()
        .iter(app.world())
        .count();
    assert_eq!(count, 1, "RunEndScreen should exist even without RunStats");

    let texts = collect_texts(&mut app);
    assert!(
        texts.iter().any(|t| t.contains("RUN COMPLETE")),
        "expected 'RUN COMPLETE' in texts even without RunStats: {texts:?}"
    );
}
