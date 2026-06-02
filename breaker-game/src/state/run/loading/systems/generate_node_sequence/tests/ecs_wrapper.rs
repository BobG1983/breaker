//! Tests for the ECS wrapper: `generate_node_sequence_system` inserts
//! `NodeSequence` resource and produces deterministic results via `NodeSequenceRng`.

use bevy::prelude::*;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;
use rantzsoft_stateflow::RoutingTable;

use super::{
    super::system::{generate_node_sequence, generate_node_sequence_system},
    helpers::*,
};
use crate::{
    prelude::*,
    shared::rng::{NodeSequenceRng, derive_seed_named},
    state::{
        run::{
            RunPlugin,
            resources::{DifficultyCurve, NodeAssignment, NodeSequence},
        },
        types::*,
    },
};

// ── Behavior 11 harness helper ────────────────────────────────────────────────

fn test_app_with_seed(seed: u64) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    let curve = make_curve(vec![make_tier(TierNodeCount::Fixed(3), 0.5, 1.0)], 0.0);
    app.insert_resource(curve);
    app.insert_resource(NodeSequenceRng::from_seed(seed));
    app.insert_resource(RunStats::default());
    app.add_systems(Update, generate_node_sequence_system);
    app
}

// ── Behavior 9: system uses NodeSequenceRng ──────────────────────────────────

// Behavior 9 (positive side of Behavior 11): harness inserts NodeSequenceRng;
// system runs without panic, NodeSequence is inserted.
// The negative side (old monolithic RNG must be absent from the harness)
// lives in migration_guard.rs to keep this file free of the migrated-away identifier.
#[test]
fn system_inserts_node_sequence_resource() {
    let mut app = test_app_with_seed(42);
    app.update();

    let seq = app
        .world()
        .get_resource::<NodeSequence>()
        .expect("system should insert NodeSequence resource");
    assert_eq!(
        seq.assignments.len(),
        4,
        "Fixed(3) + boss = 4 total assignments"
    );
    // Behavior 11 positive: NodeSequenceRng is present
    assert!(
        app.world().contains_resource::<NodeSequenceRng>(),
        "NodeSequenceRng must be present in the test harness after migration"
    );
}

// ── Behavior 10: determinism with NodeSequenceRng ────────────────────────────

// Direct replacement for the removed `system_generates_deterministic_sequence_from_game_rng`.
#[test]
fn system_generates_deterministic_sequence_from_node_sequence_rng() {
    let curve = make_curve(
        vec![
            make_tier(TierNodeCount::Fixed(2), 0.5, 1.0),
            make_tier(TierNodeCount::Range(3, 5), 0.4, 0.9),
        ],
        0.1,
    );

    let run_system = |seed: u64| -> Vec<NodeAssignment> {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(curve.clone());
        app.insert_resource(NodeSequenceRng::from_seed(seed));
        // Production reseeds `NodeSequenceRng` from
        // `derive_seed_named(RunStats.seed, "node_sequence")`, so the
        // user-visible knob is `RunStats.seed` — set it here so the
        // determinism / inequality assertions actually exercise the seed.
        app.insert_resource(RunStats { seed, ..default() });
        app.add_systems(Update, generate_node_sequence_system);
        app.update();
        app.world().resource::<NodeSequence>().assignments.clone()
    };

    let seq1 = run_system(42);
    let seq2 = run_system(42);
    assert_eq!(
        seq1, seq2,
        "same NodeSequenceRng seed must produce identical node sequence"
    );
    assert!(
        !seq1.is_empty(),
        "system must produce non-empty sequence for non-empty curve"
    );

    // Edge case: different seeds produce different sequences
    let seq_other = run_system(99);
    assert_ne!(
        seq1, seq_other,
        "different NodeSequenceRng seeds (42 vs 99) must produce different sequences"
    );
}

// ── Behavior 14: formula pin — NodeSequenceRng reseeded from RunStats.seed ───

