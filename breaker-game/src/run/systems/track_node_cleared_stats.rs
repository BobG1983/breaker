//! System to record node-level stats and detect highlights on node clear.

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::run::{
    definition::HighlightConfig,
    messages::HighlightTriggered,
    node::{messages::NodeCleared, resources::NodeTimer},
    resources::{HighlightKind, HighlightTracker, RunHighlight, RunState, RunStats},
};

/// Bundled resources for node-clear highlight detection.
#[derive(SystemParam)]
pub(crate) struct NodeClearContext<'w> {
    stats: ResMut<'w, RunStats>,
    tracker: ResMut<'w, HighlightTracker>,
    run_state: Res<'w, RunState>,
    node_timer: Res<'w, NodeTimer>,
    config: Res<'w, HighlightConfig>,
    time: Res<'w, Time<Fixed>>,
}

/// Reads [`NodeCleared`] messages and updates [`RunStats`] with node
/// completion data and highlight detection.
///
/// Uses [`HighlightConfig`] for all thresholds and the highlight cap.
pub(crate) fn track_node_cleared_stats(
    mut reader: MessageReader<NodeCleared>,
    mut ctx: NodeClearContext,
    mut highlight_writer: MessageWriter<HighlightTriggered>,
) {
    for _msg in reader.read() {
        ctx.stats.nodes_cleared += 1;

        let node_index = ctx.run_state.node_index;

        // Helper: record highlight and emit juice message
        let mut push_highlight = |kind: HighlightKind, value: f32| {
            highlight_writer.write(HighlightTriggered { kind: kind.clone() });
            ctx.stats.highlights.push(RunHighlight {
                kind,
                node_index,
                value,
            });
        };

        // ClutchClear: cleared with less than threshold seconds remaining
        if ctx.node_timer.remaining < ctx.config.clutch_clear_secs {
            push_highlight(HighlightKind::ClutchClear, ctx.node_timer.remaining);
        }

        // NoDamageNode: no bolts lost this node
        if ctx.tracker.node_bolts_lost == 0 {
            push_highlight(HighlightKind::NoDamageNode, 0.0);
        }

        // FastClear: elapsed time < fraction of total
        let elapsed = ctx.node_timer.total - ctx.node_timer.remaining;
        if elapsed < ctx.node_timer.total * ctx.config.fast_clear_fraction {
            push_highlight(HighlightKind::FastClear, 0.0);
        }

        // PerfectStreak: check current node's streak, then flush into cross-node best.
        // Only fire when the current node actually contributed a qualifying streak.
        let current_streak = ctx.tracker.consecutive_perfect_bumps;
        if current_streak >= ctx.config.perfect_streak_count {
            let streak = current_streak.min(u32::from(u16::MAX));
            push_highlight(
                HighlightKind::PerfectStreak,
                f32::from(u16::try_from(streak).unwrap_or(u16::MAX)),
            );
        }
        ctx.tracker.best_perfect_streak = ctx.tracker.best_perfect_streak.max(current_streak);

        // SpeedDemon: node elapsed time below threshold
        let node_elapsed = ctx.time.elapsed_secs() - ctx.tracker.node_start_time;
        if node_elapsed < ctx.config.speed_demon_secs {
            push_highlight(HighlightKind::SpeedDemon, node_elapsed);
        }
        ctx.tracker.fastest_node_clear_secs = ctx.tracker.fastest_node_clear_secs.min(node_elapsed);

        // Untouchable: consecutive no-damage nodes
        if ctx.tracker.node_bolts_lost == 0 {
            ctx.tracker.consecutive_no_damage_nodes += 1;
        } else {
            ctx.tracker.consecutive_no_damage_nodes = 0;
        }
        if ctx.tracker.consecutive_no_damage_nodes >= ctx.config.untouchable_nodes {
            push_highlight(
                HighlightKind::Untouchable,
                f32::from(
                    u16::try_from(ctx.tracker.consecutive_no_damage_nodes).unwrap_or(u16::MAX),
                ),
            );
        }

        // Comeback: cleared despite losing many bolts
        if ctx.tracker.node_bolts_lost >= ctx.config.comeback_bolts_lost {
            push_highlight(
                HighlightKind::Comeback,
                f32::from(u16::try_from(ctx.tracker.node_bolts_lost).unwrap_or(u16::MAX)),
            );
        }

        // PerfectNode: every bump was perfect grade
        if ctx.tracker.non_perfect_bumps_this_node == 0 && ctx.tracker.total_bumps_this_node > 0 {
            push_highlight(
                HighlightKind::PerfectNode,
                f32::from(u16::try_from(ctx.tracker.total_bumps_this_node).unwrap_or(u16::MAX)),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::run::resources::{HighlightKind, RunHighlight};

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
            .add_message::<HighlightTriggered>()
            .init_resource::<RunStats>()
            .init_resource::<HighlightTracker>()
            .init_resource::<RunState>()
            .insert_resource(HighlightConfig::default())
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

    // --- Existing behavior tests (updated to use HighlightConfig) ---

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
        // Default clutch_clear_secs is 3.0, so 5.0 remaining should not trigger
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
            "should NOT detect ClutchClear with 5.0s remaining (>= 3.0)"
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
        // remaining=20 means elapsed=10, total=30 -> 10/30 = 0.33 < 0.5 (default fast_clear_fraction)
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
            "should detect FastClear when elapsed (10) < 0.5 of total (30)"
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
            "should detect PerfectStreak with streak of 7 (>= 5)"
        );
        assert!(
            (streak.unwrap().value - 7.0).abs() < f32::EPSILON,
            "PerfectStreak value should be 7.0"
        );
    }

    // --- Behavior 4: clutch_clear_secs reads from config ---

    #[test]
    fn clutch_clear_uses_config_threshold() {
        let mut app = test_app();
        // Override config with clutch_clear_secs=5.0
        let config = HighlightConfig {
            clutch_clear_secs: 5.0,
            ..Default::default()
        };
        app.insert_resource(config);
        // remaining=4.5 < 5.0 -> ClutchClear
        app.insert_resource(NodeTimer {
            remaining: 4.5,
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
            clutch.is_some(),
            "should detect ClutchClear with config.clutch_clear_secs=5.0 and remaining=4.5"
        );
        assert!(
            (clutch.unwrap().value - 4.5).abs() < f32::EPSILON,
            "ClutchClear value should be 4.5"
        );
    }

    // --- Behavior 5: fast_clear_fraction reads from config ---

    #[test]
    fn fast_clear_uses_config_fraction() {
        let mut app = test_app();
        // Override config with fast_clear_fraction=0.3
        let config = HighlightConfig {
            fast_clear_fraction: 0.3,
            ..Default::default()
        };
        app.insert_resource(config);
        // total=30, remaining=22 -> elapsed=8, threshold=30*0.3=9.0, 8<9 -> FastClear
        app.insert_resource(NodeTimer {
            remaining: 22.0,
            total: 30.0,
        });
        app.insert_resource(TestNodeCleared(true));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        let fast = stats
            .highlights
            .iter()
            .find(|h| h.kind == HighlightKind::FastClear);
        assert!(
            fast.is_some(),
            "should detect FastClear with config.fast_clear_fraction=0.3, elapsed=8 < 9"
        );
    }

    // --- Behavior 6: perfect_streak_count reads from config ---

    #[test]
    fn perfect_streak_uses_config_count() {
        let mut app = test_app();
        // Override config with perfect_streak_count=3
        let config = HighlightConfig {
            perfect_streak_count: 3,
            ..Default::default()
        };
        app.insert_resource(config);
        // Set streak to 4 which exceeds config threshold of 3
        let mut tracker = app.world_mut().resource_mut::<HighlightTracker>();
        tracker.best_perfect_streak = 4;
        tracker.consecutive_perfect_bumps = 4;

        app.insert_resource(TestNodeCleared(true));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        let streak = stats
            .highlights
            .iter()
            .find(|h| h.kind == HighlightKind::PerfectStreak);
        assert!(
            streak.is_some(),
            "should detect PerfectStreak with config.perfect_streak_count=3 and streak=4"
        );
        assert!(
            (streak.unwrap().value - 4.0).abs() < f32::EPSILON,
            "PerfectStreak value should be 4.0"
        );
    }

    // --- Behavior 7: highlight cap reads from config ---

    #[test]
    fn highlights_stored_beyond_four_existing() {
        let mut app = test_app();
        // Cap removed — highlights always stored, selection at run-end
        // Pre-fill 4 highlights
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
            RunHighlight {
                kind: HighlightKind::ClutchClear,
                node_index: 3,
                value: 1.0,
            },
        ];

        // Set up conditions that would produce a NoDamageNode highlight
        app.world_mut()
            .resource_mut::<HighlightTracker>()
            .node_bolts_lost = 0;
        app.insert_resource(NodeTimer {
            remaining: 15.0,
            total: 30.0,
        });
        app.insert_resource(TestNodeCleared(true));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        assert!(
            stats.highlights.len() > 4,
            "highlights should be stored beyond 4 existing — no cap during detection"
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
            RunHighlight {
                kind: HighlightKind::ClutchClear,
                node_index: 3,
                value: 1.0,
            },
            RunHighlight {
                kind: HighlightKind::FastClear,
                node_index: 4,
                value: 0.0,
            },
        ];

        // Set up conditions that would produce a ClutchClear
        app.insert_resource(NodeTimer {
            remaining: 1.0,
            total: 30.0,
        });
        app.insert_resource(TestNodeCleared(true));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        assert!(
            stats.highlights.len() > 5,
            "highlights should NOT be capped at 5 — selection happens at run-end, not during detection. Got {}",
            stats.highlights.len()
        );
    }

    // --- Behavior: same kind stored across nodes even beyond old cap ---

    #[test]
    fn same_kind_stored_across_nodes_beyond_old_cap() {
        let mut app = test_app();
        // Pre-fill 5 highlights including a ClutchClear — previously at cap
        let mut stats = app.world_mut().resource_mut::<RunStats>();
        stats.highlights = vec![
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
                kind: HighlightKind::PerfectStreak,
                node_index: 2,
                value: 5.0,
            },
            RunHighlight {
                kind: HighlightKind::FastClear,
                node_index: 3,
                value: 0.0,
            },
            RunHighlight {
                kind: HighlightKind::NoDamageNode,
                node_index: 4,
                value: 0.0,
            },
        ];

        // Set up conditions for another ClutchClear on a later node
        app.world_mut().resource_mut::<RunState>().node_index = 5;
        app.insert_resource(NodeTimer {
            remaining: 1.0,
            total: 30.0,
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

    // --- Behavior 9: SpeedDemon detected when elapsed < config.speed_demon_secs ---

    #[test]
    fn speed_demon_detected_when_elapsed_below_threshold() {
        let mut app = test_app();
        let config = HighlightConfig {
            speed_demon_secs: 5.0,
            ..Default::default()
        };
        app.insert_resource(config);
        // node_start_time=10.0, current time will be ~10.0 + accumulated ticks
        // We set node_start_time and advance time to simulate elapsed=4.5
        app.world_mut()
            .resource_mut::<HighlightTracker>()
            .node_start_time = 10.0;

        // We need Time<Fixed> to report elapsed_secs around 14.5 (elapsed = 14.5 - 10.0 = 4.5)
        // Advance fixed time by accumulating many timesteps to reach ~14.5s
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        let ticks_needed = u32::try_from(
            std::time::Duration::from_millis(14500).as_micros() / timestep.as_micros(),
        )
        .expect("tick count fits in u32");
        app.insert_resource(TestNodeCleared(false));
        for _ in 0..ticks_needed {
            tick(&mut app);
        }

        // Now send the NodeCleared
        app.insert_resource(TestNodeCleared(true));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        let speed_demon = stats
            .highlights
            .iter()
            .find(|h| h.kind == HighlightKind::SpeedDemon);
        assert!(
            speed_demon.is_some(),
            "should detect SpeedDemon when node elapsed (~4.5s) < config.speed_demon_secs (5.0)"
        );
    }

    // --- Behavior 10: SpeedDemon NOT detected when elapsed >= threshold ---

    #[test]
    fn speed_demon_not_detected_when_elapsed_above_threshold() {
        let mut app = test_app();
        let config = HighlightConfig {
            speed_demon_secs: 5.0,
            ..Default::default()
        };
        app.insert_resource(config);
        // node_start_time=10.0, advance time to ~16.0 -> elapsed=6.0 >= 5.0
        app.world_mut()
            .resource_mut::<HighlightTracker>()
            .node_start_time = 10.0;

        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        let ticks_needed =
            u32::try_from(std::time::Duration::from_secs(16).as_micros() / timestep.as_micros())
                .expect("tick count fits in u32");
        app.insert_resource(TestNodeCleared(false));
        for _ in 0..ticks_needed {
            tick(&mut app);
        }

        app.insert_resource(TestNodeCleared(true));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        let speed_demon = stats
            .highlights
            .iter()
            .find(|h| h.kind == HighlightKind::SpeedDemon);
        assert!(
            speed_demon.is_none(),
            "should NOT detect SpeedDemon when node elapsed (~6.0s) >= config.speed_demon_secs (5.0)"
        );
    }

    // --- Behavior 11: Untouchable detected after consecutive no-damage nodes ---

    #[test]
    fn untouchable_detected_after_consecutive_no_damage_nodes() {
        let mut app = test_app();
        let config = HighlightConfig {
            untouchable_nodes: 2,
            ..Default::default()
        };
        app.insert_resource(config);
        // 1 existing consecutive no-damage node + current node with 0 bolts lost = 2 >= 2
        let mut tracker = app.world_mut().resource_mut::<HighlightTracker>();
        tracker.consecutive_no_damage_nodes = 1;
        tracker.node_bolts_lost = 0;

        app.insert_resource(TestNodeCleared(true));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        let untouchable = stats
            .highlights
            .iter()
            .find(|h| h.kind == HighlightKind::Untouchable);
        assert!(
            untouchable.is_some(),
            "should detect Untouchable with 1 existing + current no-damage = 2 >= config.untouchable_nodes=2"
        );
    }

    // --- Behavior 12: Untouchable streak resets when bolt lost ---

    #[test]
    fn untouchable_streak_resets_when_bolt_lost() {
        let mut app = test_app();
        let config = HighlightConfig {
            untouchable_nodes: 2,
            ..Default::default()
        };
        app.insert_resource(config);
        // Had 3 consecutive no-damage nodes, but lost a bolt this node
        let mut tracker = app.world_mut().resource_mut::<HighlightTracker>();
        tracker.consecutive_no_damage_nodes = 3;
        tracker.node_bolts_lost = 1;

        app.insert_resource(TestNodeCleared(true));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        let untouchable = stats
            .highlights
            .iter()
            .find(|h| h.kind == HighlightKind::Untouchable);
        assert!(
            untouchable.is_none(),
            "should NOT detect Untouchable when node_bolts_lost=1 (streak reset)"
        );
        // Also verify the streak was reset
        let tracker = app.world().resource::<HighlightTracker>();
        assert_eq!(
            tracker.consecutive_no_damage_nodes, 0,
            "consecutive_no_damage_nodes should reset to 0 when bolt was lost"
        );
    }

    // --- Behavior 13: Comeback when bolts lost >= threshold ---

    #[test]
    fn comeback_detected_when_bolts_lost_exceeds_threshold() {
        let mut app = test_app();
        let config = HighlightConfig {
            comeback_bolts_lost: 3,
            ..Default::default()
        };
        app.insert_resource(config);
        app.world_mut()
            .resource_mut::<HighlightTracker>()
            .node_bolts_lost = 3;

        app.insert_resource(TestNodeCleared(true));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        let comeback = stats
            .highlights
            .iter()
            .find(|h| h.kind == HighlightKind::Comeback);
        assert!(
            comeback.is_some(),
            "should detect Comeback with node_bolts_lost=3 >= config.comeback_bolts_lost=3"
        );
    }

    // --- Behavior 14: PerfectNode when all bumps perfect ---

    #[test]
    fn perfect_node_detected_when_all_bumps_perfect() {
        let mut app = test_app();
        let mut tracker = app.world_mut().resource_mut::<HighlightTracker>();
        tracker.non_perfect_bumps_this_node = 0;
        tracker.total_bumps_this_node = 5;

        app.insert_resource(TestNodeCleared(true));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        let perfect_node = stats
            .highlights
            .iter()
            .find(|h| h.kind == HighlightKind::PerfectNode);
        assert!(
            perfect_node.is_some(),
            "should detect PerfectNode when non_perfect=0 and total=5"
        );
    }

    // --- Behavior 15: PerfectNode NOT detected with non-perfect bumps ---

    #[test]
    fn perfect_node_not_detected_with_non_perfect_bumps() {
        let mut app = test_app();
        let mut tracker = app.world_mut().resource_mut::<HighlightTracker>();
        tracker.non_perfect_bumps_this_node = 1;
        tracker.total_bumps_this_node = 5;

        app.insert_resource(TestNodeCleared(true));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        let perfect_node = stats
            .highlights
            .iter()
            .find(|h| h.kind == HighlightKind::PerfectNode);
        assert!(
            perfect_node.is_none(),
            "should NOT detect PerfectNode when non_perfect_bumps_this_node=1"
        );
    }

    // ---------------------------------------------------------------
    // Regression: stale best_perfect_streak duplicates on later nodes
    // ---------------------------------------------------------------

    #[test]
    fn perfect_streak_not_duplicated_from_stale_cross_node_best() {
        // Bug: best_perfect_streak is a cross-node field. After node 2
        // achieves a streak of 7 (above threshold 5), every subsequent
        // NodeCleared records another PerfectStreak with value=7, even
        // if the current node had 0 consecutive perfect bumps.
        //
        // Correct behavior: PerfectStreak should only fire when the
        // current node actually contributes to the streak, not when
        // re-reading a stale cross-node value.
        let mut app = test_app();

        // Simulate state after a previous node set best_perfect_streak=7,
        // but the current node has 0 consecutive perfect bumps.
        let mut tracker = app.world_mut().resource_mut::<HighlightTracker>();
        tracker.best_perfect_streak = 7;
        tracker.consecutive_perfect_bumps = 0;

        // Default perfect_streak_count = 5, so 7 >= 5 would fire
        // if the system doesn't guard against stale values.

        app.insert_resource(TestNodeCleared(true));
        tick(&mut app);

        let stats = app.world().resource::<RunStats>();
        let streak = stats
            .highlights
            .iter()
            .find(|h| h.kind == HighlightKind::PerfectStreak);
        assert!(
            streak.is_none(),
            "PerfectStreak should NOT fire when the current node had 0 consecutive \
             perfect bumps, even though best_perfect_streak=7 from a prior node"
        );
    }
}
