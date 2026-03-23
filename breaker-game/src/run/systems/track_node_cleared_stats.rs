//! System to record node-level stats and detect highlights on node clear.

use bevy::prelude::*;

use crate::run::{
    definition::HighlightConfig,
    node::{messages::NodeCleared, resources::NodeTimer},
    resources::{HighlightKind, HighlightTracker, RunHighlight, RunState, RunStats},
};

/// Reads [`NodeCleared`] messages and updates [`RunStats`] with node
/// completion data and highlight detection.
///
/// Uses [`HighlightConfig`] for all thresholds and the highlight cap.
pub(crate) fn track_node_cleared_stats(
    mut reader: MessageReader<NodeCleared>,
    mut stats: ResMut<RunStats>,
    mut tracker: ResMut<HighlightTracker>,
    run_state: Res<RunState>,
    node_timer: Res<NodeTimer>,
    config: Res<HighlightConfig>,
    time: Res<Time<Fixed>>,
) {
    let cap = config.highlight_cap as usize;

    for _msg in reader.read() {
        stats.nodes_cleared += 1;

        if stats.highlights.len() >= cap {
            continue;
        }

        let node_index = run_state.node_index;

        // ClutchClear: cleared with less than threshold seconds remaining
        if node_timer.remaining < config.clutch_clear_secs && stats.highlights.len() < cap {
            stats.highlights.push(RunHighlight {
                kind: HighlightKind::ClutchClear,
                node_index,
                value: node_timer.remaining,
            });
        }

        // NoDamageNode: no bolts lost this node
        if tracker.node_bolts_lost == 0 && stats.highlights.len() < cap {
            stats.highlights.push(RunHighlight {
                kind: HighlightKind::NoDamageNode,
                node_index,
                value: 0.0,
            });
        }

        // FastClear: elapsed time < fraction of total
        let elapsed = node_timer.total - node_timer.remaining;
        if elapsed < node_timer.total * config.fast_clear_fraction && stats.highlights.len() < cap {
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
        if tracker.best_perfect_streak >= config.perfect_streak_count
            && stats.highlights.len() < cap
        {
            stats.highlights.push(RunHighlight {
                kind: HighlightKind::PerfectStreak,
                node_index,
                value: {
                    let streak = tracker.best_perfect_streak.min(u32::from(u16::MAX));
                    f32::from(u16::try_from(streak).unwrap_or(u16::MAX))
                },
            });
        }

        // SpeedDemon: node elapsed time below threshold
        let node_elapsed = time.elapsed_secs() - tracker.node_start_time;
        if node_elapsed < config.speed_demon_secs && stats.highlights.len() < cap {
            stats.highlights.push(RunHighlight {
                kind: HighlightKind::SpeedDemon,
                node_index,
                value: node_elapsed,
            });
        }
        tracker.fastest_node_clear_secs = tracker.fastest_node_clear_secs.min(node_elapsed);

        // Untouchable: consecutive no-damage nodes
        if tracker.node_bolts_lost == 0 {
            tracker.consecutive_no_damage_nodes += 1;
        } else {
            tracker.consecutive_no_damage_nodes = 0;
        }
        if tracker.consecutive_no_damage_nodes >= config.untouchable_nodes
            && stats.highlights.len() < cap
        {
            stats.highlights.push(RunHighlight {
                kind: HighlightKind::Untouchable,
                node_index,
                value: f32::from(
                    u16::try_from(tracker.consecutive_no_damage_nodes).unwrap_or(u16::MAX),
                ),
            });
        }

        // Comeback: cleared despite losing many bolts
        if tracker.node_bolts_lost >= config.comeback_bolts_lost && stats.highlights.len() < cap {
            stats.highlights.push(RunHighlight {
                kind: HighlightKind::Comeback,
                node_index,
                value: f32::from(u16::try_from(tracker.node_bolts_lost).unwrap_or(u16::MAX)),
            });
        }

        // PerfectNode: every bump was perfect grade
        if tracker.non_perfect_bumps_this_node == 0
            && tracker.total_bumps_this_node > 0
            && stats.highlights.len() < cap
        {
            stats.highlights.push(RunHighlight {
                kind: HighlightKind::PerfectNode,
                node_index,
                value: f32::from(u16::try_from(tracker.total_bumps_this_node).unwrap_or(u16::MAX)),
            });
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
    fn highlight_cap_from_config_allows_fifth_highlight() {
        let mut app = test_app();
        // Config default highlight_cap=5
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
        assert_eq!(
            stats.highlights.len(),
            5,
            "with highlight_cap=5, 5th highlight should be added (4 existing + 1 new)"
        );
    }

    // --- Behavior 8: cap at 5 enforced ---

    #[test]
    fn highlight_cap_at_five_prevents_sixth() {
        let mut app = test_app();
        // Config default highlight_cap=5
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
        assert_eq!(
            stats.highlights.len(),
            5,
            "highlights should be capped at 5, 6th highlight should be dropped"
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
        let ticks_needed = u32::try_from(
            std::time::Duration::from_secs(16).as_micros() / timestep.as_micros(),
        )
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
}
