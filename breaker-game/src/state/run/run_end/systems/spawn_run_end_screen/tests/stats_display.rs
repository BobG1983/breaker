use super::helpers::*;
use crate::{prelude::*, state::run::resources::NodeResult};

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
    let mut app = test_app_with_stats(NodeResult::Won, stats);
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
    let mut app = test_app_with_stats(NodeResult::Won, stats);
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
    let mut app = test_app_with_stats(NodeResult::Won, stats);
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
    let mut app = test_app_with_stats(NodeResult::Won, stats);
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
    let mut app = test_app_with_stats(NodeResult::Won, stats);
    app.update();

    let texts = collect_texts(&mut app);
    assert!(
        texts.iter().any(|t| t.contains('3')),
        "expected bolts_lost '3' in texts: {texts:?}"
    );
}

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

    let mut app = test_app_with_stats(NodeResult::Won, stats);
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

    let mut app = test_app_with_stats(NodeResult::Won, stats);
    app.update();

    let texts = collect_texts(&mut app);
    assert!(
        texts.iter().any(|t| t.contains('0')),
        "expected flux '0' in texts: {texts:?}"
    );
}

#[test]
fn displays_highlight_entries() {
    let stats = RunStats {
        highlights: vec![
            RunHighlight {
                kind:       HighlightKind::ClutchClear,
                node_index: 3,
                value:      1.5,
                detail:     None,
            },
            RunHighlight {
                kind:       HighlightKind::NoDamageNode,
                node_index: 1,
                value:      0.0,
                detail:     None,
            },
        ],
        ..Default::default()
    };
    let mut app = test_app_with_stats(NodeResult::Won, stats);
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
                kind:       HighlightKind::ClutchClear,
                node_index: 0,
                value:      2.0,
                detail:     None,
            },
            RunHighlight {
                kind:       HighlightKind::NoDamageNode,
                node_index: 1,
                value:      0.0,
                detail:     None,
            },
            RunHighlight {
                kind:       HighlightKind::FastClear,
                node_index: 2,
                value:      0.3,
                detail:     None,
            },
            RunHighlight {
                kind:       HighlightKind::PerfectStreak,
                node_index: 3,
                value:      7.0,
                detail:     None,
            },
        ],
        ..Default::default()
    };
    let mut app = test_app_with_stats(NodeResult::Won, stats);
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