#[test]
fn generate_node_sequence_system_seeds_node_sequence_rng_from_run_stats_seed() {
    let curve = make_curve(
        vec![
            make_tier(TierNodeCount::Fixed(2), 0.5, 1.0),
            make_tier(TierNodeCount::Range(3, 5), 0.4, 0.9),
        ],
        0.1,
    );

    // Build a MinimalPlugins app with RunStats.seed = 42 and NodeSequenceRng at default.
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.insert_resource(curve.clone());
    app.insert_resource(RunStats {
        seed: 42,
        ..default()
    });
    app.insert_resource(NodeSequenceRng::default());
    app.add_systems(Update, generate_node_sequence_system);
    app.update();

    let actual = app.world().resource::<NodeSequence>().assignments.clone();

    // Expected: sequence produced by ChaCha8Rng seeded from derive_seed_named(42, "node_sequence")
    let expected = {
        let derived = derive_seed_named(42, "node_sequence");
        let mut rng = ChaCha8Rng::seed_from_u64(derived);
        generate_node_sequence(&curve, &mut rng).assignments
    };

    assert_eq!(
        actual, expected,
        "NodeSequence must match generate_node_sequence seeded from \
         derive_seed_named(RunStats.seed=42, \"node_sequence\")"
    );
}

#[test]
fn generate_node_sequence_system_seed_zero_produces_non_panic_sequence() {
    // Edge case: RunStats.seed == 0 — derive_seed_named(0, "node_sequence") is non-zero per
    // Wave 1 hash contract; system must not panic and must produce a consistent result.
    let curve = make_curve(vec![make_tier(TierNodeCount::Fixed(2), 0.5, 1.0)], 0.0);

    let run = || -> Vec<NodeAssignment> {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(curve.clone());
        app.insert_resource(RunStats {
            seed: 0,
            ..default()
        });
        app.insert_resource(NodeSequenceRng::default());
        app.add_systems(Update, generate_node_sequence_system);
        app.update();
        app.world().resource::<NodeSequence>().assignments.clone()
    };

    let seq1 = run();
    let seq2 = run();
    assert_eq!(
        seq1, seq2,
        "RunStats.seed=0 must produce a deterministic (if unusual) sequence"
    );
    assert!(!seq1.is_empty());
}

// ── Behavior 16: NodeSequenceRng is registered in RunPlugin ─────────────────
// (Behavior 15 lives in migration_guard.rs alongside the structural guards
// so that ecs_wrapper.rs stays free of the old monolithic RNG identifier.)

fn schedule_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .with_resource::<RunStats>()
        .build();
    app.init_resource::<RoutingTable<NodeState>>();
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

// Behavior 16: NodeSequenceRng is registered in RunPlugin
#[test]
fn run_plugin_registers_node_sequence_rng() {
    let app = schedule_app();
    assert!(
        app.world().contains_resource::<NodeSequenceRng>(),
        "NodeSequenceRng must be registered in RunPlugin (init_resource::<NodeSequenceRng>())"
    );
}

// ── Behaviors 12 & 13: full schedule integration tests ───────────────────────

