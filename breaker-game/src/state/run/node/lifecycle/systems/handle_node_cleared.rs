//! System to handle node cleared events — advance or win.

use bevy::prelude::*;
use rantzsoft_lifecycle::ChangeState;
use tracing::warn;

use crate::state::{
    run::{
        node::{NodeLayoutRegistry, messages::NodeCleared},
        resources::{NodeOutcome, NodeResult, NodeSequence},
    },
    types::NodeState,
};

/// When [`NodeCleared`] is received, set [`NodeOutcome`] and transition to
/// [`NodeState::AnimateOut`]. The teardown router reads `NodeOutcome` to decide
/// whether to go to `ChipSelect` or `RunEnd`.
pub(crate) fn handle_node_cleared(
    mut reader: MessageReader<NodeCleared>,
    registry: Res<NodeLayoutRegistry>,
    node_sequence: Option<Res<NodeSequence>>,
    mut run_state: ResMut<NodeOutcome>,
    mut writer: MessageWriter<ChangeState<NodeState>>,
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

    if (run_state.node_index as usize) >= final_index {
        run_state.result = NodeResult::Won;
    }

    writer.write(ChangeState::new());
}

#[cfg(test)]
mod tests {
    use bevy::{ecs::message::Messages, state::app::StatesPlugin};
    use rantzsoft_lifecycle::ChangeState;

    use super::*;
    use crate::state::{
        run::node::{NodeLayout, definition::NodePool},
        types::{AppState, GameState, RunState},
    };

    fn make_layout(name: &str) -> NodeLayout {
        NodeLayout {
            name: name.to_owned(),
            timer_secs: 60.0,
            cols: 2,
            rows: 1,
            grid_top_offset: 50.0,
            grid: vec![vec!['S', 'S']],
            pool: NodePool::default(),
            entity_scale: 1.0,
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
            .init_state::<AppState>()
            .add_sub_state::<GameState>()
            .add_sub_state::<RunState>()
            .add_sub_state::<NodeState>()
            .add_message::<NodeCleared>()
            .add_message::<ChangeState<NodeState>>();
        let mut registry = NodeLayoutRegistry::default();
        for i in 0..layout_count {
            registry.insert(make_layout(&format!("node_{i}")));
        }
        app.insert_resource(registry)
            .insert_resource(NodeOutcome {
                node_index,
                ..default()
            })
            .insert_resource(SendNodeCleared(false))
            .add_systems(FixedUpdate, (send_cleared, handle_node_cleared).chain());
        // Navigate to NodeState so the system can set NextState<NodeState>
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Game);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Run);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<RunState>>()
            .set(RunState::Node);
        app.update();
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
    fn non_final_node_transitions_to_animate_out() {
        let mut app = test_app(0, 3);
        app.world_mut().resource_mut::<SendNodeCleared>().0 = true;
        tick(&mut app);

        let msgs = app.world().resource::<Messages<ChangeState<NodeState>>>();
        assert!(
            msgs.iter_current_update_messages().count() > 0,
            "expected ChangeState<NodeState> message"
        );
        let run_state = app.world().resource::<NodeOutcome>();
        assert!(run_state.transition_queued);
        assert_eq!(run_state.result, NodeResult::InProgress);
    }

    #[test]
    fn final_node_transitions_to_animate_out_with_won() {
        let mut app = test_app(2, 3); // index 2 of 3 layouts = final
        app.world_mut().resource_mut::<SendNodeCleared>().0 = true;
        tick(&mut app);

        let msgs = app.world().resource::<Messages<ChangeState<NodeState>>>();
        assert!(
            msgs.iter_current_update_messages().count() > 0,
            "expected ChangeState<NodeState> message"
        );

        let run_state = app.world().resource::<NodeOutcome>();
        assert_eq!(run_state.result, NodeResult::Won);
        assert!(run_state.transition_queued);
    }

