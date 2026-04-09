//! System to select the active node layout for the current node.

use bevy::prelude::*;
use tracing::warn;

use crate::state::run::{
    node::{ActiveNodeLayout, NodeLayoutRegistry, ScenarioLayoutOverride},
    resources::NodeOutcome,
};

/// Selects the active node layout based on the current node index.
///
/// Runs on `OnEnter(GameState::Playing)`, before `spawn_cells_from_layout`.
/// Wraps around if `node_index` exceeds the number of layouts.
///
/// If [`ScenarioLayoutOverride`] is `Some(name)`, that named layout is used
/// instead of the index-based selection. Falls back to index selection if the
/// named layout is not found.
pub(crate) fn set_active_layout(
    run_state: Res<NodeOutcome>,
    registry: Res<NodeLayoutRegistry>,
    override_res: Res<ScenarioLayoutOverride>,
    mut commands: Commands,
) {
    if registry.is_empty() {
        warn!("NodeLayoutRegistry is empty — no layout to set");
        return;
    }

    if let Some(name) = &override_res.0 {
        if let Some(layout) = registry.get_by_name(name) {
            commands.insert_resource(ActiveNodeLayout(layout.clone()));
            return;
        }
        warn!(
            "ScenarioLayoutOverride: layout '{}' not found, falling back to index selection",
            name
        );
    }

    let index = run_state.node_index as usize % registry.len();
    let Some(layout) = registry.get_by_index(index) else {
        return;
    };
    commands.insert_resource(ActiveNodeLayout(layout.clone()));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::run::node::{NodeLayout, ScenarioLayoutOverride, definition::NodePool};

    fn make_layout(name: &str) -> NodeLayout {
        NodeLayout {
            name: name.to_owned(),
            timer_secs: 60.0,
            cols: 2,
            rows: 1,
            grid_top_offset: 50.0,
            grid: vec![vec!["S".to_owned(), "S".to_owned()]],
            pool: NodePool::default(),
            entity_scale: 1.0,
            locks: None,
        }
    }

    fn make_node_registry(layouts: Vec<NodeLayout>) -> NodeLayoutRegistry {
        let mut registry = NodeLayoutRegistry::default();
        for layout in layouts {
            registry.insert(layout);
        }
        registry
    }

    fn test_app(node_index: u32, layouts: Vec<NodeLayout>) -> App {
        use crate::shared::test_utils::TestAppBuilder;
        TestAppBuilder::new()
            .insert_resource(NodeOutcome {
                node_index,
                ..default()
            })
            .insert_resource(make_node_registry(layouts))
            .insert_resource(ScenarioLayoutOverride::default())
            .with_system(Startup, set_active_layout)
            .build()
    }

    #[test]
    fn override_selects_named_layout_ignoring_node_index() {
        let layouts = vec![make_layout("corridor"), make_layout("open")];
        let mut app = test_app(0, layouts);
        app.world_mut()
            .insert_resource(ScenarioLayoutOverride(Some("open".to_owned())));
        app.update();

        let active = app.world().resource::<ActiveNodeLayout>();
        assert_eq!(active.0.name, "open");
    }

    #[test]
    fn override_none_falls_through_to_index_selection() {
        let layouts = vec![make_layout("corridor"), make_layout("open")];
        let mut app = test_app(0, layouts);
        // ScenarioLayoutOverride::default() is None — index 0 = "corridor"
        app.update();

        let active = app.world().resource::<ActiveNodeLayout>();
        assert_eq!(active.0.name, "corridor");
    }

    #[test]
    fn override_unknown_name_falls_back_to_index_selection() {
        let layouts = vec![make_layout("corridor"), make_layout("open")];
        let mut app = test_app(1, layouts);
        app.world_mut()
            .insert_resource(ScenarioLayoutOverride(Some("missing".to_owned())));
        app.update();

        // Falls back to index 1 = "open"
        let active = app.world().resource::<ActiveNodeLayout>();
        assert_eq!(active.0.name, "open");
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