fn full_schedule_app_seeded(seed: u64) -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .with_resource::<RunStats>()
        .with_playfield()
        .with_cell_registry()
        .build();
    app.init_resource::<RoutingTable<NodeState>>();
    app.init_resource::<crate::shared::rng::EffectBaseSeed>();
    app.init_resource::<crate::chips::inventory::ChipInventory>();
    app.init_resource::<crate::mutators::protocols::resources::ActiveProtocols>();
    app.init_resource::<crate::mutators::protocols::resources::ProtocolOffer>();
    app.init_resource::<crate::mutators::protocols::resources::ProtocolOfferingCount>();
    app.init_resource::<crate::mutators::hazards::resources::ActiveHazards>();
    app.init_resource::<crate::mutators::protocols::greed::GreedStacks>();
    app.init_resource::<crate::mutators::protocols::siphon::SiphonStreak>();
    app.init_resource::<crate::shared::rng::ChipSelectCount>();
    // `spawn_cells_from_layout` (wired by `RunPlugin`) reads
    // `Res<ActiveNodeLayout>`, and `set_active_layout` runs first in the
    // `OnEnter(NodeState::Loading)` chain — it reads `Res<NodeLayoutRegistry>`
    // and `Res<ScenarioLayoutOverride>` and writes `ActiveNodeLayout` via
    // `commands.insert_resource(...)`. The Wave 2E tests don't exercise actual
    // cell spawning — they only assert on `RunStats.seed` / `NodeSequence` —
    // but the resource-dependency chain must be satisfied so the chained
    // systems can run without param-validation failures.
    //
    // (1) populate the registry with a minimal placeholder layout so
    // `set_active_layout` produces a valid `ActiveNodeLayout`,
    // (2) init `ScenarioLayoutOverride` so `set_active_layout`'s params
    // resolve, (3) also `insert_resource(ActiveNodeLayout(...))` directly so
    // the resource exists even if `set_active_layout` is skipped.
    let placeholder_layout = crate::state::run::node::definition::NodeLayout {
        name:            "test_placeholder".to_owned(),
        timer_secs:      60.0,
        cols:            1,
        rows:            1,
        grid_top_offset: 0.0,
        grid:            vec![vec![".".to_owned()]],
        pool:            crate::state::run::node::definition::NodePool::Passive,
        entity_scale:    1.0,
        locks:           None,
        sequences:       None,
    };
    let mut registry = crate::state::run::NodeLayoutRegistry::default();
    registry.insert(placeholder_layout.clone());
    app.insert_resource(registry);
    app.init_resource::<crate::state::run::node::ScenarioLayoutOverride>();
    app.insert_resource(crate::state::run::node::ActiveNodeLayout(
        placeholder_layout,
    ));
    app.add_plugins(RunPlugin);
    // Register message channels for systems that fire in the
    // `OnEnter(NodeState::Loading)` chain. Without these the `MessageWriter`
    // params fail validation in `reset_bolt` (BoltSpawned), `setup_run`
    // (BreakerSpawned + BoltSpawned), and `spawn_cells_from_layout`
    // (CellsSpawned). `RunPlugin` doesn't register these because they are
    // owned by `BoltPlugin` / `BreakerPlugin` / NodePlugin's cell side, which
    // this harness intentionally doesn't pull in.
    app.add_message::<crate::bolt::messages::BoltSpawned>();
    app.add_message::<crate::breaker::messages::BreakerSpawned>();
    app.add_message::<crate::state::run::node::messages::CellsSpawned>();
    // HUD resources read by `spawn_timer_hud` (chain B of NodePlugin's
    // OnEnter(NodeState::Loading)): `Res<TimerUiConfig>`, `Res<NodeTimer>`.
    // `Res<AssetServer>` is provided by DefaultPlugins via `TestAppBuilder`.
    app.init_resource::<crate::state::run::node::hud::resources::TimerUiConfig>();
    app.init_resource::<crate::state::run::node::NodeTimer>();
    // `setup_run` (RunPlugin OnEnter chain) reads `SetupRunContext` =
    // SelectedBreaker / BreakerRegistry / BoltRegistry / NodeOutcome. The
    // Wave 2E tests don't exercise actual breaker/bolt spawning (no breaker
    // is pre-spawned and the registries are empty, so `setup_run` early-
    // returns after looking up a missing breaker definition), but the
    // resources must exist for system param validation.
    app.init_resource::<crate::breaker::SelectedBreaker>();
    app.init_resource::<crate::breaker::BreakerRegistry>();
    app.init_resource::<crate::bolt::BoltRegistry>();
    // Override `RunPlugin`'s default empty `DifficultyCurve` with a non-empty
    // one. Without this, `generate_node_sequence_system` iterates zero tiers
    // and produces `NodeSequence { assignments: vec![] }`, which makes both
    // `node_sequence_matches_formula_for_run_seed` (where the test re-derives
    // expected from this same resource) and
    // `node_sequence_differs_for_different_run_seeds` (where two empty vecs
    // are equal, breaking the `!=` assertion) fail.
    app.insert_resource(make_curve(
        vec![
            make_tier(TierNodeCount::Fixed(2), 0.5, 1.0),
            make_tier(TierNodeCount::Range(3, 5), 0.4, 0.9),
        ],
        0.1,
    ));
    app.insert_resource(crate::shared::RunSeed(Some(seed)));
    app
}

