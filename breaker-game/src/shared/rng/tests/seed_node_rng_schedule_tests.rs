use bevy::prelude::*;
use rand::Rng;

use super::super::rng_types::*;
use crate::{
    prelude::*,
    shared::{RunSeed, test_utils::schedule_inspect::system_in_set},
    state::{
        run::{
            loading::systems::capture_run_seed,
            resources::{NodeOutcome, RunStats},
            systems::setup_run,
        },
        types::*,
    },
};

// ── Group E — seed_node_rng scheduling ─────────────────────────────────

mod seed_node_rng_schedule {
    use rantzsoft_stateflow::RoutingTable;

    use super::*;
    use crate::state::run::{RunPlugin, node::systems::reset_bolt};

    const SENTINEL: u64 = 0xDEAD_BEEF_CAFE_1234;

    fn schedule_app() -> App {
        TestAppBuilder::new()
            .with_state_hierarchy()
            .with_resource::<RunStats>()
            .with_resource::<NodeOutcome>()
            .with_resource::<RunSeed>()
            .with_resource::<BoltRng>()
            .with_resource::<NodeGenRng>()
            .with_resource::<EffectEventCounter>()
            .build()
    }

    // B26 — seed_node_rng fires in OnEnter(NodeState::Loading)
    #[test]
    fn seed_node_rng_fires_on_enter_node_loading() {
        let mut app = schedule_app();
        app.insert_resource(RunStats {
            seed: 42,
            ..default()
        });
        app.insert_resource(NodeOutcome {
            node_index: 0,
            ..default()
        });
        // Sentinel pre-seeds so actual draw from no-op stub (SENTINEL) differs from
        // expected draw from formula stub returning 0 (from_seed(0)).
        app.insert_resource(BoltRng::from_seed(SENTINEL));
        app.insert_resource(NodeGenRng::from_seed(SENTINEL));
        app.add_systems(OnEnter(NodeState::Loading), seed_node_rng);

        // Drive into NodeState::Loading via the state hierarchy.
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Game);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Run);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<RunState>>()
            .set(RunState::Node);
        app.update();
        // NodeState::Loading is default on entry; fires OnEnter here.
        app.world_mut()
            .resource_mut::<NextState<NodeState>>()
            .set(NodeState::Loading);
        app.update();

        assert!(
            app.world().contains_resource::<BoltRng>(),
            "BoltRng must exist after OnEnter(NodeState::Loading)"
        );
        assert!(
            app.world().contains_resource::<NodeGenRng>(),
            "NodeGenRng must exist after OnEnter(NodeState::Loading)"
        );

        let expected_bolt_seed = derive_seed_named(derive_seed(42, 0u64), "bolt");
        let mut expected_bolt = BoltRng::from_seed(expected_bolt_seed);
        let expected_bolt_draw: u64 = expected_bolt.0.random();

        let actual_bolt_draw: u64 = app.world_mut().resource_mut::<BoltRng>().0.random();
        assert_eq!(
            actual_bolt_draw, expected_bolt_draw,
            "BoltRng after OnEnter(NodeState::Loading) must match formula for node_index=0"
        );
    }

    #[test]
    fn seed_node_rng_reseeds_on_second_entry_to_loading() {
        let mut app = schedule_app();
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
        app.add_systems(OnEnter(NodeState::Loading), seed_node_rng);

        // Enter Loading the first time.
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Game);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Run);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<RunState>>()
            .set(RunState::Node);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<NodeState>>()
            .set(NodeState::Loading);
        app.update();

        // Transition out to a different state, then back to Loading with
        // node_index=1 to exercise the reseed path.
        app.world_mut()
            .resource_mut::<NextState<NodeState>>()
            .set(NodeState::Playing);
        app.update();
        app.insert_resource(NodeOutcome {
            node_index: 1,
            ..default()
        });
        app.world_mut()
            .resource_mut::<NextState<NodeState>>()
            .set(NodeState::Loading);
        app.update();

        let expected_bolt_seed = derive_seed_named(derive_seed(42, 1u64), "bolt");
        let mut expected_bolt = BoltRng::from_seed(expected_bolt_seed);
        let expected_draw: u64 = expected_bolt.0.random();

        let actual_draw: u64 = app.world_mut().resource_mut::<BoltRng>().0.random();
        assert_eq!(
            actual_draw, expected_draw,
            "BoltRng must be reseeded to node_index=1 on second OnEnter(NodeState::Loading)"
        );
    }

    // B27 — seed_node_rng is in SeedNodeRngSystems::Seed
    #[test]
    fn seed_node_rng_is_in_seed_set() {
        let mut app = TestAppBuilder::new()
            .with_state_hierarchy()
            .with_resource::<RunStats>()
            .with_resource::<NodeOutcome>()
            .with_resource::<RunSeed>()
            .with_resource::<BoltRng>()
            .with_resource::<NodeGenRng>()
            .with_resource::<EffectEventCounter>()
            .build();
        app.init_resource::<RoutingTable<NodeState>>();
        // RunPlugin registers seed_node_rng in SeedNodeRngSystems::Seed.
        app.add_plugins(RunPlugin);
        assert!(
            system_in_set(
                &mut app,
                OnEnter(NodeState::Loading),
                seed_node_rng,
                SeedNodeRngSystems::Seed,
            ),
            "seed_node_rng must be in SeedNodeRngSystems::Seed in OnEnter(NodeState::Loading)"
        );
    }

    #[test]
    fn seed_node_rng_is_not_in_prepare_seed_set() {
        let mut app = TestAppBuilder::new()
            .with_state_hierarchy()
            .with_resource::<RunStats>()
            .with_resource::<NodeOutcome>()
            .with_resource::<RunSeed>()
            .with_resource::<BoltRng>()
            .with_resource::<NodeGenRng>()
            .with_resource::<EffectEventCounter>()
            .build();
        app.init_resource::<RoutingTable<NodeState>>();
        app.add_plugins(RunPlugin);
        assert!(
            !system_in_set(
                &mut app,
                OnEnter(NodeState::Loading),
                seed_node_rng,
                SeedNodeRngSystems::PrepareSeed,
            ),
            "seed_node_rng must NOT be in SeedNodeRngSystems::PrepareSeed"
        );
    }

    #[test]
    fn seed_node_rng_is_not_in_consume_seed_set() {
        let mut app = TestAppBuilder::new()
            .with_state_hierarchy()
            .with_resource::<RunStats>()
            .with_resource::<NodeOutcome>()
            .with_resource::<RunSeed>()
            .with_resource::<BoltRng>()
            .with_resource::<NodeGenRng>()
            .with_resource::<EffectEventCounter>()
            .build();
        app.init_resource::<RoutingTable<NodeState>>();
        app.add_plugins(RunPlugin);
        assert!(
            !system_in_set(
                &mut app,
                OnEnter(NodeState::Loading),
                seed_node_rng,
                SeedNodeRngSystems::ConsumeSeed,
            ),
            "seed_node_rng must NOT be in SeedNodeRngSystems::ConsumeSeed"
        );
    }

    // B28 — setup_run and reset_bolt are in SeedNodeRngSystems::ConsumeSeed
    #[test]
    fn setup_run_is_in_consume_seed_set() {
        let mut app = TestAppBuilder::new()
            .with_state_hierarchy()
            .with_resource::<RunStats>()
            .with_resource::<NodeOutcome>()
            .with_resource::<RunSeed>()
            .with_resource::<BoltRng>()
            .with_resource::<NodeGenRng>()
            .with_resource::<EffectEventCounter>()
            .build();
        app.init_resource::<RoutingTable<NodeState>>();
        app.add_plugins(RunPlugin);
        assert!(
            system_in_set(
                &mut app,
                OnEnter(NodeState::Loading),
                setup_run,
                SeedNodeRngSystems::ConsumeSeed,
            ),
            "setup_run must be in SeedNodeRngSystems::ConsumeSeed"
        );
    }

    #[test]
    fn reset_bolt_is_in_consume_seed_set() {
        let mut app = TestAppBuilder::new()
            .with_state_hierarchy()
            .with_resource::<RunStats>()
            .with_resource::<NodeOutcome>()
            .with_resource::<RunSeed>()
            .with_resource::<BoltRng>()
            .with_resource::<NodeGenRng>()
            .with_resource::<EffectEventCounter>()
            .build();
        app.init_resource::<RoutingTable<NodeState>>();
        app.add_plugins(RunPlugin);
        assert!(
            system_in_set(
                &mut app,
                OnEnter(NodeState::Loading),
                reset_bolt,
                SeedNodeRngSystems::ConsumeSeed,
            ),
            "reset_bolt must be in SeedNodeRngSystems::ConsumeSeed in OnEnter(NodeState::Loading)"
        );
    }

    #[test]
    fn setup_run_is_not_in_seed_set() {
        let mut app = TestAppBuilder::new()
            .with_state_hierarchy()
            .with_resource::<RunStats>()
            .with_resource::<NodeOutcome>()
            .with_resource::<RunSeed>()
            .with_resource::<BoltRng>()
            .with_resource::<NodeGenRng>()
            .with_resource::<EffectEventCounter>()
            .build();
        app.init_resource::<RoutingTable<NodeState>>();
        app.add_plugins(RunPlugin);
        assert!(
            !system_in_set(
                &mut app,
                OnEnter(NodeState::Loading),
                setup_run,
                SeedNodeRngSystems::Seed,
            ),
            "setup_run must NOT be in SeedNodeRngSystems::Seed (consumer, not seeder)"
        );
    }

    // B29 — capture_run_seed is RELOCATED in Wave 2E from
    // `OnEnter(NodeState::Loading) / SeedNodeRngSystems::PrepareSeed` to
    // `OnExit(MenuState::Main) / RunStartSystems::CaptureSeed`. The
    // contradictory Wave 1 assertion (formerly here) is intentionally
    // removed; see Wave 2E B1 (`capture_run_seed_not_in_seed_node_rng_prepare_seed_set`)
    // in `loading/systems/capture_run_seed.rs` for the new pin.

    #[test]
    fn capture_run_seed_is_not_in_seed_set() {
        let mut app = TestAppBuilder::new()
            .with_state_hierarchy()
            .with_resource::<RunStats>()
            .with_resource::<NodeOutcome>()
            .with_resource::<RunSeed>()
            .with_resource::<BoltRng>()
            .with_resource::<NodeGenRng>()
            .with_resource::<EffectEventCounter>()
            .build();
        app.init_resource::<RoutingTable<NodeState>>();
        app.add_plugins(RunPlugin);
        assert!(
            !system_in_set(
                &mut app,
                OnEnter(NodeState::Loading),
                capture_run_seed,
                SeedNodeRngSystems::Seed,
            ),
            "capture_run_seed must NOT be in SeedNodeRngSystems::Seed"
        );
    }
}
