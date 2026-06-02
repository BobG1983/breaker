use bevy::prelude::*;
use rand::Rng;

use super::super::rng_types::*;
use crate::{
    prelude::*,
    shared::RunSeed,
    state::run::resources::{NodeOutcome, RunStats},
};

// ── Group D — seed_node_rng resource-insertion correctness ─────────────

mod seed_node_rng_resources {
    use super::*;

    // Sentinel seeds differ from from_seed(0) so that: when derive_seed/derive_seed_named
    // stub returns 0, expected draws come from from_seed(0) but actual world resources
    // are seeded from SENTINEL, causing the assert_eq to fail at RED.
    const SENTINEL: u64 = 0xDEAD_BEEF_CAFE_1234;

    fn test_app() -> App {
        TestAppBuilder::new()
            .with_resource::<RunStats>()
            .with_resource::<NodeOutcome>()
            .with_resource::<RunSeed>()
            .with_resource::<BoltRng>()
            .with_resource::<NodeGenRng>()
            .with_resource::<EffectEventCounter>()
            .with_system(Update, seed_node_rng)
            .build()
    }

    // B19 — BoltRng seeded from correct formula
    #[test]
    fn bolt_rng_seeded_from_derive_seed_named_bolt() {
        let mut app = test_app();
        app.insert_resource(RunStats {
            seed: 42,
            ..default()
        });
        app.insert_resource(NodeOutcome {
            node_index: 0,
            ..default()
        });
        // Sentinel pre-seed: actual draws from SENTINEL; expected from formula stub (0).
        // Guarantees test fails against no-op stub at RED.
        app.insert_resource(BoltRng::from_seed(SENTINEL));
        app.insert_resource(NodeGenRng::from_seed(SENTINEL));
        app.update();

        let expected_seed = derive_seed_named(derive_seed(42, 0u64), "bolt");
        let mut expected_rng = BoltRng::from_seed(expected_seed);
        let expected_draw: u64 = expected_rng.0.random();

        let actual_draw: u64 = app.world_mut().resource_mut::<BoltRng>().0.random();
        assert_eq!(
            actual_draw, expected_draw,
            "BoltRng first draw must match derive_seed_named(derive_seed(42,0),\"bolt\")"
        );
    }

    #[test]
    fn bolt_rng_node_zero_and_one_produce_different_seeds() {
        let draw_node = |node_index: u32| -> u64 {
            let mut app = test_app();
            app.insert_resource(RunStats {
                seed: 42,
                ..default()
            });
            app.insert_resource(NodeOutcome {
                node_index,
                ..default()
            });
            app.insert_resource(BoltRng::from_seed(SENTINEL));
            app.insert_resource(NodeGenRng::from_seed(SENTINEL));
            app.update();
            app.world_mut().resource_mut::<BoltRng>().0.random()
        };
        assert_ne!(
            draw_node(0),
            draw_node(1),
            "BoltRng first draws must differ between node_index=0 and node_index=1"
        );
    }

    // B20 — BoltRng reseeds existing resource
    #[test]
    fn bolt_rng_overwrites_existing_resource() {
        let mut app = test_app();
        app.insert_resource(RunStats {
            seed: 42,
            ..default()
        });
        app.insert_resource(NodeOutcome {
            node_index: 3,
            ..default()
        });
        app.insert_resource(BoltRng::from_seed(SENTINEL));
        app.insert_resource(NodeGenRng::from_seed(SENTINEL));
        app.update();

        let expected_seed = derive_seed_named(derive_seed(42, 3u64), "bolt");
        let mut expected_rng = BoltRng::from_seed(expected_seed);
        let expected_draw: u64 = expected_rng.0.random();

        let actual_draw: u64 = app.world_mut().resource_mut::<BoltRng>().0.random();
        assert_eq!(
            actual_draw, expected_draw,
            "seed_node_rng must overwrite existing BoltRng, not skip it"
        );
    }

    #[test]
    fn bolt_rng_reseed_is_idempotent_for_same_inputs() {
        let mut app = test_app();
        app.insert_resource(RunStats {
            seed: 42,
            ..default()
        });
        app.insert_resource(NodeOutcome {
            node_index: 3,
            ..default()
        });
        app.insert_resource(BoltRng::from_seed(SENTINEL));
        app.insert_resource(NodeGenRng::from_seed(SENTINEL));
        app.update();
        let draw1: u64 = app.world_mut().resource_mut::<BoltRng>().0.random();

        let mut app2 = test_app();
        app2.insert_resource(RunStats {
            seed: 42,
            ..default()
        });
        app2.insert_resource(NodeOutcome {
            node_index: 3,
            ..default()
        });
        app2.insert_resource(BoltRng::from_seed(SENTINEL));
        app2.insert_resource(NodeGenRng::from_seed(SENTINEL));
        app2.update();
        let draw2: u64 = app2.world_mut().resource_mut::<BoltRng>().0.random();

        assert_eq!(draw1, draw2, "seed_node_rng is idempotent for same inputs");
    }

