//! System to capture the run seed into `RunStats` at node start.

use bevy::prelude::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::{prelude::*, shared::RunSeed};

/// Captures the [`RunSeed`] value into [`RunStats::seed`] on the first node.
///
/// If [`RunSeed`] is `None`, generates a random seed from [`GameRng`].
/// Only captures once (skips if `seed` is already non-zero).
pub(crate) fn capture_run_seed(
    seed: Res<RunSeed>,
    mut stats: ResMut<RunStats>,
    mut rng: ResMut<GameRng>,
) {
    if stats.seed != 0 {
        return;
    }
    if let Some(n) = seed.0 {
        stats.seed = n;
    } else {
        let generated: u64 = rng.0.random();
        stats.seed = generated;
        rng.0 = ChaCha8Rng::seed_from_u64(stats.seed);
    }
}

#[cfg(test)]
mod tests {
    use rantzsoft_stateflow::RoutingTable;

    use super::*;
    use crate::{
        shared::{RunSeed, rng::NodeSequenceRng, test_utils::schedule_inspect::system_in_set},
        state::{
            run::{
                RunPlugin,
                loading::systems::{generate_node_sequence_system, reset_run_state},
            },
            types::*,
        },
    };

    fn test_app() -> App {
        TestAppBuilder::new()
            .with_resource::<RunStats>()
            .with_resource::<GameRng>()
            .with_resource::<RunSeed>()
            .with_system(Update, capture_run_seed)
            .build()
    }

    // ── Existing behavior tests (Behavior 4 — intentionally-green guards) ──

    #[test]
    fn captures_specific_seed_into_stats() {
        let mut app = test_app();
        app.insert_resource(RunSeed(Some(42)));
        app.update();

        let stats = app.world().resource::<RunStats>();
        assert_eq!(
            stats.seed, 42,
            "RunStats.seed should be 42 when RunSeed is Some(42)"
        );
    }

    #[test]
    fn only_captures_seed_once() {
        let mut app = test_app();
        app.insert_resource(RunSeed(Some(42)));
        app.update();

        // Change the RunSeed and run again
        app.insert_resource(RunSeed(Some(99)));
        app.update();

        let stats = app.world().resource::<RunStats>();
        assert_eq!(
            stats.seed, 42,
            "RunStats.seed should remain 42 (not overwritten on second node)"
        );
    }

    #[test]
    fn generates_random_seed_when_run_seed_is_none() {
        let mut app = test_app();
        // RunSeed defaults to None
        app.update();

        let stats = app.world().resource::<RunStats>();
        assert_ne!(
            stats.seed, 0,
            "RunStats.seed should be non-zero when generated from RNG"
        );
    }

    // ── Schedule app helper (used by Behaviors 1, 2, 3, 19) ──────────────────

    fn schedule_app() -> App {
        let mut app = TestAppBuilder::new()
            .with_state_hierarchy()
            .with_resource::<RunStats>()
            .with_resource::<RunSeed>()
            .with_resource::<NodeSequenceRng>()
            .build();
        app.init_resource::<RoutingTable<NodeState>>();
        // Resources required by reset_run_state (RunInventories) which RunPlugin
        // wires into OnExit(MenuState::Main) but does not init itself.
        app.init_resource::<crate::shared::rng::EffectBaseSeed>();
        app.init_resource::<crate::chips::inventory::ChipInventory>();
        app.init_resource::<crate::mutators::protocols::resources::ActiveProtocols>();
        app.init_resource::<crate::mutators::protocols::resources::ProtocolOffer>();
        app.init_resource::<crate::mutators::protocols::resources::ProtocolOfferingCount>();
        app.init_resource::<crate::mutators::hazards::resources::ActiveHazards>();
        app.init_resource::<crate::mutators::protocols::greed::GreedStacks>();
        app.init_resource::<crate::mutators::protocols::siphon::SiphonStreak>();
        app.init_resource::<crate::shared::rng::ChipSelectCount>();
        app.init_resource::<crate::state::run::NodeLayoutRegistry>();
        app.add_plugins(RunPlugin);
        app
    }

    // ── Group A — Behavior 1: capture_run_seed is NOT in OnEnter(NodeState::Loading) ──

