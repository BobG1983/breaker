use bevy::prelude::*;

use super::super::rng_types::*;
use crate::{prelude::*, state::run::resources::NodeOutcome};

// ── Group F — EffectBaseSeed derived once at run start ──────────────────

mod effect_base_seed {
    use super::*;
    use crate::{
        chips::inventory::ChipInventory,
        mutators::{
            hazards::resources::ActiveHazards,
            protocols::{
                greed::GreedStacks,
                resources::{ActiveProtocols, ProtocolOffer, ProtocolOfferingCount},
                siphon::SiphonStreak,
            },
        },
        shared::RunSeed,
        state::run::{
            loading::systems::reset_run_state,
            resources::{HighlightTracker, RunStats},
        },
    };

    const EFFECT_SENTINEL: u64 = 0xDEAD_BEEF_CAFE_5678;

    fn test_app() -> App {
        TestAppBuilder::new()
            .insert_resource(NodeOutcome {
                node_index: 5,
                result: crate::state::run::resources::NodeResult::Won,
                ..default()
            })
            .with_resource::<GameRng>()
            .with_resource::<RunSeed>()
            .with_resource::<ChipInventory>()
            .with_resource::<RunStats>()
            .with_resource::<HighlightTracker>()
            .with_resource::<EffectBaseSeed>()
            .with_resource::<ChipSelectCount>()
            .with_resource::<ActiveProtocols>()
            .with_resource::<ProtocolOffer>()
            .with_resource::<ProtocolOfferingCount>()
            .with_resource::<ActiveHazards>()
            .with_resource::<GreedStacks>()
            .with_resource::<SiphonStreak>()
            .with_system(Update, reset_run_state)
            .build()
    }

    // B30 — reset_run_state sets EffectBaseSeed for seeded runs
    #[test]
    fn sets_effect_base_seed_from_derive_seed_named_for_seeded_run() {
        let mut app = test_app();
        app.insert_resource(RunSeed(Some(42)));
        app.insert_resource(EffectBaseSeed(EFFECT_SENTINEL));
        app.update();

        let actual = app.world().resource::<EffectBaseSeed>().0;
        let expected = derive_seed_named(42, "effect");
        assert_eq!(
            actual, expected,
            "reset_run_state must set EffectBaseSeed to derive_seed_named(42, \"effect\")"
        );
    }

    #[test]
    fn sets_effect_base_seed_for_seed_zero() {
        let mut app = test_app();
        app.insert_resource(RunSeed(Some(0)));
        app.insert_resource(EffectBaseSeed(EFFECT_SENTINEL));
        app.update();

        let actual = app.world().resource::<EffectBaseSeed>().0;
        let expected = derive_seed_named(0, "effect");
        assert_eq!(
            actual, expected,
            "reset_run_state must handle run_seed=0 without panic"
        );
    }

    // B31 — reset_run_state sets EffectBaseSeed for unseeded runs
    #[test]
    fn sets_effect_base_seed_nonzero_for_unseeded_run() {
        let mut app = test_app();
        // RunSeed defaults to None
        app.update();

        let actual = app.world().resource::<EffectBaseSeed>().0;
        // Cannot assert the exact value (OS entropy), but must be non-zero
        // with overwhelming probability — mirrors capture_run_seed convention.
        assert_ne!(
            actual, 0,
            "EffectBaseSeed must be non-zero after reset_run_state with RunSeed(None) \
             (OS entropy path)"
        );
    }

    // B32 — reset_run_state overwrites stale EffectBaseSeed from prior run
    #[test]
    fn overwrites_stale_effect_base_seed_from_prior_run() {
        let mut app = test_app();
        app.insert_resource(EffectBaseSeed(0xDEAD_BEEF));
        app.insert_resource(RunSeed(Some(42)));
        app.update();

        let actual = app.world().resource::<EffectBaseSeed>().0;
        let expected = derive_seed_named(42, "effect");
        assert_ne!(
            actual, 0xDEAD_BEEF,
            "reset_run_state must overwrite stale EffectBaseSeed(0xDEAD_BEEF)"
        );
        assert_eq!(
            actual, expected,
            "reset_run_state must set EffectBaseSeed to derive_seed_named(42, \"effect\"), \
             not leave the stale value"
        );
    }

    // B33 — seed_node_rng does NOT touch EffectBaseSeed
    #[test]
    fn seed_node_rng_does_not_touch_effect_base_seed() {
        let mut app = TestAppBuilder::new()
            .with_resource::<RunStats>()
            .with_resource::<NodeOutcome>()
            .with_resource::<RunSeed>()
            .with_resource::<BoltRng>()
            .with_resource::<NodeGenRng>()
            .with_resource::<EffectEventCounter>()
            .with_resource::<EffectBaseSeed>()
            .with_system(Update, seed_node_rng)
            .build();
        app.insert_resource(RunStats {
            seed: 42,
            ..default()
        });
        app.insert_resource(NodeOutcome {
            node_index: 5,
            ..default()
        });
        app.insert_resource(EffectBaseSeed(0xABCD_1234));
        app.update();

        let actual = app.world().resource::<EffectBaseSeed>().0;
        assert_eq!(
            actual, 0xABCD_1234,
            "seed_node_rng must NOT modify EffectBaseSeed"
        );
    }

    #[test]
    fn seed_node_rng_repeated_invocations_leave_effect_base_seed_unchanged() {
        let mut app = TestAppBuilder::new()
            .with_resource::<RunStats>()
            .with_resource::<NodeOutcome>()
            .with_resource::<RunSeed>()
            .with_resource::<BoltRng>()
            .with_resource::<NodeGenRng>()
            .with_resource::<EffectEventCounter>()
            .with_resource::<EffectBaseSeed>()
            .with_system(Update, seed_node_rng)
            .build();
        app.insert_resource(RunStats {
            seed: 42,
            ..default()
        });
        app.insert_resource(EffectBaseSeed(0xABCD_1234));

        for i in 0u32..5 {
            app.insert_resource(NodeOutcome {
                node_index: i,
                ..default()
            });
            app.update();
        }

        let actual = app.world().resource::<EffectBaseSeed>().0;
        assert_eq!(
            actual, 0xABCD_1234,
            "EffectBaseSeed must remain unchanged after multiple seed_node_rng invocations"
        );
    }
}