// Drives through the full sub-state hierarchy to fire OnExit(MenuState::Main)
// and then reach NodeState::Loading.
// Order: AppState::Game → GameState::Menu → MenuState::Main → GameState::Run
//        → RunState::Node (reaches NodeState::Loading via OnEnter).
fn drive_to_node_loading(app: &mut App) {
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
    app.world_mut()
        .resource_mut::<NextState<RunState>>()
        .set(RunState::Node);
    app.update();
}

// Behavior 12: RunStats.seed is non-zero by the time we've driven through
// OnExit(MenuState::Main) and into NodeState::Loading.
#[test]
fn run_stats_seed_is_set_before_node_loading() {
    let mut app = full_schedule_app_seeded(42);
    drive_to_node_loading(&mut app);

    let seed = app.world().resource::<RunStats>().seed;
    assert_eq!(
        seed, 42,
        "RunStats.seed must be 42 (from RunSeed(Some(42))) by the time NodeState::Loading is reached"
    );
}

// Behavior 12 edge case: unseeded run also populates RunStats.seed before NodeState::Loading
#[test]
fn run_stats_seed_is_non_zero_before_node_loading_when_run_seed_is_none() {
    // Use `full_schedule_app_seeded` as the base — RunPlugin requires the
    // same `RunInventories` / playfield / layout resources regardless of
    // whether the run is seeded or not. Override `RunSeed` with `None` after
    // build to exercise the OS-entropy fallback in `capture_run_seed`.
    let mut app = full_schedule_app_seeded(0);
    app.insert_resource(crate::shared::RunSeed(None));
    drive_to_node_loading(&mut app);

    let seed = app.world().resource::<RunStats>().seed;
    assert_ne!(
        seed, 0,
        "RunStats.seed must be non-zero even for an unseeded run before NodeState::Loading"
    );
}

// Behavior 13: NodeSequence matches the formula for the given RunSeed.
#[test]
fn node_sequence_matches_formula_for_run_seed() {
    let mut app = full_schedule_app_seeded(42);
    drive_to_node_loading(&mut app);

    let actual = app
        .world()
        .get_resource::<NodeSequence>()
        .expect("NodeSequence must be inserted by generate_node_sequence_system")
        .assignments
        .clone();

    // Expected: generate_node_sequence with ChaCha8Rng seeded from derive_seed_named(42, "node_sequence").
    // DifficultyCurve comes from RunPlugin's init (the production default).
    let curve = app.world().resource::<DifficultyCurve>().clone();
    let expected = {
        let derived = derive_seed_named(42, "node_sequence");
        let mut rng = ChaCha8Rng::seed_from_u64(derived);
        generate_node_sequence(&curve, &mut rng).assignments
    };

    assert_eq!(
        actual, expected,
        "NodeSequence.assignments must be bit-identical to the formula's output for RunSeed(Some(42))"
    );
}

// Behavior 13 edge case: different RunSeed produces different assignments
#[test]
fn node_sequence_differs_for_different_run_seeds() {
    let seq_for = |seed: u64| -> Vec<NodeAssignment> {
        let mut app = full_schedule_app_seeded(seed);
        drive_to_node_loading(&mut app);
        app.world().resource::<NodeSequence>().assignments.clone()
    };

    let seq42 = seq_for(42);
    let seq99 = seq_for(99);
    assert_ne!(
        seq42, seq99,
        "different RunSeeds (42 vs 99) must produce different NodeSequence assignments"
    );
}

// Behaviors 17 & 18 live in migration_guard.rs (sibling file) — the structural
// guards use include_str! and assert absence of an identifier, which would
// create a false-positive if that identifier appeared in the assert strings here.
