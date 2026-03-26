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
                detail: None,
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
