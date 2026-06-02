use rand::Rng;

use super::super::rng_types::*;

// ── Group G — Cross-channel distinctness sanity ─────────────────────────

mod channel_distinctness {
    use super::*;

    // B34 — All seven channels produce distinct first-draw seeds for run_seed=42, node_index=0.
    #[test]
    fn seven_rng_channel_seeds_are_pairwise_distinct_for_standard_inputs() {
        let run_seed = 42u64;
        let node_index = 0u64;

        let node_sequence_seed = derive_seed_named(run_seed, "node_sequence");
        let node_gen_seed = derive_seed_named(derive_seed(run_seed, node_index), "node_gen");
        let bolt_seed = derive_seed_named(derive_seed(run_seed, node_index), "bolt");
        let chip_seed = derive_seed_named(derive_seed(run_seed, node_index), "chip");
        let protocol_seed = derive_seed_named(derive_seed(run_seed, node_index), "protocol");
        let hazard_seed = derive_seed_named(derive_seed(run_seed, node_index), "hazard");
        let effect_seed = derive_seed_named(run_seed, "effect");

        let seeds = [
            ("node_sequence", node_sequence_seed),
            ("node_gen", node_gen_seed),
            ("bolt", bolt_seed),
            ("chip", chip_seed),
            ("protocol", protocol_seed),
            ("hazard", hazard_seed),
            ("effect", effect_seed),
        ];

        for i in 0..seeds.len() {
            for j in (i + 1)..seeds.len() {
                assert_ne!(
                    seeds[i].1, seeds[j].1,
                    "channel \"{}\" seed must differ from channel \"{}\" seed \
                     (run_seed=42, node_index=0)",
                    seeds[i].0, seeds[j].0
                );
            }
        }
    }

    #[test]
    fn seven_rng_channel_seeds_are_pairwise_distinct_for_zero_run_seed() {
        let run_seed = 0u64;
        let node_index = 0u64;

        let node_sequence_seed = derive_seed_named(run_seed, "node_sequence");
        let node_gen_seed = derive_seed_named(derive_seed(run_seed, node_index), "node_gen");
        let bolt_seed = derive_seed_named(derive_seed(run_seed, node_index), "bolt");
        let chip_seed = derive_seed_named(derive_seed(run_seed, node_index), "chip");
        let protocol_seed = derive_seed_named(derive_seed(run_seed, node_index), "protocol");
        let hazard_seed = derive_seed_named(derive_seed(run_seed, node_index), "hazard");
        let effect_seed = derive_seed_named(run_seed, "effect");

        let seeds = [
            ("node_sequence", node_sequence_seed),
            ("node_gen", node_gen_seed),
            ("bolt", bolt_seed),
            ("chip", chip_seed),
            ("protocol", protocol_seed),
            ("hazard", hazard_seed),
            ("effect", effect_seed),
        ];

        for i in 0..seeds.len() {
            for j in (i + 1)..seeds.len() {
                assert_ne!(
                    seeds[i].1, seeds[j].1,
                    "channel \"{}\" seed must differ from channel \"{}\" seed \
                     (run_seed=0, node_index=0)",
                    seeds[i].0, seeds[j].0
                );
            }
        }
    }
}

// ── Existing test (preserved) ─────────────────────────────────────────

#[test]
fn game_rng_from_seed_is_deterministic() {
    let mut rng1 = GameRng::from_seed(42);
    let mut rng2 = GameRng::from_seed(42);
    let v1: f32 = rng1.0.random();
    let v2: f32 = rng2.0.random();
    assert!((v1 - v2).abs() < f32::EPSILON);
}
