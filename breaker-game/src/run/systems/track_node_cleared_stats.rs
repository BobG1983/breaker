//! System to record node-level stats and detect highlights on node clear.

use bevy::prelude::*;

use crate::run::{
    node::{messages::NodeCleared, resources::NodeTimer},
    resources::{
        CLUTCH_CLEAR_THRESHOLD, FAST_CLEAR_FRACTION, HighlightKind, HighlightTracker,
        PERFECT_STREAK_THRESHOLD, RunHighlight, RunState, RunStats,
    },
};

/// Reads [`NodeCleared`] messages and updates [`RunStats`] with node
/// completion data and highlight detection.
pub(crate) fn track_node_cleared_stats(
    mut reader: MessageReader<NodeCleared>,
    mut stats: ResMut<RunStats>,
    mut tracker: ResMut<HighlightTracker>,
    run_state: Res<RunState>,
    node_timer: Res<NodeTimer>,
) {
    for _msg in reader.read() {
        stats.nodes_cleared += 1;

        if stats.highlights.len() >= 3 {
            continue;
        }

        let node_index = run_state.node_index;

        // ClutchClear: cleared with less than threshold seconds remaining
        if node_timer.remaining < CLUTCH_CLEAR_THRESHOLD && stats.highlights.len() < 3 {
            stats.highlights.push(RunHighlight {
                kind: HighlightKind::ClutchClear,
                node_index,
                value: node_timer.remaining,
            });
        }

        // NoDamageNode: no bolts lost this node
        if tracker.node_bolts_lost == 0 && stats.highlights.len() < 3 {
            stats.highlights.push(RunHighlight {
                kind: HighlightKind::NoDamageNode,
                node_index,
                value: 0.0,
            });
        }

        // FastClear: elapsed time < fraction of total
        let elapsed = node_timer.total - node_timer.remaining;
        if elapsed < node_timer.total * FAST_CLEAR_FRACTION && stats.highlights.len() < 3 {
            stats.highlights.push(RunHighlight {
                kind: HighlightKind::FastClear,
                node_index,
                value: 0.0,
            });
        }

        // PerfectStreak: flush active streak first, then check
        tracker.best_perfect_streak = tracker
            .best_perfect_streak
            .max(tracker.consecutive_perfect_bumps);
        if tracker.best_perfect_streak >= PERFECT_STREAK_THRESHOLD && stats.highlights.len() < 3 {
            stats.highlights.push(RunHighlight {
                kind: HighlightKind::PerfectStreak,
                node_index,
                value: {
                    let streak = tracker.best_perfect_streak.min(u32::from(u16::MAX));
                    f32::from(u16::try_from(streak).unwrap_or(u16::MAX))
                },
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::run::resources::{
        CLUTCH_CLEAR_THRESHOLD, FAST_CLEAR_FRACTION, HighlightKind, PERFECT_STREAK_THRESHOLD,
        RunHighlight,
    };

    #[derive(Resource)]
    struct TestNodeCleared(bool);

    fn enqueue_node_cleared(msg_res: Res<TestNodeCleared>, mut writer: MessageWriter<NodeCleared>) {
        if msg_res.0 {
            writer.write(NodeCleared);
        }
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .add_message::<NodeCleared>()
            .init_resource::<RunStats>()
            .init_resource::<HighlightTracker>()
            .init_resource::<RunState>()
            .insert_resource(NodeTimer {
                remaining: 15.0,
                total: 30.0,
            })
            .add_systems(
                FixedUpdate,
                (enqueue_node_cleared, track_node_cleared_stats).chain(),
            );
        app
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    #[test]
    fn increments_nodes_cleared() {
        let mut app = test_app();
        app.insert_resource(TestNodeCleared(true));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        assert_eq!(
            stats.nodes_cleared, 1,
            "NodeCleared should increment nodes_cleared"
        );
    }

    #[test]
    fn detects_clutch_clear_when_timer_below_threshold() {
        let mut app = test_app();
        app.insert_resource(NodeTimer {
            remaining: 2.5,
            total: 30.0,
        });
        app.world_mut().resource_mut::<RunState>().node_index = 3;
        app.insert_resource(TestNodeCleared(true));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        let clutch = stats
            .highlights
            .iter()
            .find(|h| h.kind == HighlightKind::ClutchClear);
        assert!(
            clutch.is_some(),
            "should detect ClutchClear with 2.5s remaining"
        );
        let clutch = clutch.unwrap();
        assert_eq!(clutch.node_index, 3);
        assert!(
            (clutch.value - 2.5).abs() < f32::EPSILON,
            "ClutchClear value should be 2.5, got {}",
            clutch.value
        );
    }

    #[test]
    fn no_clutch_clear_when_timer_above_threshold() {
        let mut app = test_app();
        app.insert_resource(NodeTimer {
            remaining: 5.0,
            total: 30.0,
        });
        app.insert_resource(TestNodeCleared(true));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        let clutch = stats
            .highlights
            .iter()
            .find(|h| h.kind == HighlightKind::ClutchClear);
        assert!(
            clutch.is_none(),
            "should NOT detect ClutchClear with 5.0s remaining (>= {CLUTCH_CLEAR_THRESHOLD})"
        );
    }

    #[test]
    fn detects_no_damage_node_when_no_bolts_lost() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<HighlightTracker>()
            .node_bolts_lost = 0;
        app.world_mut().resource_mut::<RunState>().node_index = 2;
        app.insert_resource(TestNodeCleared(true));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        let no_damage = stats
            .highlights
            .iter()
            .find(|h| h.kind == HighlightKind::NoDamageNode);
        assert!(
            no_damage.is_some(),
            "should detect NoDamageNode when node_bolts_lost == 0"
        );
        assert_eq!(no_damage.unwrap().node_index, 2);
    }

    #[test]
    fn no_damage_node_not_detected_when_bolts_lost() {
        let mut app = test_app();
        app.world_mut()
            .resource_mut::<HighlightTracker>()
            .node_bolts_lost = 1;
        app.insert_resource(TestNodeCleared(true));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        let no_damage = stats
            .highlights
            .iter()
            .find(|h| h.kind == HighlightKind::NoDamageNode);
        assert!(
            no_damage.is_none(),
            "should NOT detect NoDamageNode when bolts were lost"
        );
    }

    #[test]
    fn detects_fast_clear_when_elapsed_below_half_total() {
        let mut app = test_app();
        // remaining=20 means elapsed=10, total=30 -> 10/30 = 0.33 < 0.5
        app.insert_resource(NodeTimer {
            remaining: 20.0,
            total: 30.0,
        });
        app.world_mut().resource_mut::<RunState>().node_index = 1;
        app.insert_resource(TestNodeCleared(true));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        let fast = stats
            .highlights
            .iter()
            .find(|h| h.kind == HighlightKind::FastClear);
        assert!(
            fast.is_some(),
            "should detect FastClear when elapsed ({}) < {} of total ({})",
            10.0,
            FAST_CLEAR_FRACTION,
            30.0
        );
        assert_eq!(fast.unwrap().node_index, 1);
    }

    #[test]
    fn detects_perfect_streak_at_node_end() {
        let mut app = test_app();
        let mut tracker = app.world_mut().resource_mut::<HighlightTracker>();
        tracker.best_perfect_streak = 7;
        tracker.consecutive_perfect_bumps = 7;

        app.insert_resource(TestNodeCleared(true));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        let streak = stats
            .highlights
            .iter()
            .find(|h| h.kind == HighlightKind::PerfectStreak);
        assert!(
            streak.is_some(),
            "should detect PerfectStreak with streak of 7 (>= {PERFECT_STREAK_THRESHOLD})"
        );
        assert!(
            (streak.unwrap().value - 7.0).abs() < f32::EPSILON,
            "PerfectStreak value should be 7.0"
        );
    }

    #[test]
    fn highlight_cap_at_three() {
        let mut app = test_app();
        // Pre-fill 3 highlights
        let mut stats = app.world_mut().resource_mut::<RunStats>();
        stats.highlights = vec![
            RunHighlight {
                kind: HighlightKind::FastClear,
                node_index: 0,
                value: 0.0,
            },
            RunHighlight {
                kind: HighlightKind::NoDamageNode,
                node_index: 1,
                value: 0.0,
            },
            RunHighlight {
                kind: HighlightKind::PerfectStreak,
                node_index: 2,
                value: 5.0,
            },
        ];

        // Set up conditions that would produce a ClutchClear highlight
        app.insert_resource(NodeTimer {
            remaining: 1.0,
            total: 30.0,
        });
        app.insert_resource(TestNodeCleared(true));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        assert_eq!(
            stats.highlights.len(),
            3,
            "highlights should be capped at 3, 4th highlight should be dropped"
        );
    }
}
