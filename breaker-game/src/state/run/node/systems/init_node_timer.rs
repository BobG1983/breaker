//! System to initialize the node timer from the active layout.

use bevy::prelude::*;

use crate::state::run::{
    node::{ActiveNodeLayout, NodeTimer},
    resources::{NodeOutcome, NodeSequence},
};

/// Initializes [`NodeTimer`] from the active node layout's `timer_secs`,
/// scaled by the current node's `timer_mult` from [`NodeSequence`].
///
/// Falls back to a multiplier of `1.0` when [`NodeOutcome`] or [`NodeSequence`]
/// are absent (e.g. in tests or scenario overrides).
///
/// Runs on `OnEnter(GameState::Playing)`, after `set_active_layout`.
pub(crate) fn init_node_timer(
    layout: Res<ActiveNodeLayout>,
    mut commands: Commands,
    run_state: Option<Res<NodeOutcome>>,
    node_sequence: Option<Res<NodeSequence>>,
) {
    let timer_mult = if let (Some(state), Some(sequence)) = (&run_state, &node_sequence) {
        sequence
            .assignments
            .get(state.node_index as usize)
            .map_or(1.0, |a| a.timer_mult)
    } else {
        1.0
    };

    let secs = layout.0.timer_secs * timer_mult;
    commands.insert_resource(NodeTimer {
        remaining: secs,
        total: secs,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::run::{
        definition::NodeType,
        node::{NodeLayout, definition::NodePool},
        resources::{NodeAssignment, NodeOutcome, NodeSequence},
    };

    fn test_app(timer_secs: f32) -> App {
        use crate::shared::test_utils::TestAppBuilder;
        TestAppBuilder::new()
            .insert_resource(ActiveNodeLayout(NodeLayout {
                name: "test".to_owned(),
                timer_secs,
                cols: 2,
                rows: 1,
                grid_top_offset: 50.0,
                grid: vec![vec!["S".to_owned(), "S".to_owned()]],
                pool: NodePool::default(),
                entity_scale: 1.0,
                locks: None,
            }))
            .with_system(Startup, init_node_timer)
            .build()
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

    // --- A5: Timer multiplier tests ---

    fn test_app_with_node_sequence(timer_secs: f32, timer_mult: f32) -> App {
        use crate::shared::test_utils::TestAppBuilder;
        TestAppBuilder::new()
            .insert_resource(ActiveNodeLayout(NodeLayout {
                name: "timer_mult_test".to_owned(),
                timer_secs,
                cols: 2,
                rows: 1,
                grid_top_offset: 50.0,
                grid: vec![vec!["S".to_owned(), "S".to_owned()]],
                pool: NodePool::default(),
                entity_scale: 1.0,
                locks: None,
            }))
            .insert_resource(NodeOutcome {
                node_index: 0,
                ..Default::default()
            })
            .insert_resource(NodeSequence {
                assignments: vec![NodeAssignment {
                    node_type: NodeType::Active,
                    tier_index: 0,
                    timer_mult,
                }],
            })
            .with_system(Startup, init_node_timer)
            .build()
    }

    #[test]
    fn node_timer_scaled_by_node_assignment_timer_mult() {
        let mut app = test_app_with_node_sequence(60.0, 0.7);
        app.update();

        // 60.0 * 0.7 = 42.0
        let timer = app.world().resource::<NodeTimer>();
        assert!(
            (timer.remaining - 42.0).abs() < f32::EPSILON,
            "timer remaining should be 60.0 * 0.7 = 42.0, got {}",
            timer.remaining
        );
        assert!(
            (timer.total - 42.0).abs() < f32::EPSILON,
            "timer total should be 60.0 * 0.7 = 42.0, got {}",
            timer.total
        );
    }

    #[test]
    fn node_timer_unchanged_when_timer_mult_is_one() {
        let mut app = test_app_with_node_sequence(60.0, 1.0);
        app.update();

        // 60.0 * 1.0 = 60.0
        let timer = app.world().resource::<NodeTimer>();
        assert!(
            (timer.remaining - 60.0).abs() < f32::EPSILON,
            "timer remaining should be 60.0 * 1.0 = 60.0, got {}",
            timer.remaining
        );
        assert!(
            (timer.total - 60.0).abs() < f32::EPSILON,
            "timer total should be 60.0 * 1.0 = 60.0, got {}",
            timer.total
        );
    }
}
