use bevy::prelude::*;

use super::{super::spawn_run_end_screen, helpers::*};
use crate::run::resources::{RunOutcome, RunState, RunStats};

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