    #[test]
    fn empty_registry_does_not_transition() {
        let mut app = test_app(0, 0); // 0 layouts
        app.world_mut().resource_mut::<SendNodeCleared>().0 = true;
        tick(&mut app);

        let msgs = app.world().resource::<Messages<ChangeState<NodeState>>>();
        assert_eq!(
            msgs.iter_current_update_messages().count(),
            0,
            "empty registry should not trigger any ChangeState message"
        );
        let run_state = app.world().resource::<NodeOutcome>();
        assert_eq!(run_state.result, NodeResult::InProgress);
    }

    #[test]
    fn no_message_no_change() {
        let mut app = test_app(0, 3);
        // SendNodeCleared stays false
        tick(&mut app);

        let msgs = app.world().resource::<Messages<ChangeState<NodeState>>>();
        assert_eq!(
            msgs.iter_current_update_messages().count(),
            0,
            "expected no ChangeState message"
        );
    }

    // ── NodeSequence-length regression tests (A3) ─────────────────────

    use crate::state::run::{
        definition::NodeType,
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
    fn test_app_with_sequence(node_index: u32, layout_count: usize, sequence_len: usize) -> App {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .init_state::<AppState>()
            .add_sub_state::<GameState>()
            .add_sub_state::<RunState>()
            .add_sub_state::<NodeState>()
            .add_message::<NodeCleared>()
            .add_message::<ChangeState<NodeState>>();
        let mut registry = NodeLayoutRegistry::default();
        for i in 0..layout_count {
            registry.insert(make_layout(&format!("node_{i}")));
        }
        app.insert_resource(registry)
            .insert_resource(make_node_sequence(sequence_len))
            .insert_resource(NodeOutcome {
                node_index,
                ..default()
            })
            .insert_resource(SendNodeCleared(false))
            .add_systems(FixedUpdate, (send_cleared, handle_node_cleared).chain());
        // Navigate to NodeState
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Game);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Run);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<RunState>>()
            .set(RunState::Node);
        app.update();
        app
    }

    #[test]
    fn mid_run_node_does_not_end_game_when_sequence_longer_than_registry() {
        let mut app = test_app_with_sequence(3, 3, 9);
        app.world_mut().resource_mut::<SendNodeCleared>().0 = true;
        tick(&mut app);

        let msgs = app.world().resource::<Messages<ChangeState<NodeState>>>();
        assert!(
            msgs.iter_current_update_messages().count() > 0,
            "node_index 3 of 9 should send ChangeState<NodeState> message"
        );

        let run_state = app.world().resource::<NodeOutcome>();
        assert_eq!(
            run_state.result,
            NodeResult::InProgress,
            "run should still be in progress at node 3 of 9"
        );
    }

    #[test]
    fn run_ends_at_last_node_sequence_assignment() {
        let mut app = test_app_with_sequence(8, 3, 9);
        app.world_mut().resource_mut::<SendNodeCleared>().0 = true;
        tick(&mut app);

        let msgs = app.world().resource::<Messages<ChangeState<NodeState>>>();
        assert!(
            msgs.iter_current_update_messages().count() > 0,
            "node_index 8 of 9 should send ChangeState<NodeState> message"
        );

        let run_state = app.world().resource::<NodeOutcome>();
        assert_eq!(
            run_state.result,
            NodeResult::Won,
            "outcome should be Won at final node"
        );
    }

    #[test]
    fn penultimate_node_transitions_to_animate_out_not_run_end() {
        let mut app = test_app_with_sequence(7, 3, 9);
        app.world_mut().resource_mut::<SendNodeCleared>().0 = true;
        tick(&mut app);

        let msgs = app.world().resource::<Messages<ChangeState<NodeState>>>();
        assert!(
            msgs.iter_current_update_messages().count() > 0,
            "node_index 7 of 9 should send ChangeState<NodeState> message"
        );

        let run_state = app.world().resource::<NodeOutcome>();
        assert_eq!(
            run_state.result,
            NodeResult::InProgress,
            "run should still be in progress at penultimate node"
        );
    }
}
