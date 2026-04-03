//! System to spawn the run-end screen UI.

use bevy::{ecs::hierarchy::ChildSpawnerCommands, prelude::*};
use tracing::info;

use crate::state::run::{
    definition::HighlightConfig, resources::*, run_end::RunEndScreen,
    systems::select_highlights::select_highlights,
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
                let selected: Vec<&RunHighlight> = config.as_ref().map_or_else(
                    || stats.highlights.iter().take(3).collect(),
                    |c| {
                        let cap = c.highlight_cap as usize;
                        let indices = select_highlights(&stats.highlights, c, cap);
                        indices.iter().map(|&i| &stats.highlights[i]).collect()
                    },
                );
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
            HighlightKind::MostPowerfulEvolution => highlight.detail.as_ref().map_or_else(
                || format!("Most Powerful Evolution - Node {}", highlight.node_index),
                |name| format!("Most Powerful Evolution: {name}"),
            ),
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
