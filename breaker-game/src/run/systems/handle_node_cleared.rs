//! System to handle node cleared events — advance or win.

use bevy::prelude::*;
use tracing::warn;

use crate::{
    run::{
        node::{NodeLayoutRegistry, messages::NodeCleared},
        resources::{NodeSequence, RunOutcome, RunState},
    },
    shared::GameState,
};

/// When [`NodeCleared`] is received, either advance to the next node or win the run.
pub fn handle_node_cleared(
    mut reader: MessageReader<NodeCleared>,
    registry: Res<NodeLayoutRegistry>,
    node_sequence: Option<Res<NodeSequence>>,
    mut run_state: ResMut<RunState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if reader.read().next().is_none() {
        return;
    }

    let total_nodes = node_sequence
        .as_ref()
        .map_or_else(|| registry.len(), |seq| seq.assignments.len());

    if total_nodes == 0 {
        warn!("NodeCleared received but no nodes in sequence or registry — ignoring");
        return;
    }

    let final_index = total_nodes.saturating_sub(1);

    run_state.transition_queued = true;

    if (run_state.node_index as usize) < final_index {
        next_state.set(GameState::ChipSelect);
    } else {
        run_state.outcome = RunOutcome::Won;
        next_state.set(GameState::RunEnd);
    }
}

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;

    use super::*;
    use crate::run::node::{NodeLayout, definition::NodePool};

    fn make_layout(name: &str) -> NodeLayout {
        NodeLayout {
            name: name.to_owned(),
            timer_secs: 60.0,
            cols: 2,
            rows: 1,
            grid_top_offset: 50.0,
            grid: vec![vec!['S', 'S']],
            pool: NodePool::default(),
        }
    }

    #[derive(Resource)]
    struct SendNodeCleared(bool);

    fn send_cleared(flag: Res<SendNodeCleared>, mut writer: MessageWriter<NodeCleared>) {
        if flag.0 {
            writer.write(NodeCleared);
        }
    }

    fn test_app(node_index: u32, layout_count: usize) -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_state::<GameState>()
            .add_message::<NodeCleared>();
        let mut registry = NodeLayoutRegistry::default();
        for i in 0..layout_count {
            registry.insert(make_layout(&format!("node_{i}")));
        }
        app.insert_resource(registry)
            .insert_resource(RunState {
                node_index,
                ..default()
            })
            .insert_resource(SendNodeCleared(false))
            .add_systems(FixedUpdate, (send_cleared, handle_node_cleared).chain());
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
    fn non_final_node_transitions_to_chip_select() {
        let mut app = test_app(0, 3);
        app.world_mut().resource_mut::<SendNodeCleared>().0 = true;
        tick(&mut app);

        let next = app.world().resource::<NextState<GameState>>();
        assert!(
            format!("{next:?}").contains("ChipSelect"),
            "expected ChipSelect, got: {next:?}"
        );
        let run_state = app.world().resource::<RunState>();
        assert!(run_state.transition_queued);
    }

    #[test]
    fn final_node_transitions_to_run_end_with_won() {
        let mut app = test_app(2, 3); // index 2 of 3 layouts = final
        app.world_mut().resource_mut::<SendNodeCleared>().0 = true;
        tick(&mut app);

        let next = app.world().resource::<NextState<GameState>>();
        assert!(
            format!("{next:?}").contains("RunEnd"),
            "expected RunEnd, got: {next:?}"
        );

        let run_state = app.world().resource::<RunState>();
        assert_eq!(run_state.outcome, RunOutcome::Won);
        assert!(run_state.transition_queued);
    }

    #[test]
    fn empty_registry_does_not_transition() {
        let mut app = test_app(0, 0); // 0 layouts
        app.world_mut().resource_mut::<SendNodeCleared>().0 = true;
        tick(&mut app);

        let next = app.world().resource::<NextState<GameState>>();
        let debug = format!("{next:?}");
        assert!(
            !debug.contains("ChipSelect") && !debug.contains("RunEnd"),
            "empty registry should not trigger any transition, got: {next:?}"
        );
        let run_state = app.world().resource::<RunState>();
        assert_eq!(run_state.outcome, RunOutcome::InProgress);
    }

    #[test]
    fn no_message_no_change() {
        let mut app = test_app(0, 3);
        // SendNodeCleared stays false
        tick(&mut app);

        let next = app.world().resource::<NextState<GameState>>();
        let debug = format!("{next:?}");
        assert!(
            !debug.contains("ChipSelect") && !debug.contains("RunEnd"),
            "expected no state change, got: {next:?}"
        );
    }

    // ── NodeSequence-length regression tests (A3) ─────────────────────

    use crate::run::{
        difficulty::NodeType,
        resources::{NodeAssignment, NodeSequence},
    };

    /// Helper: build a [`NodeSequence`] with `count` assignments (all Passive, tier 0).
    fn make_node_sequence(count: usize) -> NodeSequence {
        NodeSequence {
            assignments: (0..count)
                .map(|_| NodeAssignment {
                    node_type: NodeType::Passive,
                    tier_index: 0,
                    hp_mult: 1.0,
                    timer_mult: 1.0,
                })
                .collect(),
        }
    }

    /// Helper: build an app with *both* a [`NodeLayoutRegistry`] and a [`NodeSequence`].
    ///
    /// `layout_count` controls the registry size; `sequence_len` controls the
    /// node sequence length. The system should use `sequence_len` to determine
    /// the final node index, **not** `layout_count`.
    fn test_app_with_sequence(node_index: u32, layout_count: usize, sequence_len: usize) -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_state::<GameState>()
            .add_message::<NodeCleared>();
        let mut registry = NodeLayoutRegistry::default();
        for i in 0..layout_count {
            registry.insert(make_layout(&format!("node_{i}")));
        }
        app.insert_resource(registry)
            .insert_resource(make_node_sequence(sequence_len))
            .insert_resource(RunState {
                node_index,
                ..default()
            })
            .insert_resource(SendNodeCleared(false))
            .add_systems(FixedUpdate, (send_cleared, handle_node_cleared).chain());
        app
    }

    #[test]
    fn mid_run_node_does_not_end_game_when_sequence_longer_than_registry() {
        // NodeSequence has 9 assignments, registry has 3 layouts.
        // At node_index 3, we are only at the 4th of 9 nodes — NOT the end.
        // Bug: system uses registry.len() (3) and thinks index 3 >= final (2).
        let mut app = test_app_with_sequence(3, 3, 9);
        app.world_mut().resource_mut::<SendNodeCleared>().0 = true;
        tick(&mut app);

        let next = app.world().resource::<NextState<GameState>>();
        assert!(
            format!("{next:?}").contains("ChipSelect"),
            "node_index 3 of 9 should advance to ChipSelect, not end the run; got: {next:?}"
        );

        let run_state = app.world().resource::<RunState>();
        assert_eq!(
            run_state.outcome,
            RunOutcome::InProgress,
            "run should still be in progress at node 3 of 9"
        );
    }

    #[test]
    fn run_ends_at_last_node_sequence_assignment() {
        // NodeSequence has 9 assignments (final_index = 8), registry has 3 layouts.
        // At node_index 8, we are at the last node — run should end with Won.
        let mut app = test_app_with_sequence(8, 3, 9);
        app.world_mut().resource_mut::<SendNodeCleared>().0 = true;
        tick(&mut app);

        let next = app.world().resource::<NextState<GameState>>();
        assert!(
            format!("{next:?}").contains("RunEnd"),
            "node_index 8 of 9 should trigger RunEnd; got: {next:?}"
        );

        let run_state = app.world().resource::<RunState>();
        assert_eq!(
            run_state.outcome,
            RunOutcome::Won,
            "outcome should be Won at final node"
        );
    }

    #[test]
    fn penultimate_node_transitions_to_chip_select_not_run_end() {
        // NodeSequence has 9 assignments (final_index = 8), registry has 3 layouts.
        // At node_index 7, we are one before the last — should go to ChipSelect.
        let mut app = test_app_with_sequence(7, 3, 9);
        app.world_mut().resource_mut::<SendNodeCleared>().0 = true;
        tick(&mut app);

        let next = app.world().resource::<NextState<GameState>>();
        assert!(
            format!("{next:?}").contains("ChipSelect"),
            "node_index 7 of 9 should advance to ChipSelect; got: {next:?}"
        );

        let run_state = app.world().resource::<RunState>();
        assert_eq!(
            run_state.outcome,
            RunOutcome::InProgress,
            "run should still be in progress at penultimate node"
        );
    }
}
