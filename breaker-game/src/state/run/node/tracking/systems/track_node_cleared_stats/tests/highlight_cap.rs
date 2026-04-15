//! Tests for highlight storage beyond the old cap: verifying that highlights
//! are accumulated without limit during detection (selection at run-end).

use bevy::prelude::*;

use super::helpers::*;
use crate::{
    prelude::*,
    state::run::resources::{HighlightTracker, NodeOutcome},
};

// --- Behavior 7: highlight cap reads from config ---

#[test]
fn highlights_stored_beyond_four_existing() {
    let mut app = test_app();
    // Cap removed -- highlights always stored, selection at run-end
    // Pre-fill 4 highlights
    let mut stats = app.world_mut().resource_mut::<RunStats>();
    stats.highlights = vec![
        RunHighlight {
            kind:       HighlightKind::FastClear,
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
            kind:       HighlightKind::PerfectStreak,
            node_index: 2,
            value:      5.0,
            detail:     None,
        },
        RunHighlight {
            kind:       HighlightKind::ClutchClear,
            node_index: 3,
            value:      1.0,
            detail:     None,
        },
    ];

    // Set up conditions that would produce a NoDamageNode highlight
    app.world_mut()
        .resource_mut::<HighlightTracker>()
        .node_bolts_lost = 0;
    app.insert_resource(NodeTimer {
        remaining: 15.0,
        total:     30.0,
    });
    app.insert_resource(TestNodeCleared(true));
    tick(&mut app);

    let stats = app.world().resource::<RunStats>();
    assert!(
        stats.highlights.len() > 4,
        "highlights should be stored beyond 4 existing -- no cap during detection"
    );
}

// --- Behavior 8: sixth highlight stored beyond old cap ---

#[test]
fn sixth_highlight_stored_beyond_old_cap() {
    let mut app = test_app();
    // Pre-fill 5 highlights
    let mut stats = app.world_mut().resource_mut::<RunStats>();
    stats.highlights = vec![
        RunHighlight {
            kind:       HighlightKind::FastClear,
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
            kind:       HighlightKind::PerfectStreak,
            node_index: 2,
            value:      5.0,
            detail:     None,
        },
        RunHighlight {
            kind:       HighlightKind::ClutchClear,
            node_index: 3,
            value:      1.0,
            detail:     None,
        },
        RunHighlight {
            kind:       HighlightKind::FastClear,
            node_index: 4,
            value:      0.0,
            detail:     None,
        },
    ];

    // Set up conditions that would produce a ClutchClear
    app.insert_resource(NodeTimer {
        remaining: 1.0,
        total:     30.0,
    });
    app.insert_resource(TestNodeCleared(true));
    tick(&mut app);

    let stats = app.world().resource::<RunStats>();
    assert!(
        stats.highlights.len() > 5,
        "highlights should NOT be capped at 5 -- selection happens at run-end, not during detection. Got {}",
        stats.highlights.len()
    );
}

// --- Behavior: same kind stored across nodes even beyond old cap ---

#[test]
fn same_kind_stored_across_nodes_beyond_old_cap() {
    let mut app = test_app();
    // Pre-fill 5 highlights including a ClutchClear -- previously at cap
    let mut stats = app.world_mut().resource_mut::<RunStats>();
    stats.highlights = vec![
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
            kind:       HighlightKind::PerfectStreak,
            node_index: 2,
            value:      5.0,
            detail:     None,
        },
        RunHighlight {
            kind:       HighlightKind::FastClear,
            node_index: 3,
            value:      0.0,
            detail:     None,
        },
        RunHighlight {
            kind:       HighlightKind::NoDamageNode,
            node_index: 4,
            value:      0.0,
            detail:     None,
        },
    ];

    // Set up conditions for another ClutchClear on a later node
    app.world_mut().resource_mut::<NodeOutcome>().node_index = 5;
    app.insert_resource(NodeTimer {
        remaining: 1.0,
        total:     30.0,
    });
    app.insert_resource(TestNodeCleared(true));
    tick(&mut app);

    let stats = app.world().resource::<RunStats>();
    let clutch_count = stats
        .highlights
        .iter()
        .filter(|h| h.kind == HighlightKind::ClutchClear)
        .count();
    assert!(
        clutch_count >= 2,
        "same kind should be stored multiple times across nodes even beyond old cap of 5, got {clutch_count}"
    );
}