    // B21 — NodeGenRng seeded from correct formula
    #[test]
    fn node_gen_rng_seeded_from_derive_seed_named_node_gen() {
        let mut app = test_app();
        app.insert_resource(RunStats {
            seed: 42,
            ..default()
        });
        app.insert_resource(NodeOutcome {
            node_index: 0,
            ..default()
        });
        app.insert_resource(BoltRng::from_seed(SENTINEL));
        app.insert_resource(NodeGenRng::from_seed(SENTINEL));
        app.update();

        let expected_seed = derive_seed_named(derive_seed(42, 0u64), "node_gen");
        let mut expected_rng = NodeGenRng::from_seed(expected_seed);
        let expected_draw: u64 = expected_rng.0.random();

        let actual_draw: u64 = app.world_mut().resource_mut::<NodeGenRng>().0.random();
        assert_eq!(
            actual_draw, expected_draw,
            "NodeGenRng first draw must match derive_seed_named(derive_seed(42,0),\"node_gen\")"
        );
    }

    #[test]
    fn node_gen_rng_differs_from_bolt_rng_for_same_inputs() {
        let mut app = test_app();
        app.insert_resource(RunStats {
            seed: 42,
            ..default()
        });
        app.insert_resource(NodeOutcome {
            node_index: 0,
            ..default()
        });
        app.insert_resource(BoltRng::from_seed(SENTINEL));
        app.insert_resource(NodeGenRng::from_seed(SENTINEL));
        app.update();

        let bolt_draw: u64 = app.world_mut().resource_mut::<BoltRng>().0.random();
        let node_gen_draw: u64 = app.world_mut().resource_mut::<NodeGenRng>().0.random();
        assert_ne!(
            bolt_draw, node_gen_draw,
            "BoltRng and NodeGenRng must have different first draws (different channel names)"
        );
    }

    // B22 — NodeGenRng reseeds existing resource
    #[test]
    fn node_gen_rng_overwrites_existing_resource() {
        let mut app = test_app();
        app.insert_resource(RunStats {
            seed: 42,
            ..default()
        });
        app.insert_resource(NodeOutcome {
            node_index: 4,
            ..default()
        });
        app.insert_resource(BoltRng::from_seed(SENTINEL));
        app.insert_resource(NodeGenRng::from_seed(SENTINEL));
        app.update();

        let expected_seed = derive_seed_named(derive_seed(42, 4u64), "node_gen");
        let mut expected_rng = NodeGenRng::from_seed(expected_seed);
        let expected_draw: u64 = expected_rng.0.random();

        let actual_draw: u64 = app.world_mut().resource_mut::<NodeGenRng>().0.random();
        assert_eq!(
            actual_draw, expected_draw,
            "seed_node_rng must overwrite existing NodeGenRng"
        );
    }

    // B23 — EffectEventCounter reset to 0
    #[test]
    fn effect_event_counter_reset_to_zero() {
        let mut app = test_app();
        app.insert_resource(RunStats {
            seed: 42,
            ..default()
        });
        app.insert_resource(NodeOutcome {
            node_index: 0,
            ..default()
        });
        app.insert_resource(EffectEventCounter(99));
        app.update();

        let counter = app.world().resource::<EffectEventCounter>();
        assert_eq!(
            counter.0, 0,
            "seed_node_rng must reset EffectEventCounter to 0"
        );
    }

    #[test]
    fn effect_event_counter_inserted_as_zero_when_absent() {
        let mut app = TestAppBuilder::new()
            .with_resource::<RunStats>()
            .with_resource::<NodeOutcome>()
            .with_resource::<RunSeed>()
            .with_resource::<BoltRng>()
            .with_resource::<NodeGenRng>()
            // NOTE: NOT inserting EffectEventCounter — tests insertion from absent state
            .with_system(Update, seed_node_rng)
            .build();
        app.insert_resource(RunStats {
            seed: 42,
            ..default()
        });
        app.insert_resource(NodeOutcome {
            node_index: 0,
            ..default()
        });
        app.update();

        let counter = app.world().resource::<EffectEventCounter>();
        assert_eq!(
            counter.0, 0,
            "seed_node_rng must insert EffectEventCounter(0) when absent"
        );
    }