    #[test]
    fn capture_run_seed_not_in_seed_node_rng_prepare_seed_set() {
        let mut app = schedule_app();
        assert!(
            !system_in_set(
                &mut app,
                OnEnter(NodeState::Loading),
                capture_run_seed,
                crate::shared::rng::SeedNodeRngSystems::PrepareSeed,
            ),
            "capture_run_seed must NOT be in SeedNodeRngSystems::PrepareSeed after Wave 2E \
             relocation"
        );
    }

    // Behavior 1 edge case: also absent from Seed and ConsumeSeed
    #[test]
    fn capture_run_seed_absent_from_all_on_enter_node_loading_sets() {
        let mut app = schedule_app();
        assert!(
            !system_in_set(
                &mut app,
                OnEnter(NodeState::Loading),
                capture_run_seed,
                crate::shared::rng::SeedNodeRngSystems::Seed,
            ),
            "capture_run_seed must NOT be in SeedNodeRngSystems::Seed"
        );
        assert!(
            !system_in_set(
                &mut app,
                OnEnter(NodeState::Loading),
                capture_run_seed,
                crate::shared::rng::SeedNodeRngSystems::ConsumeSeed,
            ),
            "capture_run_seed must NOT be in SeedNodeRngSystems::ConsumeSeed"
        );
    }

    // ── Group A — Behavior 2: capture_run_seed IS in OnExit(MenuState::Main) ──

    #[test]
    fn capture_run_seed_in_run_start_systems_capture_seed_set() {
        let mut app = schedule_app();
        assert!(
            system_in_set(
                &mut app,
                OnExit(MenuState::Main),
                capture_run_seed,
                crate::state::run::RunStartSystems::CaptureSeed,
            ),
            "capture_run_seed must be in RunStartSystems::CaptureSeed in \
             OnExit(MenuState::Main)"
        );
    }

    // Behavior 2 edge case: capture_run_seed NOT in ResetState or GenerateSequence
    #[test]
    fn capture_run_seed_not_in_reset_state_or_generate_sequence_sets() {
        let mut app = schedule_app();
        assert!(
            !system_in_set(
                &mut app,
                OnExit(MenuState::Main),
                capture_run_seed,
                crate::state::run::RunStartSystems::ResetState,
            ),
            "capture_run_seed must NOT be in RunStartSystems::ResetState"
        );
        assert!(
            !system_in_set(
                &mut app,
                OnExit(MenuState::Main),
                capture_run_seed,
                crate::state::run::RunStartSystems::GenerateSequence,
            ),
            "capture_run_seed must NOT be in RunStartSystems::GenerateSequence"
        );
    }

    // ── Group A — Behavior 3: RunStartSystems ordering ───────────────────────

    #[test]
    fn run_start_systems_each_in_correct_set_and_pairwise_distinct() {
        let mut app = schedule_app();

        // Positive assertions
        assert!(
            system_in_set(
                &mut app,
                OnExit(MenuState::Main),
                reset_run_state,
                crate::state::run::RunStartSystems::ResetState,
            ),
            "reset_run_state must be in RunStartSystems::ResetState"
        );
        assert!(
            system_in_set(
                &mut app,
                OnExit(MenuState::Main),
                capture_run_seed,
                crate::state::run::RunStartSystems::CaptureSeed,
            ),
            "capture_run_seed must be in RunStartSystems::CaptureSeed"
        );
        assert!(
            system_in_set(
                &mut app,
                OnExit(MenuState::Main),
                generate_node_sequence_system,
                crate::state::run::RunStartSystems::GenerateSequence,
            ),
            "generate_node_sequence_system must be in RunStartSystems::GenerateSequence"
        );

        // Negative pairwise assertions
        assert!(
            !system_in_set(
                &mut app,
                OnExit(MenuState::Main),
                reset_run_state,
                crate::state::run::RunStartSystems::CaptureSeed,
            ),
            "reset_run_state must NOT be in RunStartSystems::CaptureSeed"
        );
        assert!(
            !system_in_set(
                &mut app,
                OnExit(MenuState::Main),
                reset_run_state,
                crate::state::run::RunStartSystems::GenerateSequence,
            ),
            "reset_run_state must NOT be in RunStartSystems::GenerateSequence"
        );
        assert!(
            !system_in_set(
                &mut app,
                OnExit(MenuState::Main),
                capture_run_seed,
                crate::state::run::RunStartSystems::ResetState,
            ),
            "capture_run_seed must NOT be in RunStartSystems::ResetState"
        );
        assert!(
            !system_in_set(
                &mut app,
                OnExit(MenuState::Main),
                capture_run_seed,
                crate::state::run::RunStartSystems::GenerateSequence,
            ),
            "capture_run_seed must NOT be in RunStartSystems::GenerateSequence"
        );
        assert!(
            !system_in_set(
                &mut app,
                OnExit(MenuState::Main),
                generate_node_sequence_system,
                crate::state::run::RunStartSystems::ResetState,
            ),
            "generate_node_sequence_system must NOT be in RunStartSystems::ResetState"
        );
        assert!(
            !system_in_set(
                &mut app,
                OnExit(MenuState::Main),
                generate_node_sequence_system,
                crate::state::run::RunStartSystems::CaptureSeed,
            ),
            "generate_node_sequence_system must NOT be in RunStartSystems::CaptureSeed"
        );
    }

