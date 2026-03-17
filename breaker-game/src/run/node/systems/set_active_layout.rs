//! System to select the active node layout for the current node.

use bevy::prelude::*;

use crate::run::{
    node::{ActiveNodeLayout, NodeLayoutRegistry},
    resources::RunState,
};

/// Selects the active node layout based on the current node index.
///
/// Runs on `OnEnter(GameState::Playing)`, before `spawn_cells_from_layout`.
/// Wraps around if `node_index` exceeds the number of layouts.
pub fn set_active_layout(
    run_state: Res<RunState>,
    registry: Res<NodeLayoutRegistry>,
    mut commands: Commands,
) {
    if registry.layouts.is_empty() {
        warn!("NodeLayoutRegistry is empty — no layout to set");
        return;
    }

    let index = run_state.node_index as usize % registry.layouts.len();
    let layout = registry.layouts[index].clone();
    commands.insert_resource(ActiveNodeLayout(layout));
}

#[cfg(test)]
mod tests {
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

    fn test_app(node_index: u32, layouts: Vec<NodeLayout>) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(RunState {
            node_index,
            ..default()
        });
        app.insert_resource(NodeLayoutRegistry { layouts });
        app.add_systems(Startup, set_active_layout);
        app
    }

    #[test]
    fn index_zero_selects_first_layout() {
        let layouts = vec![make_layout("first"), make_layout("second")];
        let mut app = test_app(0, layouts);
        app.update();

        let active = app.world().resource::<ActiveNodeLayout>();
        assert_eq!(active.0.name, "first");
    }

    #[test]
    fn index_one_selects_second_layout() {
        let layouts = vec![make_layout("first"), make_layout("second")];
        let mut app = test_app(1, layouts);
        app.update();

        let active = app.world().resource::<ActiveNodeLayout>();
        assert_eq!(active.0.name, "second");
    }

    #[test]
    fn out_of_bounds_wraps() {
        let layouts = vec![
            make_layout("first"),
            make_layout("second"),
            make_layout("third"),
        ];
        let mut app = test_app(5, layouts); // 5 % 3 = 2 → "third"
        app.update();

        let active = app.world().resource::<ActiveNodeLayout>();
        assert_eq!(active.0.name, "third");
    }
}