    // B24 — Stability across independent apps
    #[test]
    fn bolt_rng_and_node_gen_rng_stable_across_independent_apps() {
        let run_seed = 1234u64;
        let node_index = 7u32;

        let (bolt_draw_a, node_gen_draw_a) = {
            let mut app = test_app();
            app.insert_resource(RunStats {
                seed: run_seed,
                ..default()
            });
            app.insert_resource(NodeOutcome {
                node_index,
                ..default()
            });
            app.insert_resource(BoltRng::from_seed(SENTINEL));
            app.insert_resource(NodeGenRng::from_seed(SENTINEL));
            app.update();
            let b: u64 = app.world_mut().resource_mut::<BoltRng>().0.random();
            let n: u64 = app.world_mut().resource_mut::<NodeGenRng>().0.random();
            (b, n)
        };

        let (bolt_draw_b, node_gen_draw_b) = {
            let mut app = test_app();
            app.insert_resource(RunStats {
                seed: run_seed,
                ..default()
            });
            app.insert_resource(NodeOutcome {
                node_index,
                ..default()
            });
            app.insert_resource(BoltRng::from_seed(SENTINEL));
            app.insert_resource(NodeGenRng::from_seed(SENTINEL));
            app.update();
            let b: u64 = app.world_mut().resource_mut::<BoltRng>().0.random();
            let n: u64 = app.world_mut().resource_mut::<NodeGenRng>().0.random();
            (b, n)
        };

        assert_eq!(
            bolt_draw_a, bolt_draw_b,
            "BoltRng first draw must be identical across independent apps with same seed"
        );
        assert_eq!(
            node_gen_draw_a, node_gen_draw_b,
            "NodeGenRng first draw must be identical across independent apps with same seed"
        );
    }

    #[test]
    fn zero_run_seed_zero_node_index_is_stable() {
        let draw_bolt = || {
            let mut app = test_app();
            app.insert_resource(RunStats {
                seed: 0,
                ..default()
            });
            app.insert_resource(NodeOutcome {
                node_index: 0,
                ..default()
            });
            app.insert_resource(BoltRng::from_seed(SENTINEL));
            app.insert_resource(NodeGenRng::from_seed(SENTINEL));
            app.update();
            app.world_mut().resource_mut::<BoltRng>().0.random::<u64>()
        };
        assert_eq!(
            draw_bolt(),
            draw_bolt(),
            "run_seed=0, node_index=0 must be stable"
        );
    }

    // B25 — Reads RunStats.seed, NOT RunSeed
    #[test]
    fn uses_run_stats_seed_not_run_seed() {
        let mut app = test_app();
        app.insert_resource(RunStats {
            seed: 42,
            ..default()
        });
        app.insert_resource(RunSeed(Some(999))); // intentionally mismatched
        app.insert_resource(NodeOutcome {
            node_index: 0,
            ..default()
        });
        app.insert_resource(BoltRng::from_seed(SENTINEL));
        app.insert_resource(NodeGenRng::from_seed(SENTINEL));
        app.update();

        // Must use 42 (RunStats.seed), NOT 999 (RunSeed)
        let expected_seed = derive_seed_named(derive_seed(42, 0u64), "bolt");
        let mut expected_rng = BoltRng::from_seed(expected_seed);
        let expected_draw: u64 = expected_rng.0.random();

        let actual_draw: u64 = app.world_mut().resource_mut::<BoltRng>().0.random();
        assert_eq!(
            actual_draw, expected_draw,
            "seed_node_rng must read RunStats.seed (42), not RunSeed (999)"
        );
    }

    #[test]
    fn uses_run_stats_seed_when_run_seed_is_none() {
        let mut app = test_app();
        app.insert_resource(RunStats {
            seed: 42,
            ..default()
        });
        app.insert_resource(RunSeed(None));
        app.insert_resource(NodeOutcome {
            node_index: 0,
            ..default()
        });
        app.insert_resource(BoltRng::from_seed(SENTINEL));
        app.insert_resource(NodeGenRng::from_seed(SENTINEL));
        app.update();

        let expected_seed = derive_seed_named(derive_seed(42, 0u64), "bolt");
        let mut expected_rng = BoltRng::from_seed(expected_seed);
        let expected_draw: u64 = expected_rng.0.random();

        let actual_draw: u64 = app.world_mut().resource_mut::<BoltRng>().0.random();
        assert_eq!(
            actual_draw, expected_draw,
            "seed_node_rng must use RunStats.seed even when RunSeed is None"
        );
    }
}
