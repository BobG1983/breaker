//! System to initialize the node timer from the active layout.

use bevy::prelude::*;

use crate::run::node::{ActiveNodeLayout, NodeTimer};

/// Initializes [`NodeTimer`] from the active node layout's `timer_secs`.
///
/// Runs on `OnEnter(GameState::Playing)`, after `set_active_layout`.
pub fn init_node_timer(layout: Res<ActiveNodeLayout>, mut commands: Commands) {
    let secs = layout.0.timer_secs;
    commands.insert_resource(NodeTimer {
        remaining: secs,
        total: secs,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::run::node::{NodeLayout, definition::NodePool};

    fn test_app(timer_secs: f32) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ActiveNodeLayout(NodeLayout {
                name: "test".to_owned(),
                timer_secs,
                cols: 2,
                rows: 1,
                grid_top_offset: 50.0,
                grid: vec![vec!['S', 'S']],
                pool: NodePool::default(),
            }))
            .add_systems(Startup, init_node_timer);
        app
    }

    #[test]
    fn sets_timer_from_layout() {
        let mut app = test_app(45.0);
        app.update();

        let timer = app.world().resource::<NodeTimer>();
        assert!((timer.remaining - 45.0).abs() < f32::EPSILON);
        assert!((timer.total - 45.0).abs() < f32::EPSILON);
    }

    #[test]
    fn different_layout_different_timer() {
        let mut app = test_app(90.0);
        app.update();

        let timer = app.world().resource::<NodeTimer>();
        assert!((timer.remaining - 90.0).abs() < f32::EPSILON);
    }
}
