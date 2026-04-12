use super::{super::helpers::*, helpers::*};
use crate::state::run::{
    definition::HighlightConfig,
    resources::{HighlightKind, NodeResult, RunHighlight, RunStats},
};

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
                kind:       HighlightKind::PerfectStreak,
                node_index: 0,
                value:      10.0,
                detail:     None,
            },
            RunHighlight {
                kind:       HighlightKind::ComboKing,
                node_index: 1,
                value:      16.0,
                detail:     None,
            },
            RunHighlight {
                kind:       HighlightKind::NoDamageNode,
                node_index: 2,
                value:      0.0,
                detail:     None,
            },
            RunHighlight {
                kind:       HighlightKind::ClutchClear,
                node_index: 3,
                value:      1.0,
                detail:     None,
            },
        ],
        ..Default::default()
    };
    let mut app = test_app_with_stats(NodeResult::Won, stats);
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
                value:      0.0,
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
                kind:       HighlightKind::ClutchClear,
                node_index: 0,
                value:      0.0,
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
                value:      0.0,
                detail:     None,
            },
            RunHighlight {
                kind:       HighlightKind::PerfectStreak,
                node_index: 3,
                value:      0.0,
                detail:     None,
            },
            RunHighlight {
                kind:       HighlightKind::MassDestruction,
                node_index: 4,
                value:      0.0,
                detail:     None,
            },
            RunHighlight {
                kind:       HighlightKind::FirstEvolution,
                node_index: 5,
                value:      0.0,
                detail:     None,
            },
        ],
        ..Default::default()
    };
    let mut app = test_app_with_stats(NodeResult::Won, stats);
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