    // ── Group D' — Behavior 19: multi-run seed correctness ───────────────────

    // Drives app through the full sub-state hierarchy to reach MenuState::Main,
    // then exits it via GameState::Run — firing OnExit(MenuState::Main).
    // AppState → AppState::Game → GameState::Menu → MenuState::Main → GameState::Run.
    fn drive_through_menu_main_then_exit(app: &mut App) {
        app.world_mut()
            .resource_mut::<NextState<AppState>>()
            .set(AppState::Game);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Menu);
        app.update();
        app.world_mut()
            .resource_mut::<NextState<MenuState>>()
            .set(MenuState::Main);
        app.update();
        // Exit MenuState::Main — fires OnExit(MenuState::Main).
        app.world_mut()
            .resource_mut::<NextState<GameState>>()
            .set(GameState::Run);
        app.update();
    }

    // Simulates run 2 after run 1: pre-seeds RunStats with leftover seed 42,
    // sets RunSeed to Some(99), drives OnExit(MenuState::Main), then asserts
    // that RunStats.seed == 99 (not the leftover 42).
    // RED-phase note: assertion is checked AFTER OnExit(MenuState::Main) fires.
    // If capture_run_seed is still wired only to OnEnter(NodeState::Loading),
    // seed remains 42 (reset_run_state cleared it to 0 then capture_run_seed
    // didn't run yet). If reset_run_state didn't run before capture_run_seed,
    // the early-return guard (seed != 0) blocks re-capture and seed stays 42.
    #[test]
    fn multi_run_captures_second_run_seed_after_first_run_completes() {
        let mut app = schedule_app();

        // Simulate leftover state from run 1
        app.insert_resource(RunStats {
            seed: 42,
            ..default()
        });
        app.insert_resource(RunSeed(Some(99)));

        drive_through_menu_main_then_exit(&mut app);

        let seed = app.world().resource::<RunStats>().seed;
        assert_eq!(
            seed, 99,
            "RunStats.seed must be 99 (run 2's seed) after OnExit(MenuState::Main); \
             if 0, capture_run_seed is not yet wired into OnExit(MenuState::Main); \
             if 42, reset_run_state did not run before capture_run_seed"
        );
    }

    // Behavior 19 edge case A: unseeded run 2 — leftover 42 is cleared
    #[test]
    fn multi_run_unseeded_run2_clears_leftover_seed() {
        let mut app = schedule_app();

        app.insert_resource(RunStats {
            seed: 42,
            ..default()
        });
        app.insert_resource(RunSeed(None)); // unseeded run 2

        drive_through_menu_main_then_exit(&mut app);

        let seed = app.world().resource::<RunStats>().seed;
        assert_ne!(
            seed, 42,
            "RunStats.seed must NOT be 42 (leftover from run 1) for an unseeded run 2"
        );
        assert_ne!(
            seed, 0,
            "RunStats.seed must be non-zero for an unseeded run 2 (OS-entropy path)"
        );
    }

    // Behavior 19 edge case B: run 2 seed == run 1 seed (corner case)
    #[test]
    fn multi_run_same_seed_both_runs_still_captures_correctly() {
        let mut app = schedule_app();

        app.insert_resource(RunStats {
            seed: 42,
            ..default()
        });
        app.insert_resource(RunSeed(Some(42))); // same seed as run 1

        drive_through_menu_main_then_exit(&mut app);

        let seed = app.world().resource::<RunStats>().seed;
        assert_eq!(
            seed, 42,
            "RunStats.seed must be 42 when run 2's RunSeed is also Some(42)"
        );
    }
}
