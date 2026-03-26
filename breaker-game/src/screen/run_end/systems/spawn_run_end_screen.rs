//! System to spawn the run-end screen UI.

use bevy::{ecs::hierarchy::ChildSpawnerCommands, prelude::*};
use tracing::info;

use crate::{
    run::{
        definition::HighlightConfig, resources::*, systems::select_highlights::select_highlights,
    },
    screen::run_end::RunEndScreen,
};

const WON_SUBTITLES: [&str; 5] = [
    "The bolt obeys. For now.",
    "Every wall crumbles eventually.",
    "Built different. Broke everything.",
    "The signal holds. Barely.",
    "Clean sweep. Next time won't be.",
];
const TIMER_EXPIRED_SUBTITLES: [&str; 5] = [
    "The clock doesn't wait.",
    "Almost had it.",
    "Time ran out. The build didn't.",
    "So close. So far.",
    "One more second would've changed everything.",
];
const LIVES_DEPLETED_SUBTITLES: [&str; 5] = [
    "Signal lost. Rerouting.",
    "The bolt slipped away.",
    "Every loss teaches something.",
    "Down but not deleted.",
    "The grid remembers.",
];

/// Label color for stat rows and section headers.
const LABEL_COLOR: Color = Color::srgba(0.7, 0.7, 0.7, 1.0);

