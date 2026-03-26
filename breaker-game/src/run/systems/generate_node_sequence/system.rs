use bevy::prelude::*;
use rand::{Rng, seq::SliceRandom};
use rand_chacha::ChaCha8Rng;

use crate::run::{
    definition::{NodeType, TierNodeCount},
    resources::{DifficultyCurve, NodeAssignment, NodeSequence},
};

/// Generates a deterministic node sequence from the difficulty curve.
///
/// Uses the provided `ChaCha8Rng` for deterministic randomness (range
/// resolution and shuffle). The caller is responsible for seeding the RNG.
#[must_use]
pub(super) fn generate_node_sequence(
    curve: &DifficultyCurve,
    rng: &mut ChaCha8Rng,
) -> NodeSequence {
    let mut assignments = Vec::new();
    let mut cumulative_timer_reduction: f32 = 0.0;

    for (tier_index, tier) in (0_u32..).zip(curve.tiers.iter()) {
        // 1. Resolve node count
        let count = match tier.nodes {
            TierNodeCount::Fixed(n) => n,
            TierNodeCount::Range(min, max) => rng.random_range(min..=max),
        };

        // 2. Compute active threshold — kept as f32 to avoid float→int cast lints.
        //    active_ratio is 0.0..=1.0; threshold marks the boundary between Active and Passive.
        let active_threshold =
            (f32::from(u16::try_from(count).unwrap_or(u16::MAX)) * tier.active_ratio).round();

        // 3. Create non-boss assignments
        let mut tier_nodes: Vec<NodeAssignment> = (0..count)
            .map(|i| {
                let node_type =
                    if f32::from(u16::try_from(i).unwrap_or(u16::MAX)) < active_threshold {
                        NodeType::Active
                    } else {
                        NodeType::Passive
                    };
                NodeAssignment {
                    node_type,
                    tier_index,
                    hp_mult: 0.0,    // set below
                    timer_mult: 0.0, // set below
                }
            })
            .collect();

        // 4. Shuffle the non-boss assignments
        tier_nodes.shuffle(rng);

        // 5. Compute timer_mult with cumulative boss reduction
        let timer_mult = (tier.timer_mult - cumulative_timer_reduction).max(0.1);

        // 6-7. Set hp_mult, timer_mult, and tier_index on all non-boss assignments
        for node in &mut tier_nodes {
            node.hp_mult = tier.hp_mult;
            node.timer_mult = timer_mult;
        }

        // 8. Append non-boss nodes to the main assignments vec
        assignments.append(&mut tier_nodes);

        // 9. Append Boss node
        assignments.push(NodeAssignment {
            node_type: NodeType::Boss,
            tier_index,
            hp_mult: tier.hp_mult * curve.boss_hp_mult,
            timer_mult,
        });

        // 10. Advance cumulative timer reduction
        cumulative_timer_reduction += curve.timer_reduction_per_boss;
    }

    NodeSequence { assignments }
}

/// ECS system wrapper — generates the node sequence at run start.
///
/// Runs on `OnExit(GameState::MainMenu)`, after `reset_run_state` reseeds the RNG.
pub(crate) fn generate_node_sequence_system(
    curve: Res<DifficultyCurve>,
    mut rng: ResMut<crate::shared::GameRng>,
    mut commands: Commands,
) {
    let sequence = generate_node_sequence(&curve, &mut rng.0);
    commands.insert_resource(sequence);
}
