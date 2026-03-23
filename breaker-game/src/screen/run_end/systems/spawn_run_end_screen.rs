//! System to spawn the run-end screen UI.

use bevy::{ecs::hierarchy::ChildSpawnerCommands, prelude::*};
use tracing::info;

use crate::{
    run::resources::{HighlightKind, RunOutcome, RunState, RunStats},
    screen::run_end::RunEndScreen,
};

/// Label color for stat rows and section headers.
const LABEL_COLOR: Color = Color::srgba(0.7, 0.7, 0.7, 1.0);

/// Spawns the run-end UI showing the run outcome and optional stats summary.
pub(crate) fn spawn_run_end_screen(
    mut commands: Commands,
    run_state: Res<RunState>,
    stats: Option<Res<RunStats>>,
) {
    info!("run ended");
    let (title, subtitle) = match run_state.outcome {
        RunOutcome::Won => ("RUN COMPLETE", "The bolt obeys. For now."),
        RunOutcome::TimerExpired => ("TIME'S UP", "Almost had it."),
        RunOutcome::LivesDepleted => ("SIGNAL LOST", "Almost had it."),
        RunOutcome::InProgress => ("RUN ENDED", ""),
    };

    commands
        .spawn((
            RunEndScreen,
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(43.0),
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(title),
                TextFont {
                    font_size: 130.0,
                    ..default()
                },
                TextColor(Color::WHITE),
            ));

            if !subtitle.is_empty() {
                parent.spawn((
                    Text::new(subtitle),
                    TextFont {
                        font_size: 50.0,
                        ..default()
                    },
                    TextColor(LABEL_COLOR),
                ));
            }

            if let Some(stats) = &stats {
                spawn_stats_section(parent, stats);
                spawn_flux_section(parent, stats);
                spawn_highlights_section(parent, stats);
                spawn_chips_section(parent, stats);
                spawn_seed_section(parent, stats);
            }

            parent.spawn((
                Text::new("Press Enter to continue"),
                TextFont {
                    font_size: 43.0,
                    ..default()
                },
                TextColor(Color::srgba(0.5, 0.5, 0.5, 1.0)),
            ));
        });
}

