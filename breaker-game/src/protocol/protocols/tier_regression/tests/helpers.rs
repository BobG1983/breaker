//! Shared test fixtures for Tier Regression protocol tests.
//!
//! App builders, `activate_now` (CommandQueue-flush pattern), direct-install
//! helpers for `TierRegressionConfig` / `TierRegressionPending` / `NodeOutcome`
//! / `NodeSequence`, and `ActiveProtocols` seeding.

use bevy::{ecs::world::CommandQueue, prelude::*};

use super::super::system::{
    TierRegressionConfig, TierRegressionPending, activate, apply_tier_regression, register,
};
use crate::{
    prelude::*,
    protocol::{
        definition::{ProtocolDefinition, ProtocolTuning},
        resources::ActiveProtocols,
    },
    state::run::{
        node::NodeSystems,
        resources::{NodeAssignment, NodeOutcome, NodeSequence},
        systems::advance_node,
    },
};

// ── App builders ────────────────────────────────────────────────────────────

/// Group B app: registers `apply_tier_regression` on `Update` so Group B
/// tests can invoke the body directly with one `app.update()` without
/// needing the full state hierarchy.
pub(super) fn build_apply_app() -> App {
    TestAppBuilder::new()
        .with_system(Update, apply_tier_regression)
        .build()
}

/// Group C app: full state hierarchy driven to `AppState::Game` +
/// `GameState::Run`, but NOT yet into `RunState::Node`. Initializes
/// `ActiveProtocols` (the `protocol_active` run-condition requires this
/// resource to exist — otherwise it panics when `OnEnter(RunState::Node)`
/// fires). Also registers the real `advance_node` tagged with
/// `NodeSystems::AdvanceNode` on `OnEnter(RunState::Node)` so the
/// `.before(NodeSystems::AdvanceNode)` ordering edge resolves and Group C
/// can observe `advance_node`'s contribution.
pub(super) fn build_register_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .with_resource::<ActiveProtocols>()
        .build();
    drive_to_run(&mut app);
    app.add_systems(
        OnEnter(RunState::Node),
        advance_node.in_set(NodeSystems::AdvanceNode),
    );
    register(&mut app);
    app
}

/// Drives the state hierarchy from `AppState::Startup` through
/// `AppState::Game` → `GameState::Run` (but not into `RunState::Node`).
fn drive_to_run(app: &mut App) {
    app.world_mut()
        .resource_mut::<NextState<AppState>>()
        .set(AppState::Game);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<GameState>>()
        .set(GameState::Run);
    app.update();
}

/// Drives `NextState<RunState>::set(RunState::Node)` and runs a single
/// `app.update()`. Use this in Group C tests to trigger the
/// `OnEnter(RunState::Node)` edge that `register` wires
/// `apply_tier_regression` onto.
pub(super) fn enter_run_state_node(app: &mut App) {
    app.world_mut()
        .resource_mut::<NextState<RunState>>()
        .set(RunState::Node);
    app.update();
}

/// Drives `NextState<RunState>::set(target)` and runs a single `app.update()`
/// — used to leave `RunState::Node` for the one-shot re-entry edge case.
pub(super) fn leave_run_state_node_to(app: &mut App, target: RunState) {
    app.world_mut()
        .resource_mut::<NextState<RunState>>()
        .set(target);
    app.update();
}

// ── Direct resource installers ──────────────────────────────────────────────

/// Inserts `TierRegressionConfig { tiers_back }` as a resource.
pub(super) fn install_config(app: &mut App, tiers_back: u32) {
    app.world_mut()
        .insert_resource(TierRegressionConfig { tiers_back });
}

/// Inserts a default `TierRegressionPending` marker as a resource. The
/// `activation_tier` / `activation_node_index` fields start as `None`; in
/// Group C (`build_register_app`) the `snapshot_pre_advance_state` system
/// fills them on `OnEnter(RunState::Node)`, and in Group B
/// (`build_apply_app`) `apply_tier_regression` falls back to the current
/// `NodeOutcome` values.
pub(super) fn install_pending(app: &mut App) {
    app.world_mut()
        .insert_resource(TierRegressionPending::default());
}

/// Inserts a `NodeSequence { assignments }` as a resource.
pub(super) fn install_sequence_with_tiers(app: &mut App, assignments: Vec<NodeAssignment>) {
    app.world_mut()
        .insert_resource(NodeSequence { assignments });
}

/// Inserts a `NodeOutcome` with the given positional fields. All other
/// fields use their `Default` values.
pub(super) fn install_outcome(app: &mut App, node_index: u32, tier: u32, position_in_tier: u32) {
    app.world_mut().insert_resource(NodeOutcome {
        node_index,
        tier,
        position_in_tier,
        ..Default::default()
    });
}

/// Replaces `NodeOutcome` wholesale. Used when tests need to pin fields
/// beyond `node_index` / `tier` / `position_in_tier`.
pub(super) fn install_outcome_full(app: &mut App, outcome: NodeOutcome) {
    app.world_mut().insert_resource(outcome);
}

// ── Active protocols ────────────────────────────────────────────────────────

/// Inserts a `TierRegression` `ProtocolDefinition` into `ActiveProtocols`
/// so the `protocol_active(ProtocolKind::TierRegression)` run-condition
/// passes.
pub(super) fn seed_active_protocols_with_tier_regression(app: &mut App, tiers_back: u32) {
    app.world_mut()
        .resource_mut::<ActiveProtocols>()
        .insert(ProtocolDefinition {
            name:        "TierRegression".into(),
            description: String::new(),
            unlock_tier: 0,
            tuning:      ProtocolTuning::TierRegression { tiers_back },
        });
}

// ── Activate-now (`CommandQueue` flush) ─────────────────────────────────────

/// Invokes `tier_regression::activate` directly via a fresh `CommandQueue`
/// so each call has an independent, deterministic flush. Mirrors other
/// protocol domains' `activate_now` helpers.
pub(super) fn activate_now(app: &mut App, tuning: &ProtocolTuning) {
    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, app.world());
        activate(tuning, &mut commands);
    }
    queue.apply(app.world_mut());
}

// ── Assignment constructors ─────────────────────────────────────────────────

/// Builds a `NodeAssignment { node_type, tier_index, timer_mult }`.
pub(super) fn na(node_type: NodeType, tier_index: u32, timer_mult: f32) -> NodeAssignment {
    NodeAssignment {
        node_type,
        tier_index,
        timer_mult,
    }
}
