use super::{super::helpers::*, helpers::*};
use crate::state::run::{
    definition::HighlightConfig,
    resources::{HighlightKind, NodeResult, RunHighlight, RunStats},
};

#[test]
fn empty_highlights_with_config_produces_no_highlight_text() {
    let stats = RunStats {
        highlights: vec![],
        ..Default::default()
    };
    let mut app = test_app_with_stats(NodeResult::Won, stats);
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
                kind:       HighlightKind::ClutchClear,
                node_index: 0,
                value:      1.0,
                detail:     None,
            },
            RunHighlight {
                kind:       HighlightKind::NoDamageNode,
                node_index: 1,
                value:      0.0,
                detail:     None,
            },
            RunHighlight {
                kind:       HighlightKind::PerfectStreak,
                node_index: 2,
                value:      5.0,
                detail:     None,
            },
        ],
        ..Default::default()
    };
    let mut app = test_app_with_stats(NodeResult::Won, stats);
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
            kind:       HighlightKind::NoDamageNode,
            node_index: 0,
            value:      0.0,
            detail:     None,
        }],
        ..Default::default()
    };
    let mut app = test_app_with_stats(NodeResult::Won, stats);
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
            kind:       HighlightKind::MostPowerfulEvolution,
            node_index: 0,
            value:      400.0,
            detail:     Some("Chain Lightning".to_owned()),
        }],
        ..Default::default()
    };
    let mut app = test_app_with_stats(NodeResult::Won, stats);
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
