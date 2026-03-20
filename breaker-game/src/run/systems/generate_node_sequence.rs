//! Procedural node sequence generation from difficulty curve.

use bevy::prelude::*;
use rand::{Rng, seq::SliceRandom};
use rand_chacha::ChaCha8Rng;

use crate::run::{
    difficulty::{DifficultyCurve, NodeType, TierNodeCount},
    resources::{NodeAssignment, NodeSequence},
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

#[cfg(test)]
mod tests {
    use rand::SeedableRng;

    use super::*;
    use crate::run::difficulty::{TierDefinition, TierNodeCount};

    // ── Test helpers ──────────────────────────────────────────────────

    fn make_tier(
        nodes: TierNodeCount,
        active_ratio: f32,
        hp_mult: f32,
        timer_mult: f32,
    ) -> TierDefinition {
        TierDefinition {
            nodes,
            active_ratio,
            hp_mult,
            timer_mult,
            introduced_cells: vec![],
        }
    }

    fn make_curve(
        tiers: Vec<TierDefinition>,
        boss_hp_mult: f32,
        timer_reduction_per_boss: f32,
    ) -> DifficultyCurve {
        DifficultyCurve {
            tiers,
            boss_hp_mult,
            timer_reduction_per_boss,
        }
    }

    fn rng_from_seed(seed: u64) -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(seed)
    }

    // ── 1. Determinism: same seed produces identical sequence ─────────

    #[test]
    fn same_seed_produces_identical_sequence() {
        let curve = make_curve(
            vec![
                make_tier(TierNodeCount::Fixed(3), 0.5, 1.0, 1.0),
                make_tier(TierNodeCount::Fixed(3), 0.5, 1.5, 0.9),
            ],
            3.0,
            0.1,
        );

        let mut rng1 = rng_from_seed(42);
        let mut rng2 = rng_from_seed(42);

        let seq1 = generate_node_sequence(&curve, &mut rng1);
        let seq2 = generate_node_sequence(&curve, &mut rng2);

        assert_eq!(
            seq1.assignments, seq2.assignments,
            "same seed must produce identical node sequences"
        );
        // Also verify non-empty (stub returns empty, so this forces implementation)
        assert!(
            !seq1.assignments.is_empty(),
            "sequence must not be empty for a curve with tiers"
        );
    }

    // ── 2. Different seeds produce different sequences ────────────────

    #[test]
    fn different_seeds_produce_different_sequences() {
        let curve = make_curve(
            vec![
                make_tier(TierNodeCount::Fixed(3), 0.5, 1.0, 1.0),
                make_tier(TierNodeCount::Fixed(3), 0.5, 1.5, 0.9),
            ],
            3.0,
            0.1,
        );

        let mut rng_a = rng_from_seed(42);
        let mut rng_b = rng_from_seed(99);

        let seq_a = generate_node_sequence(&curve, &mut rng_a);
        let seq_b = generate_node_sequence(&curve, &mut rng_b);

        assert_ne!(
            seq_a.assignments, seq_b.assignments,
            "different seeds should produce different sequences (shuffle differs)"
        );
    }

    // ── 3. Fixed(5) produces 5 non-boss + 1 boss per tier ────────────

    #[test]
    fn fixed_count_produces_correct_node_totals() {
        let curve = make_curve(
            vec![make_tier(TierNodeCount::Fixed(5), 0.5, 1.0, 1.0)],
            3.0,
            0.0,
        );
        let mut rng = rng_from_seed(42);

        let seq = generate_node_sequence(&curve, &mut rng);

        assert_eq!(
            seq.assignments.len(),
            6,
            "Fixed(5) + boss = 6 total assignments"
        );

        let non_boss_count = seq
            .assignments
            .iter()
            .filter(|a| a.node_type != NodeType::Boss)
            .count();
        assert_eq!(non_boss_count, 5, "should have 5 non-boss nodes");

        assert_eq!(
            seq.assignments.last().unwrap().node_type,
            NodeType::Boss,
            "last node in tier must be Boss"
        );
    }

    // ── 4. Range(4, 6) stays in bounds across 100 seeds ──────────────

    #[test]
    fn range_count_stays_within_bounds_across_seeds() {
        let curve = make_curve(
            vec![make_tier(TierNodeCount::Range(4, 6), 0.5, 1.0, 1.0)],
            3.0,
            0.0,
        );

        for seed in 0..100_u64 {
            let mut rng = rng_from_seed(seed);
            let seq = generate_node_sequence(&curve, &mut rng);
            let len = seq.assignments.len();
            assert!(
                (5..=7).contains(&len),
                "seed {seed}: expected 5..=7 assignments (4..=6 + boss), got {len}"
            );
        }
    }

    // ── 5. active_ratio 0.0 produces all Passive ─────────────────────

    #[test]
    fn active_ratio_zero_produces_all_passive() {
        let curve = make_curve(
            vec![make_tier(TierNodeCount::Fixed(5), 0.0, 1.0, 1.0)],
            3.0,
            0.0,
        );
        let mut rng = rng_from_seed(42);

        let seq = generate_node_sequence(&curve, &mut rng);

        let active_count = seq
            .assignments
            .iter()
            .filter(|a| a.node_type == NodeType::Active)
            .count();
        let passive_count = seq
            .assignments
            .iter()
            .filter(|a| a.node_type == NodeType::Passive)
            .count();
        let boss_count = seq
            .assignments
            .iter()
            .filter(|a| a.node_type == NodeType::Boss)
            .count();

        assert_eq!(active_count, 0, "active_ratio 0.0 should produce 0 Active");
        assert_eq!(passive_count, 5, "should have 5 Passive nodes");
        assert_eq!(boss_count, 1, "should have 1 Boss node");
    }

    // ── 6. active_ratio 1.0 produces all Active ──────────────────────

    #[test]
    fn active_ratio_one_produces_all_active() {
        let curve = make_curve(
            vec![make_tier(TierNodeCount::Fixed(5), 1.0, 1.0, 1.0)],
            3.0,
            0.0,
        );
        let mut rng = rng_from_seed(42);

        let seq = generate_node_sequence(&curve, &mut rng);

        let active_count = seq
            .assignments
            .iter()
            .filter(|a| a.node_type == NodeType::Active)
            .count();
        let passive_count = seq
            .assignments
            .iter()
            .filter(|a| a.node_type == NodeType::Passive)
            .count();
        let boss_count = seq
            .assignments
            .iter()
            .filter(|a| a.node_type == NodeType::Boss)
            .count();

        assert_eq!(active_count, 5, "active_ratio 1.0 should produce 5 Active");
        assert_eq!(passive_count, 0, "should have 0 Passive nodes");
        assert_eq!(boss_count, 1, "should have 1 Boss node");
    }

    // ── 7. active_ratio 0.4 with Fixed(5) produces 2 Active + 3 Passive

    #[test]
    fn active_ratio_fractional_rounds_correctly() {
        let curve = make_curve(
            vec![make_tier(TierNodeCount::Fixed(5), 0.4, 1.0, 1.0)],
            3.0,
            0.0,
        );
        let mut rng = rng_from_seed(42);

        let seq = generate_node_sequence(&curve, &mut rng);

        let active_count = seq
            .assignments
            .iter()
            .filter(|a| a.node_type == NodeType::Active)
            .count();
        let passive_count = seq
            .assignments
            .iter()
            .filter(|a| a.node_type == NodeType::Passive)
            .count();

        assert_eq!(active_count, 2, "round(5 * 0.4) = 2 Active nodes");
        assert_eq!(passive_count, 3, "5 - 2 = 3 Passive nodes");
    }

    // ── 8. Each tier ends with a Boss node ────────────────────────────

    #[test]
    fn each_tier_ends_with_boss_node() {
        let curve = make_curve(
            vec![
                make_tier(TierNodeCount::Fixed(2), 0.5, 1.0, 1.0),
                make_tier(TierNodeCount::Fixed(2), 0.5, 1.5, 0.9),
                make_tier(TierNodeCount::Fixed(2), 0.5, 2.0, 0.8),
            ],
            3.0,
            0.1,
        );
        let mut rng = rng_from_seed(42);

        let seq = generate_node_sequence(&curve, &mut rng);

        // 3 tiers * (2 nodes + 1 boss) = 9 total
        assert_eq!(seq.assignments.len(), 9, "3 tiers of 3 = 9 total");

        // Bosses at indices 2, 5, 8 (end of each tier)
        assert_eq!(
            seq.assignments[2].node_type,
            NodeType::Boss,
            "boss at index 2 (end of tier 0)"
        );
        assert_eq!(
            seq.assignments[5].node_type,
            NodeType::Boss,
            "boss at index 5 (end of tier 1)"
        );
        assert_eq!(
            seq.assignments[8].node_type,
            NodeType::Boss,
            "boss at index 8 (end of tier 2)"
        );
    }

    // ── 9. HP multipliers applied per tier ────────────────────────────

    #[test]
    fn hp_multipliers_applied_per_tier() {
        let curve = make_curve(
            vec![
                make_tier(TierNodeCount::Fixed(2), 0.5, 1.0, 1.0),
                make_tier(TierNodeCount::Fixed(2), 0.5, 2.0, 1.0),
            ],
            1.0, // boss_hp_mult = 1.0 so boss hp_mult == tier hp_mult
            0.0,
        );
        let mut rng = rng_from_seed(42);

        let seq = generate_node_sequence(&curve, &mut rng);

        // Tier 0: indices 0, 1 (non-boss) should have hp_mult 1.0
        for i in 0..2 {
            assert!(
                (seq.assignments[i].hp_mult - 1.0).abs() < f32::EPSILON,
                "tier 0 node {i} hp_mult should be 1.0, got {}",
                seq.assignments[i].hp_mult
            );
        }

        // Tier 1: indices 3, 4 (non-boss) should have hp_mult 2.0
        for i in 3..5 {
            assert!(
                (seq.assignments[i].hp_mult - 2.0).abs() < f32::EPSILON,
                "tier 1 node {i} hp_mult should be 2.0, got {}",
                seq.assignments[i].hp_mult
            );
        }
    }

    // ── 10. Boss HP = tier.hp_mult * boss_hp_mult ─────────────────────

    #[test]
    fn boss_hp_mult_is_tier_hp_mult_times_boss_hp_mult() {
        let curve = make_curve(
            vec![make_tier(TierNodeCount::Fixed(2), 0.5, 1.5, 1.0)],
            3.0,
            0.0,
        );
        let mut rng = rng_from_seed(42);

        let seq = generate_node_sequence(&curve, &mut rng);

        let boss = seq
            .assignments
            .iter()
            .find(|a| a.node_type == NodeType::Boss)
            .expect("should have a boss node");

        let expected = 1.5 * 3.0; // 4.5
        assert!(
            (boss.hp_mult - expected).abs() < f32::EPSILON,
            "boss hp_mult should be 4.5 (1.5 * 3.0), got {}",
            boss.hp_mult
        );
    }

    // ── 11. Timer reduction cumulates after each boss ─────────────────

    #[test]
    fn timer_reduction_cumulates_after_each_boss() {
        let curve = make_curve(
            vec![
                make_tier(TierNodeCount::Fixed(1), 0.0, 1.0, 1.0),
                make_tier(TierNodeCount::Fixed(1), 0.0, 1.0, 1.0),
                make_tier(TierNodeCount::Fixed(1), 0.0, 1.0, 1.0),
            ],
            1.0,
            0.1,
        );
        let mut rng = rng_from_seed(42);

        let seq = generate_node_sequence(&curve, &mut rng);

        // Tier 0 (indices 0-1): timer_mult = 1.0 (no reduction yet)
        assert!(
            (seq.assignments[0].timer_mult - 1.0).abs() < f32::EPSILON,
            "tier 0 non-boss timer_mult should be 1.0, got {}",
            seq.assignments[0].timer_mult
        );

        // Tier 1 (indices 2-3): timer_mult = 1.0 - 0.1 = 0.9
        assert!(
            (seq.assignments[2].timer_mult - 0.9).abs() < f32::EPSILON,
            "tier 1 non-boss timer_mult should be 0.9, got {}",
            seq.assignments[2].timer_mult
        );

        // Tier 2 (indices 4-5): timer_mult = 1.0 - 0.2 = 0.8
        assert!(
            (seq.assignments[4].timer_mult - 0.8).abs() < f32::EPSILON,
            "tier 2 non-boss timer_mult should be 0.8, got {}",
            seq.assignments[4].timer_mult
        );
    }

    // ── 12. Timer mult clamped to minimum 0.1 ─────────────────────────

    #[test]
    fn timer_mult_clamped_to_minimum() {
        let curve = make_curve(
            vec![
                make_tier(TierNodeCount::Fixed(1), 0.0, 1.0, 0.15),
                make_tier(TierNodeCount::Fixed(1), 0.0, 1.0, 0.15),
            ],
            1.0,
            0.1,
        );
        let mut rng = rng_from_seed(42);

        let seq = generate_node_sequence(&curve, &mut rng);

        // Tier 0: timer_mult = 0.15 (no reduction yet)
        assert!(
            (seq.assignments[0].timer_mult - 0.15).abs() < f32::EPSILON,
            "tier 0 timer_mult should be 0.15, got {}",
            seq.assignments[0].timer_mult
        );

        // Tier 1: timer_mult = 0.15 - 0.1 = 0.05, clamped to 0.1
        assert!(
            (seq.assignments[2].timer_mult - 0.1).abs() < f32::EPSILON,
            "tier 1 timer_mult should be clamped to 0.1, got {}",
            seq.assignments[2].timer_mult
        );
    }

    // ── 13. Empty tiers produces empty sequence ───────────────────────

    #[test]
    fn empty_tiers_produces_empty_sequence() {
        let curve = make_curve(vec![], 3.0, 0.1);
        let mut rng = rng_from_seed(42);

        let seq = generate_node_sequence(&curve, &mut rng);

        assert!(
            seq.assignments.is_empty(),
            "no tiers should produce empty sequence"
        );
    }

    // ── 14. Fixed(0) produces only Boss node ──────────────────────────

    #[test]
    fn fixed_zero_produces_only_boss_node() {
        let curve = make_curve(
            vec![make_tier(TierNodeCount::Fixed(0), 0.5, 1.0, 1.0)],
            3.0,
            0.0,
        );
        let mut rng = rng_from_seed(42);

        let seq = generate_node_sequence(&curve, &mut rng);

        assert_eq!(
            seq.assignments.len(),
            1,
            "Fixed(0) should produce 1 assignment (boss only)"
        );
        assert_eq!(
            seq.assignments[0].node_type,
            NodeType::Boss,
            "the single assignment must be a Boss"
        );
    }

    // ── 15. tier_index correctly assigned ─────────────────────────────

    #[test]
    fn tier_index_correctly_assigned() {
        let curve = make_curve(
            vec![
                make_tier(TierNodeCount::Fixed(2), 0.5, 1.0, 1.0),
                make_tier(TierNodeCount::Fixed(2), 0.5, 1.5, 0.9),
                make_tier(TierNodeCount::Fixed(2), 0.5, 2.0, 0.8),
            ],
            3.0,
            0.1,
        );
        let mut rng = rng_from_seed(42);

        let seq = generate_node_sequence(&curve, &mut rng);

        // Tier 0: indices 0, 1, 2 (2 non-boss + 1 boss)
        for i in 0..3 {
            assert_eq!(
                seq.assignments[i].tier_index, 0,
                "assignment {i} should be tier_index 0, got {}",
                seq.assignments[i].tier_index
            );
        }

        // Tier 1: indices 3, 4, 5
        for i in 3..6 {
            assert_eq!(
                seq.assignments[i].tier_index, 1,
                "assignment {i} should be tier_index 1, got {}",
                seq.assignments[i].tier_index
            );
        }

        // Tier 2: indices 6, 7, 8
        for i in 6..9 {
            assert_eq!(
                seq.assignments[i].tier_index, 2,
                "assignment {i} should be tier_index 2, got {}",
                seq.assignments[i].tier_index
            );
        }
    }
}
