//! System to advance to the next node during a node transition.

use bevy::prelude::*;

use crate::state::run::{
    definition::NodeType,
    resources::{NodeOutcome, NodeSequence},
};

/// Increments the node index and resets the transition flag.
///
/// Runs on `OnEnter(RunState::Node)` — called when entering a new node
/// after chip select.
pub(crate) fn advance_node(mut outcome: ResMut<NodeOutcome>, sequence: Option<Res<NodeSequence>>) {
    if let Some(sequence) = sequence {
        let completed_node_type = sequence
            .assignments
            .get(outcome.node_index as usize)
            .map(|a| a.node_type);

        if completed_node_type == Some(NodeType::Boss) {
            outcome.tier += 1;
            outcome.position_in_tier = 0;
        } else if completed_node_type.is_some() {
            outcome.position_in_tier += 1;
        }
        // If past end of sequence, tier and position remain unchanged
    }

    outcome.node_index += 1;
    outcome.cleared_this_frame = false;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::run::{
        definition::NodeType,
        resources::{NodeAssignment, NodeSequence},
    };

    fn test_app() -> App {
        use crate::shared::test_utils::TestAppBuilder;
        TestAppBuilder::new()
            .insert_resource(NodeOutcome {
                node_index: 0,
                cleared_this_frame: true,
                ..default()
            })
            .with_system(Update, advance_node)
            .build()
    }

    #[test]
    fn increments_node_index() {
        let mut app = test_app();
        app.update();

        let run_state = app.world().resource::<NodeOutcome>();
        assert_eq!(run_state.node_index, 1);
    }

    #[test]
    fn resets_cleared_this_frame() {
        let mut app = test_app();
        app.update();

        let run_state = app.world().resource::<NodeOutcome>();
        assert!(
            !run_state.cleared_this_frame,
            "cleared_this_frame should be reset for the next node"
        );
    }

    // ── Part I: Toughness + HP Scaling — tier and position_in_tier tracking ──

    fn test_app_with_sequence(
        node_index: u32,
        tier: u32,
        position_in_tier: u32,
        cleared_this_frame: bool,
        assignments: Vec<NodeAssignment>,
    ) -> App {
        use crate::shared::test_utils::TestAppBuilder;
        TestAppBuilder::new()
            .insert_resource(NodeOutcome {
                node_index,
                cleared_this_frame,
                tier,
                position_in_tier,
                ..default()
            })
            .insert_resource(NodeSequence { assignments })
            .with_system(Update, advance_node)
            .build()
    }

    // Behavior 22: advance_node updates tier and position_in_tier based on previous node's type
    #[test]
    fn advance_from_active_node_increments_position_in_tier() {
        let mut app = test_app_with_sequence(
            0,
            0,
            0,
            true,
            vec![
                NodeAssignment {
                    node_type:  NodeType::Active,
                    tier_index: 0,
                    timer_mult: 1.0,
                },
                NodeAssignment {
                    node_type:  NodeType::Active,
                    tier_index: 0,
                    timer_mult: 1.0,
                },
                NodeAssignment {
                    node_type:  NodeType::Boss,
                    tier_index: 0,
                    timer_mult: 1.0,
                },
            ],
        );
        app.update();

        let outcome = app.world().resource::<NodeOutcome>();
        assert_eq!(outcome.node_index, 1, "node_index should advance to 1");
        assert_eq!(outcome.tier, 0, "tier should remain 0");
        assert_eq!(
            outcome.position_in_tier, 1,
            "position_in_tier should increment to 1"
        );
    }

    // Behavior 23: advance_node resets position_in_tier on tier change (boss clear)
    #[test]
    fn advance_from_boss_node_increments_tier_and_resets_position() {
        let mut app = test_app_with_sequence(
            2,
            0,
            2,
            true,
            vec![
                NodeAssignment {
                    node_type:  NodeType::Active,
                    tier_index: 0,
                    timer_mult: 1.0,
                },
                NodeAssignment {
                    node_type:  NodeType::Active,
                    tier_index: 0,
                    timer_mult: 1.0,
                },
                NodeAssignment {
                    node_type:  NodeType::Boss,
                    tier_index: 0,
                    timer_mult: 1.0,
                },
                NodeAssignment {
                    node_type:  NodeType::Active,
                    tier_index: 1,
                    timer_mult: 0.9,
                },
            ],
        );
        app.update();

        let outcome = app.world().resource::<NodeOutcome>();
        assert_eq!(outcome.node_index, 3, "node_index should advance to 3");
        assert_eq!(outcome.tier, 1, "tier should advance to 1 after boss clear");
        assert_eq!(
            outcome.position_in_tier, 0,
            "position_in_tier should reset to 0 in new tier"
        );
    }

    // Behavior 24: advance_node increments tier when previous node was Boss
    #[test]
    fn advance_from_boss_increments_tier_by_one() {
        let mut app = test_app_with_sequence(
            2,
            0,
            2,
            true,
            vec![
                NodeAssignment {
                    node_type:  NodeType::Active,
                    tier_index: 0,
                    timer_mult: 1.0,
                },
                NodeAssignment {
                    node_type:  NodeType::Active,
                    tier_index: 0,
                    timer_mult: 1.0,
                },
                NodeAssignment {
                    node_type:  NodeType::Boss,
                    tier_index: 0,
                    timer_mult: 1.0,
                },
            ],
        );
        app.update();

        let outcome = app.world().resource::<NodeOutcome>();
        assert_eq!(
            outcome.tier, 1,
            "tier should increment by exactly 1 after boss clear"
        );
    }

    // Behavior 25: advance_node increments position_in_tier when staying in same tier (Passive)
    #[test]
    fn advance_from_passive_node_increments_position_in_tier() {
        let mut app = test_app_with_sequence(
            1,
            0,
            1,
            true,
            vec![
                NodeAssignment {
                    node_type:  NodeType::Passive,
                    tier_index: 0,
                    timer_mult: 1.0,
                },
                NodeAssignment {
                    node_type:  NodeType::Passive,
                    tier_index: 0,
                    timer_mult: 1.0,
                },
                NodeAssignment {
                    node_type:  NodeType::Boss,
                    tier_index: 0,
                    timer_mult: 1.0,
                },
            ],
        );
        app.update();

        let outcome = app.world().resource::<NodeOutcome>();
        assert_eq!(outcome.node_index, 2, "node_index should advance to 2");
        assert_eq!(outcome.tier, 0, "tier should remain 0");
        assert_eq!(
            outcome.position_in_tier, 2,
            "position_in_tier should increment to 2"
        );
    }

    // Behavior 26: advance_node still resets cleared_this_frame
    #[test]
    fn advance_with_sequence_resets_cleared_this_frame() {
        let mut app = test_app_with_sequence(
            0,
            0,
            0,
            true,
            vec![NodeAssignment {
                node_type:  NodeType::Active,
                tier_index: 0,
                timer_mult: 1.0,
            }],
        );
        app.update();

        let outcome = app.world().resource::<NodeOutcome>();
        assert!(
            !outcome.cleared_this_frame,
            "cleared_this_frame should be reset"
        );
    }

    // Behavior 26 edge case: already false stays false
    #[test]
    fn advance_with_transition_already_false_stays_false() {
        let mut app = test_app_with_sequence(
            0,
            0,
            0,
            false,
            vec![NodeAssignment {
                node_type:  NodeType::Active,
                tier_index: 0,
                timer_mult: 1.0,
            }],
        );
        app.update();

        let outcome = app.world().resource::<NodeOutcome>();
        assert!(
            !outcome.cleared_this_frame,
            "cleared_this_frame should remain false"
        );
    }

    // Behavior 27: advance_node gracefully handles missing NodeSequence resource
    #[test]
    fn advance_without_node_sequence_only_increments_index() {
        use crate::shared::test_utils::TestAppBuilder;
        let mut app = TestAppBuilder::new()
            .insert_resource(NodeOutcome {
                node_index: 0,
                tier: 0,
                position_in_tier: 0,
                cleared_this_frame: true,
                ..default()
            })
            .with_system(Update, advance_node)
            .build();
        app.update();

        let outcome = app.world().resource::<NodeOutcome>();
        assert_eq!(outcome.node_index, 1, "node_index should advance to 1");
        assert_eq!(outcome.tier, 0, "tier should remain 0 without sequence");
        assert_eq!(
            outcome.position_in_tier, 0,
            "position_in_tier should remain 0 without sequence"
        );
    }

    // Behavior 28: advance_node past end of sequence holds tier and position_in_tier
    #[test]
    fn advance_past_end_of_sequence_holds_tier_and_position() {
        let mut app = test_app_with_sequence(
            5,
            2,
            1,
            true,
            vec![
                NodeAssignment {
                    node_type:  NodeType::Active,
                    tier_index: 0,
                    timer_mult: 1.0,
                },
                NodeAssignment {
                    node_type:  NodeType::Active,
                    tier_index: 0,
                    timer_mult: 1.0,
                },
                NodeAssignment {
                    node_type:  NodeType::Boss,
                    tier_index: 0,
                    timer_mult: 1.0,
                },
                NodeAssignment {
                    node_type:  NodeType::Active,
                    tier_index: 1,
                    timer_mult: 0.9,
                },
                NodeAssignment {
                    node_type:  NodeType::Boss,
                    tier_index: 1,
                    timer_mult: 0.9,
                },
            ],
        );
        app.update();

        let outcome = app.world().resource::<NodeOutcome>();
        assert_eq!(outcome.node_index, 6, "node_index should advance to 6");
        assert_eq!(
            outcome.tier, 2,
            "tier should remain 2 (past end of sequence)"
        );
        assert_eq!(
            outcome.position_in_tier, 1,
            "position_in_tier should remain 1 (past end of sequence)"
        );
    }

    // Behavior 28 edge case: multiple advances past end
    #[test]
    fn multiple_advances_past_end_hold_tier_and_position() {
        let mut app = test_app_with_sequence(
            3,
            1,
            0,
            true,
            vec![
                NodeAssignment {
                    node_type:  NodeType::Active,
                    tier_index: 0,
                    timer_mult: 1.0,
                },
                NodeAssignment {
                    node_type:  NodeType::Boss,
                    tier_index: 0,
                    timer_mult: 1.0,
                },
                NodeAssignment {
                    node_type:  NodeType::Active,
                    tier_index: 1,
                    timer_mult: 0.9,
                },
            ],
        );
        app.update();

        let outcome = app.world().resource::<NodeOutcome>();
        assert_eq!(outcome.node_index, 4);
        assert_eq!(outcome.tier, 1, "tier should remain unchanged past end");
        assert_eq!(
            outcome.position_in_tier, 0,
            "position_in_tier should remain unchanged past end"
        );
    }
}