/// Spawns stat label:value text nodes for the core run statistics.
fn spawn_stats_section(parent: &mut ChildSpawnerCommands<'_>, stats: &RunStats) {
    let stat_entries = [
        ("Nodes Cleared", stats.nodes_cleared.to_string()),
        ("Cells Destroyed", stats.cells_destroyed.to_string()),
        ("Bumps", stats.bumps_performed.to_string()),
        ("Flawless Bumps", stats.perfect_bumps.to_string()),
        ("Bolts Lost", stats.bolts_lost.to_string()),
    ];

    for (label, value) in &stat_entries {
        parent.spawn((
            Text::new(format!("{label}: {value}")),
            TextFont {
                font_size: 28.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    }
}

/// Spawns a text node showing total Flux earned.
fn spawn_flux_section(parent: &mut ChildSpawnerCommands<'_>, stats: &RunStats) {
    let flux = stats.flux_earned();
    parent.spawn((
        Text::new(format!("Flux Earned: {flux}")),
        TextFont {
            font_size: 28.0,
            ..default()
        },
        TextColor(Color::WHITE),
    ));
}

/// Spawns text nodes for up to 3 run highlights.
fn spawn_highlights_section(parent: &mut ChildSpawnerCommands<'_>, stats: &RunStats) {
    for highlight in stats.highlights.iter().take(3) {
        let text = match &highlight.kind {
            HighlightKind::ClutchClear => {
                format!("Clutch Clear - Node {}", highlight.node_index)
            }
            HighlightKind::NoDamageNode => {
                format!("No Damage - Node {}", highlight.node_index)
            }
            HighlightKind::FastClear => {
                format!("Fast Clear - Node {}", highlight.node_index)
            }
            HighlightKind::PerfectStreak => {
                format!("Perfect Streak x{:.0}", highlight.value)
            }
            HighlightKind::MassDestruction => {
                format!("Mass Destruction - Node {}", highlight.node_index)
            }
            HighlightKind::FirstEvolution => {
                format!("First Evolution - Node {}", highlight.node_index)
            }
        };
        parent.spawn((
            Text::new(text),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(LABEL_COLOR),
        ));
    }
}

/// Spawns text nodes for each chip collected during the run.
fn spawn_chips_section(parent: &mut ChildSpawnerCommands<'_>, stats: &RunStats) {
    for name in &stats.chips_collected {
        parent.spawn((
            Text::new(name.clone()),
            TextFont {
                font_size: 24.0,
                ..default()
            },
            TextColor(Color::WHITE),
        ));
    }
}

/// Spawns a text node showing the run seed.
fn spawn_seed_section(parent: &mut ChildSpawnerCommands<'_>, stats: &RunStats) {
    parent.spawn((
        Text::new(format!("Seed: {}", stats.seed)),
        TextFont {
            font_size: 20.0,
            ..default()
        },
        TextColor(LABEL_COLOR),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::run::resources::{HighlightKind, RunHighlight, RunStats};

    fn test_app(outcome: RunOutcome) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(RunState {
                node_index: 0,
                outcome,
                ..default()
            })
            .add_systems(Update, spawn_run_end_screen);
        app
    }

    fn test_app_with_stats(outcome: RunOutcome, stats: RunStats) -> App {
        let mut app = test_app(outcome);
        app.insert_resource(stats);
        app
    }

    /// Collects all `Text` component values from the world.
    fn collect_texts(app: &mut App) -> Vec<String> {
        app.world_mut()
            .query::<&Text>()
            .iter(app.world())
            .map(|t| t.0.clone())
            .collect()
    }

    // ---- Existing behavior tests (regression) ----

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

    // ---- Behavior 2: Stats grid from RunStats ----

    #[test]
    fn displays_nodes_cleared_from_stats() {
        let stats = RunStats {
            nodes_cleared: 5,
            cells_destroyed: 42,
            bumps_performed: 20,
            perfect_bumps: 8,
            bolts_lost: 3,
            time_elapsed: 125.5,
            ..Default::default()
        };
        let mut app = test_app_with_stats(RunOutcome::Won, stats);
        app.update();

        let texts = collect_texts(&mut app);
        assert!(
            texts.iter().any(|t| t.contains('5')),
            "expected nodes_cleared '5' in texts: {texts:?}"
        );
    }

    #[test]
    fn displays_cells_destroyed_from_stats() {
        let stats = RunStats {
            nodes_cleared: 5,
            cells_destroyed: 42,
            bumps_performed: 20,
            perfect_bumps: 8,
            bolts_lost: 3,
            time_elapsed: 125.5,
            ..Default::default()
        };
        let mut app = test_app_with_stats(RunOutcome::Won, stats);
        app.update();

        let texts = collect_texts(&mut app);
        assert!(
            texts.iter().any(|t| t.contains("42")),
            "expected cells_destroyed '42' in texts: {texts:?}"
        );
    }

    #[test]
    fn displays_bumps_performed_from_stats() {
        let stats = RunStats {
            nodes_cleared: 5,
            cells_destroyed: 42,
            bumps_performed: 20,
            perfect_bumps: 8,
            bolts_lost: 3,
            time_elapsed: 125.5,
            ..Default::default()
        };
        let mut app = test_app_with_stats(RunOutcome::Won, stats);
        app.update();

        let texts = collect_texts(&mut app);
        assert!(
            texts.iter().any(|t| t.contains("20")),
            "expected bumps_performed '20' in texts: {texts:?}"
        );
    }

    #[test]
    fn displays_perfect_bumps_from_stats() {
        let stats = RunStats {
            nodes_cleared: 5,
            cells_destroyed: 42,
            bumps_performed: 20,
            perfect_bumps: 8,
            bolts_lost: 3,
            time_elapsed: 125.5,
            ..Default::default()
        };
        let mut app = test_app_with_stats(RunOutcome::Won, stats);
        app.update();

        let texts = collect_texts(&mut app);
        assert!(
            texts.iter().any(|t| t.contains('8')),
            "expected perfect_bumps '8' in texts: {texts:?}"
        );
    }

    #[test]
    fn displays_bolts_lost_from_stats() {
        let stats = RunStats {
            nodes_cleared: 5,
            cells_destroyed: 42,
            bumps_performed: 20,
            perfect_bumps: 8,
            bolts_lost: 3,
            time_elapsed: 125.5,
            ..Default::default()
        };
        let mut app = test_app_with_stats(RunOutcome::Won, stats);
        app.update();

        let texts = collect_texts(&mut app);
        assert!(
            texts.iter().any(|t| t.contains('3')),
            "expected bolts_lost '3' in texts: {texts:?}"
        );
    }

    // ---- Behavior 3: Flux earned ----

    #[test]
    fn displays_flux_earned_value() {
        // flux_earned = (5*10) + (8*2) + (0*25) - (3*3) = 50 + 16 + 0 - 9 = 57
        let stats = RunStats {
            nodes_cleared: 5,
            perfect_bumps: 8,
            evolutions_performed: 0,
            bolts_lost: 3,
            ..Default::default()
        };
        let expected_flux = stats.flux_earned();
        assert_eq!(expected_flux, 57, "sanity check: flux_earned formula");

        let mut app = test_app_with_stats(RunOutcome::Won, stats);
        app.update();

        let texts = collect_texts(&mut app);
        assert!(
            texts.iter().any(|t| t.contains("57")),
            "expected flux earned '57' in texts: {texts:?}"
        );
    }

    #[test]
    fn displays_zero_flux_when_penalty_exceeds_bonuses() {
        let stats = RunStats {
            nodes_cleared: 0,
            perfect_bumps: 0,
            evolutions_performed: 0,
            bolts_lost: 10,
            ..Default::default()
        };
        assert_eq!(stats.flux_earned(), 0, "sanity check: flux floors at 0");

        let mut app = test_app_with_stats(RunOutcome::Won, stats);
        app.update();

        let texts = collect_texts(&mut app);
        assert!(
            texts.iter().any(|t| t.contains('0')),
            "expected flux '0' in texts: {texts:?}"
        );
    }

    // ---- Behavior 4: Highlights (up to 3) ----

    #[test]
    fn displays_highlight_entries() {
        let stats = RunStats {
            highlights: vec![
                RunHighlight {
                    kind: HighlightKind::ClutchClear,
                    node_index: 3,
                    value: 1.5,
                },
                RunHighlight {
                    kind: HighlightKind::NoDamageNode,
                    node_index: 1,
                    value: 0.0,
                },
            ],
            ..Default::default()
        };
        let mut app = test_app_with_stats(RunOutcome::Won, stats);
        app.update();

        let texts = collect_texts(&mut app);
        // At least 2 text nodes should reference highlight content.
        // The exact label format is up to implementation, but each highlight
        // should produce at least one text node.
        let highlight_related_count = texts
            .iter()
            .filter(|t| t.contains("Clutch") || t.contains("No Damage") || t.contains("NoDamage"))
            .count();
        assert!(
            highlight_related_count >= 2,
            "expected at least 2 highlight text entries, found {highlight_related_count} in texts: {texts:?}"
        );
    }

    #[test]
    fn caps_highlights_at_three_when_four_provided() {
        let stats = RunStats {
            highlights: vec![
                RunHighlight {
                    kind: HighlightKind::ClutchClear,
                    node_index: 0,
                    value: 2.0,
                },
                RunHighlight {
                    kind: HighlightKind::NoDamageNode,
                    node_index: 1,
                    value: 0.0,
                },
                RunHighlight {
                    kind: HighlightKind::FastClear,
                    node_index: 2,
                    value: 0.3,
                },
                RunHighlight {
                    kind: HighlightKind::PerfectStreak,
                    node_index: 3,
                    value: 7.0,
                },
            ],
            ..Default::default()
        };
        let mut app = test_app_with_stats(RunOutcome::Won, stats);
        app.update();

        let texts = collect_texts(&mut app);
        // The first 3 highlights should be shown
        let first_three_shown = texts
            .iter()
            .filter(|t| {
                t.contains("Clutch")
                    || t.contains("No Damage")
                    || t.contains("NoDamage")
                    || t.contains("Fast")
            })
            .count();
        assert!(
            first_three_shown >= 3,
            "expected first 3 highlights to be shown, found {first_three_shown} in texts: {texts:?}"
        );
        // The 4th highlight (PerfectStreak) should NOT appear
        let fourth_shown = texts
            .iter()
            .any(|t| t.contains("Perfect") || t.contains("Streak"));
        assert!(
            !fourth_shown,
            "expected 4th highlight (PerfectStreak) to be omitted, but found it in texts: {texts:?}"
        );
    }

    // ---- Behavior 5: Seed display ----

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

    // ---- Behavior 6: Chip build list ----

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

    // ---- Behavior 7: Graceful degradation without RunStats ----

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
}