/// Spawns the run-end UI showing the run outcome and optional stats summary.
pub(crate) fn spawn_run_end_screen(
    mut commands: Commands,
    run_state: Res<RunState>,
    stats: Option<Res<RunStats>>,
    config: Option<Res<HighlightConfig>>,
) {
    info!("run ended");
    let seed = stats.as_ref().map_or(0_u64, |s| s.seed);
    let idx = usize::try_from(seed % 5).unwrap_or(0);
    let (title, subtitle) = match run_state.outcome {
        RunOutcome::Won => ("RUN COMPLETE", WON_SUBTITLES[idx]),
        RunOutcome::TimerExpired => ("TIME'S UP", TIMER_EXPIRED_SUBTITLES[idx]),
        RunOutcome::LivesDepleted => ("SIGNAL LOST", LIVES_DEPLETED_SUBTITLES[idx]),
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
                let selected: Vec<&RunHighlight> = if let Some(c) = config.as_ref() {
                    let cap = c.highlight_cap as usize;
                    let indices = select_highlights(&stats.highlights, c, cap);
                    indices.iter().map(|&i| &stats.highlights[i]).collect()
                } else {
                    stats.highlights.iter().take(3).collect()
                };
                spawn_stats_section(parent, stats);
                spawn_flux_section(parent, stats);
                spawn_highlights_section(parent, &selected);
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

/// Spawns text nodes for the selected run highlights.
fn spawn_highlights_section(parent: &mut ChildSpawnerCommands<'_>, highlights: &[&RunHighlight]) {
    for highlight in highlights {
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
            HighlightKind::MostPowerfulEvolution => {
                if let Some(name) = &highlight.detail {
                    format!("Most Powerful Evolution: {name}")
                } else {
                    format!("Most Powerful Evolution - Node {}", highlight.node_index)
                }
            }
            HighlightKind::CloseSave => {
                format!("Close Save - Node {}", highlight.node_index)
            }
            HighlightKind::SpeedDemon => {
                format!(
                    "Speed Demon - Node {} ({:.1}s)",
                    highlight.node_index, highlight.value
                )
            }
            HighlightKind::Untouchable => {
                format!("Untouchable x{:.0}", highlight.value)
            }
            HighlightKind::ComboKing => {
                format!(
                    "Combo King x{:.0} - Node {}",
                    highlight.value, highlight.node_index
                )
            }
            HighlightKind::PinballWizard => {
                format!(
                    "Pinball Wizard x{:.0} - Node {}",
                    highlight.value, highlight.node_index
                )
            }
            HighlightKind::Comeback => {
                format!("Comeback - Node {}", highlight.node_index)
            }
            HighlightKind::PerfectNode => {
                format!("Perfect Node - Node {}", highlight.node_index)
            }
            HighlightKind::NailBiter => {
                format!("Nail Biter - Node {}", highlight.node_index)
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
                    detail: None,
                },
                RunHighlight {
                    kind: HighlightKind::NoDamageNode,
                    node_index: 1,
                    value: 0.0,
                    detail: None,
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
                    detail: None,
                },
                RunHighlight {
                    kind: HighlightKind::NoDamageNode,
                    node_index: 1,
                    value: 0.0,
                    detail: None,
                },
                RunHighlight {
                    kind: HighlightKind::FastClear,
                    node_index: 2,
                    value: 0.3,
                    detail: None,
                },
                RunHighlight {
                    kind: HighlightKind::PerfectStreak,
                    node_index: 3,
                    value: 7.0,
                    detail: None,
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

    // ---- Death copy variants (seed-based subtitle selection) ----

    const WON_SUBS: [&str; 5] = [
        "The bolt obeys. For now.",
        "Every wall crumbles eventually.",
        "Built different. Broke everything.",
        "The signal holds. Barely.",
        "Clean sweep. Next time won't be.",
    ];
    const TIMER_EXPIRED_SUBS: [&str; 5] = [
        "The clock doesn't wait.",
        "Almost had it.",
        "Time ran out. The build didn't.",
        "So close. So far.",
        "One more second would've changed everything.",
    ];
    const LIVES_DEPLETED_SUBS: [&str; 5] = [
        "Signal lost. Rerouting.",
        "The bolt slipped away.",
        "Every loss teaches something.",
        "Down but not deleted.",
        "The grid remembers.",
    ];

    #[test]
    fn won_subtitle_is_from_known_variants() {
        let stats = RunStats {
            seed: 42,
            ..Default::default()
        };
        let mut app = test_app_with_stats(RunOutcome::Won, stats);
        app.update();

        let texts = collect_texts(&mut app);
        let subtitle_found = texts.iter().any(|t| WON_SUBS.contains(&t.as_str()));
        assert!(
            subtitle_found,
            "expected subtitle to be one of the known Won variants, got texts: {texts:?}"
        );
    }

    #[test]
    fn won_subtitle_is_deterministic_with_same_seed() {
        let make_app = || {
            let stats = RunStats {
                seed: 42,
                ..Default::default()
            };
            let mut app = test_app_with_stats(RunOutcome::Won, stats);
            app.update();
            collect_texts(&mut app)
        };

        let texts_a = make_app();
        let texts_b = make_app();

        // Find subtitle text (not the title, not "Press Enter")
        let find_subtitle = |texts: &[String]| -> Option<String> {
            texts
                .iter()
                .find(|t| WON_SUBS.contains(&t.as_str()))
                .cloned()
        };

        let sub_a = find_subtitle(&texts_a).expect("first app should have a known Won subtitle");
        let sub_b = find_subtitle(&texts_b).expect("second app should have a known Won subtitle");
        assert_eq!(
            sub_a, sub_b,
            "same seed=42 should produce the same subtitle across runs"
        );
    }

    #[test]
    fn timer_expired_subtitle_is_from_known_variants() {
        let stats = RunStats {
            seed: 99,
            ..Default::default()
        };
        let mut app = test_app_with_stats(RunOutcome::TimerExpired, stats);
        app.update();

        let texts = collect_texts(&mut app);
        let subtitle_found = TIMER_EXPIRED_SUBS
            .iter()
            .any(|sub| texts.iter().any(|t| t.as_str() == *sub));
        assert!(
            subtitle_found,
            "expected subtitle to be one of the known TimerExpired variants, got texts: {texts:?}"
        );
    }

    #[test]
    fn lives_depleted_subtitle_is_from_known_variants() {
        let stats = RunStats {
            seed: 77,
            ..Default::default()
        };
        let mut app = test_app_with_stats(RunOutcome::LivesDepleted, stats);
        app.update();

        let texts = collect_texts(&mut app);
        let subtitle_found = LIVES_DEPLETED_SUBS
            .iter()
            .any(|sub| texts.iter().any(|t| t.as_str() == *sub));
        assert!(
            subtitle_found,
            "expected subtitle to be one of the known LivesDepleted variants, got texts: {texts:?}"
        );
    }

    #[test]
    fn in_progress_outcome_shows_run_ended() {
        let mut app = test_app(RunOutcome::InProgress);
        app.update();

        let texts = collect_texts(&mut app);
        assert!(
            texts.iter().any(|t| t.as_str() == "RUN ENDED"),
            "expected title 'RUN ENDED' for InProgress outcome, got texts: {texts:?}"
        );
    }

    #[test]
    fn subtitle_falls_back_to_first_variant_without_stats() {
        // No RunStats inserted — subtitle should fall back to first Won variant.
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(RunState {
                node_index: 0,
                outcome: RunOutcome::Won,
                ..default()
            })
            .add_systems(Update, spawn_run_end_screen);
        app.update();

        let texts = collect_texts(&mut app);
        assert!(
            texts
                .iter()
                .any(|t| t.as_str() == "The bolt obeys. For now."),
            "expected fallback subtitle 'The bolt obeys. For now.' without RunStats, got texts: {texts:?}"
        );
    }

    #[test]
    fn different_seeds_produce_different_subtitles() {
        let seeds = [0u64, 1, 2, 3, 4];
        let mut subtitles = Vec::new();

        for seed in seeds {
            let stats = RunStats {
                seed,
                ..Default::default()
            };
            let mut app = test_app_with_stats(RunOutcome::Won, stats);
            app.update();

            let texts = collect_texts(&mut app);
            let subtitle = texts
                .iter()
                .find(|t| WON_SUBS.contains(&t.as_str()))
                .cloned()
                .unwrap_or_default();
            subtitles.push(subtitle);
        }

        subtitles.sort();
        subtitles.dedup();
        assert!(
            subtitles.len() >= 2,
            "expected at least 2 distinct subtitles across seeds [0..4], got: {subtitles:?}"
        );
    }

    // ---- Dynamic highlight cap ----

    /// Known highlight text prefixes used in `spawn_highlights_section`.
    fn is_highlight_text(text: &str) -> bool {
        text.starts_with("Clutch Clear")
            || text.starts_with("No Damage")
            || text.starts_with("Fast Clear")
            || text.starts_with("Perfect Streak")
            || text.starts_with("Mass Destruction")
            || text.starts_with("First Evolution")
            || text.starts_with("Most Powerful Evolution")
            || text.starts_with("Close Save")
            || text.starts_with("Speed Demon")
            || text.starts_with("Untouchable")
            || text.starts_with("Combo King")
            || text.starts_with("Pinball Wizard")
            || text.starts_with("Comeback")
            || text.starts_with("Perfect Node")
            || text.starts_with("Nail Biter")
    }

    fn make_highlights(count: usize) -> Vec<RunHighlight> {
        let kinds = [
            HighlightKind::ClutchClear,
            HighlightKind::NoDamageNode,
            HighlightKind::FastClear,
            HighlightKind::PerfectStreak,
            HighlightKind::MassDestruction,
            HighlightKind::FirstEvolution,
        ];
        (0..count)
            .map(|i| RunHighlight {
                kind: kinds[i % kinds.len()].clone(),
                node_index: u32::try_from(i).unwrap_or(u32::MAX),
                value: 1.0,
                detail: None,
            })
            .collect()
    }

    #[test]
    fn highlight_cap_reads_from_config() {
        let stats = RunStats {
            highlights: make_highlights(6),
            ..Default::default()
        };
        let mut app = test_app_with_stats(RunOutcome::Won, stats);
        app.insert_resource(HighlightConfig {
            highlight_cap: 4,
            ..Default::default()
        });
        app.update();

        let texts = collect_texts(&mut app);
        let highlight_count = texts.iter().filter(|t| is_highlight_text(t)).count();
        assert_eq!(
            highlight_count, 4,
            "expected 4 highlights when HighlightConfig.highlight_cap = 4, got {highlight_count} in texts: {texts:?}"
        );
    }

    #[test]
    fn highlight_cap_falls_back_to_three_without_config() {
        let stats = RunStats {
            highlights: make_highlights(6),
            ..Default::default()
        };
        let mut app = test_app_with_stats(RunOutcome::Won, stats);
        // Deliberately do NOT insert HighlightConfig.
        app.update();

        let texts = collect_texts(&mut app);
        let highlight_count = texts.iter().filter(|t| is_highlight_text(t)).count();
        assert_eq!(
            highlight_count, 3,
            "expected 3 highlights as fallback without HighlightConfig, got {highlight_count} in texts: {texts:?}"
        );
    }

    #[test]
    fn highlight_cap_shows_fewer_when_fewer_exist() {
        let stats = RunStats {
            highlights: make_highlights(2),
            ..Default::default()
        };
        let mut app = test_app_with_stats(RunOutcome::Won, stats);
        app.insert_resource(HighlightConfig {
            highlight_cap: 10,
            ..Default::default()
        });
        app.update();

        let texts = collect_texts(&mut app);
        let highlight_count = texts.iter().filter(|t| is_highlight_text(t)).count();
        assert_eq!(
            highlight_count, 2,
            "expected 2 highlights when only 2 exist (cap=10), got {highlight_count} in texts: {texts:?}"
        );
    }

    // ---- Diversity-penalized highlight selection ----

    /// Collects only highlight texts from all spawned Text entities, in spawn order.
    fn collect_highlight_texts(app: &mut App) -> Vec<String> {
        collect_texts(app)
            .into_iter()
            .filter(|t| is_highlight_text(t))
            .collect()
    }

    #[test]
    fn highlights_displayed_in_diversity_penalized_order_with_config() {
        // 4 highlights in FIFO order (input order):
        // idx 0: PerfectStreak (Execution, score ~0.333)
        // idx 1: ComboKing (Execution, score ~0.333)
        // idx 2: NoDamageNode (Endurance, binary score 1.0)
        // idx 3: ClutchClear (Clutch, score ~0.222)
        let stats = RunStats {
            highlights: vec![
                RunHighlight {
                    kind: HighlightKind::PerfectStreak,
                    node_index: 0,
                    value: 10.0,
                    detail: None,
                },
                RunHighlight {
                    kind: HighlightKind::ComboKing,
                    node_index: 1,
                    value: 16.0,
                    detail: None,
                },
                RunHighlight {
                    kind: HighlightKind::NoDamageNode,
                    node_index: 2,
                    value: 0.0,
                    detail: None,
                },
                RunHighlight {
                    kind: HighlightKind::ClutchClear,
                    node_index: 3,
                    value: 1.0,
                    detail: None,
                },
            ],
            ..Default::default()
        };
        let mut app = test_app_with_stats(RunOutcome::Won, stats);
        app.insert_resource(HighlightConfig::default()); // diversity_penalty=0.5, highlight_cap=5
        app.update();

        let highlights = collect_highlight_texts(&mut app);
        assert_eq!(
            highlights.len(),
            4,
            "expected 4 highlights spawned, got {}: {highlights:?}",
            highlights.len()
        );

        // Diversity-penalized selection order:
        // Round 1: NoDamageNode (1.0, Endurance, no penalty)
        // Round 2: PerfectStreak (0.333, Execution, 0 prior Execution picks)
        // Round 3: ClutchClear (0.222, Clutch, 0 prior Clutch picks > ComboKing penalized 0.333*0.5=0.167)
        // Round 4: ComboKing (last remaining)
        assert!(
            highlights[0].contains("No Damage"),
            "first highlight should be 'No Damage - Node 2', got: {:?}",
            highlights[0]
        );
        assert!(
            highlights[1].contains("Perfect Streak"),
            "second highlight should be 'Perfect Streak', got: {:?}",
            highlights[1]
        );
        assert!(
            highlights[2].contains("Clutch Clear"),
            "third highlight should be 'Clutch Clear - Node 3', got: {:?}",
            highlights[2]
        );
        assert!(
            highlights[3].contains("Combo King"),
            "fourth highlight should be 'Combo King', got: {:?}",
            highlights[3]
        );
    }

    #[test]
    fn highlights_fifo_fallback_without_config() {
        // 4 highlights, NO HighlightConfig → FIFO fallback with default cap=3
        let stats = RunStats {
            highlights: vec![
                RunHighlight {
                    kind: HighlightKind::ClutchClear,
                    node_index: 0,
                    value: 2.0,
                    detail: None,
                },
                RunHighlight {
                    kind: HighlightKind::NoDamageNode,
                    node_index: 1,
                    value: 0.0,
                    detail: None,
                },
                RunHighlight {
                    kind: HighlightKind::FastClear,
                    node_index: 2,
                    value: 0.0,
                    detail: None,
                },
                RunHighlight {
                    kind: HighlightKind::PerfectStreak,
                    node_index: 3,
                    value: 7.0,
                    detail: None,
                },
            ],
            ..Default::default()
        };
        let mut app = test_app_with_stats(RunOutcome::Won, stats);
        // Deliberately do NOT insert HighlightConfig.
        app.update();

        let highlights = collect_highlight_texts(&mut app);
        assert_eq!(
            highlights.len(),
            3,
            "expected exactly 3 highlights (default cap), got {}: {highlights:?}",
            highlights.len()
        );
        // FIFO order: first 3 in input order
        assert!(
            highlights[0].contains("Clutch Clear"),
            "first FIFO highlight should be 'Clutch Clear', got: {:?}",
            highlights[0]
        );
        assert!(
            highlights[1].contains("No Damage"),
            "second FIFO highlight should be 'No Damage', got: {:?}",
            highlights[1]
        );
        assert!(
            highlights[2].contains("Fast Clear"),
            "third FIFO highlight should be 'Fast Clear', got: {:?}",
            highlights[2]
        );
        // PerfectStreak (4th) should NOT be shown
        let has_perfect_streak = highlights.iter().any(|t| t.contains("Perfect Streak"));
        assert!(
            !has_perfect_streak,
            "PerfectStreak (4th highlight) should be omitted with cap=3, but found it in: {highlights:?}"
        );
    }

    #[test]
    fn respects_highlight_cap_from_config_with_diversity_selection() {
        // 6 highlights, cap=2 → exactly 2 shown
        let stats = RunStats {
            highlights: vec![
                RunHighlight {
                    kind: HighlightKind::ClutchClear,
                    node_index: 0,
                    value: 0.0,
                    detail: None,
                },
                RunHighlight {
                    kind: HighlightKind::NoDamageNode,
                    node_index: 1,
                    value: 0.0,
                    detail: None,
                },
                RunHighlight {
                    kind: HighlightKind::FastClear,
                    node_index: 2,
                    value: 0.0,
                    detail: None,
                },
                RunHighlight {
                    kind: HighlightKind::PerfectStreak,
                    node_index: 3,
                    value: 0.0,
                    detail: None,
                },
                RunHighlight {
                    kind: HighlightKind::MassDestruction,
                    node_index: 4,
                    value: 0.0,
                    detail: None,
                },
                RunHighlight {
                    kind: HighlightKind::FirstEvolution,
                    node_index: 5,
                    value: 0.0,
                    detail: None,
                },
            ],
            ..Default::default()
        };
        let mut app = test_app_with_stats(RunOutcome::Won, stats);
        app.insert_resource(HighlightConfig {
            highlight_cap: 2,
            ..Default::default()
        });
        app.update();

        let highlights = collect_highlight_texts(&mut app);
        assert_eq!(
            highlights.len(),
            2,
            "expected exactly 2 highlights with highlight_cap=2, got {}: {highlights:?}",
            highlights.len()
        );
    }

    #[test]
    fn empty_highlights_with_config_produces_no_highlight_text() {
        let stats = RunStats {
            highlights: vec![],
            ..Default::default()
        };
        let mut app = test_app_with_stats(RunOutcome::Won, stats);
        app.insert_resource(HighlightConfig::default());
        app.update();

        let highlights = collect_highlight_texts(&mut app);
        assert!(
            highlights.is_empty(),
            "expected no highlight texts with empty highlights vec, got: {highlights:?}"
        );
    }

    #[test]
    fn highlight_cap_zero_produces_no_highlights() {
        let stats = RunStats {
            highlights: vec![
                RunHighlight {
                    kind: HighlightKind::ClutchClear,
                    node_index: 0,
                    value: 1.0,
                    detail: None,
                },
                RunHighlight {
                    kind: HighlightKind::NoDamageNode,
                    node_index: 1,
                    value: 0.0,
                    detail: None,
                },
                RunHighlight {
                    kind: HighlightKind::PerfectStreak,
                    node_index: 2,
                    value: 5.0,
                    detail: None,
                },
            ],
            ..Default::default()
        };
        let mut app = test_app_with_stats(RunOutcome::Won, stats);
        app.insert_resource(HighlightConfig {
            highlight_cap: 0,
            ..Default::default()
        });
        app.update();

        let highlights = collect_highlight_texts(&mut app);
        assert!(
            highlights.is_empty(),
            "expected no highlight texts with highlight_cap=0, got: {highlights:?}"
        );
    }

    #[test]
    fn single_highlight_with_config_uses_selection() {
        let stats = RunStats {
            highlights: vec![RunHighlight {
                kind: HighlightKind::NoDamageNode,
                node_index: 0,
                value: 0.0,
                detail: None,
            }],
            ..Default::default()
        };
        let mut app = test_app_with_stats(RunOutcome::Won, stats);
        app.insert_resource(HighlightConfig::default());
        app.update();

        let highlights = collect_highlight_texts(&mut app);
        assert_eq!(
            highlights.len(),
            1,
            "expected exactly 1 highlight, got {}: {highlights:?}",
            highlights.len()
        );
        assert!(
            highlights[0].contains("No Damage"),
            "single highlight should be 'No Damage - Node 0', got: {:?}",
            highlights[0]
        );
    }

    #[test]
    fn most_powerful_evolution_shows_chip_name_from_run_stats() {
        let stats = RunStats {
            highlights: vec![RunHighlight {
                kind: HighlightKind::MostPowerfulEvolution,
                node_index: 0,
                value: 400.0,
                detail: Some("Chain Lightning".to_owned()),
            }],
            ..Default::default()
        };
        let mut app = test_app_with_stats(RunOutcome::Won, stats);
        app.insert_resource(HighlightConfig::default());
        app.update();

        let highlights = collect_highlight_texts(&mut app);
        assert_eq!(
            highlights.len(),
            1,
            "expected exactly 1 highlight, got {}: {highlights:?}",
            highlights.len()
        );
        assert!(
            highlights[0].contains("Chain Lightning"),
            "MostPowerfulEvolution highlight should contain the chip name 'Chain Lightning', got: {:?}",
            highlights[0]
        );
    }
}
