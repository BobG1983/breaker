//! System to handle node cleared events — advance or win.

use bevy::prelude::*;

use crate::{
    run::{
        messages::NodeCleared,
        node::NodeLayoutRegistry,
        resources::{RunOutcome, RunState},
    },
    shared::GameState,
};

/// When [`NodeCleared`] is received, either advance to the next node or win the run.
pub fn handle_node_cleared(
    mut reader: MessageReader<NodeCleared>,
    registry: Res<NodeLayoutRegistry>,
    mut run_state: ResMut<RunState>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if reader.read().next().is_none() {
        return;
    }

    let final_index = registry.layouts.len().saturating_sub(1);

    if (run_state.node_index as usize) < final_index {
        next_state.set(GameState::NodeTransition);
    } else {
        run_state.outcome = RunOutcome::Won;
        next_state.set(GameState::RunEnd);
    }
}

#[cfg(test)]
mod tests {
    use bevy::state::app::StatesPlugin;

    use super::*;
    use crate::run::node::NodeLayout;

    fn make_layout(name: &str) -> NodeLayout {
        NodeLayout {
            name: name.to_owned(),
            timer_secs: 60.0,
            cols: 2,
            rows: 1,
            grid_top_offset: 50.0,
            grid: vec![vec!['S', 'S']],
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
        app.add_plugins((MinimalPlugins, StatesPlugin));
        app.init_state::<GameState>();
        app.add_message::<NodeCleared>();
        let layouts: Vec<NodeLayout> = (0..layout_count)
            .map(|i| make_layout(&format!("node_{i}")))
            .collect();
        app.insert_resource(NodeLayoutRegistry { layouts });
        app.insert_resource(RunState {
            node_index,
            ..default()
        });
        app.insert_resource(SendNodeCleared(false));
        app.add_systems(FixedUpdate, (send_cleared, handle_node_cleared).chain());
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
    fn non_final_node_transitions_to_node_transition() {
        let mut app = test_app(0, 3);
        app.world_mut().resource_mut::<SendNodeCleared>().0 = true;
        tick(&mut app);

        let next = app.world().resource::<NextState<GameState>>();
        assert!(
            format!("{next:?}").contains("NodeTransition"),
            "expected NodeTransition, got: {next:?}"
        );
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
    }

    #[test]
    fn no_message_no_change() {
        let mut app = test_app(0, 3);
        // SendNodeCleared stays false
        tick(&mut app);

        let next = app.world().resource::<NextState<GameState>>();
        let debug = format!("{next:?}");
        assert!(
            !debug.contains("NodeTransition") && !debug.contains("RunEnd"),
            "expected no state change, got: {next:?}"
        );
    }
}
