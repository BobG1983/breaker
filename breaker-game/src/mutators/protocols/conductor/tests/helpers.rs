//! Shared test fixtures for Conductor protocol tests.
//!
//! App builders, canonical `ConductorConfig`, bolt spawners with distinct
//! `BoundEffects` / `StagedEffects` fingerprints, a `BumpPerformed` writer,
//! an `activate_now` `CommandQueue` helper, and marker/fingerprint probes used
//! across the 10 behavior files.

use bevy::{
    ecs::{message::Messages, world::CommandQueue},
    prelude::*,
};

use super::super::system::{ConductorConfig, activate, register};
use crate::{
    bolt::components::{ExtraBolt, PrimaryBolt},
    breaker::messages::BumpGrade,
    effect_v3::types::Tree,
    mutators::protocols::{
        definition::{ProtocolDefinition, ProtocolTuning},
        resources::ActiveProtocols,
    },
    prelude::*,
};

// ── App builders ────────────────────────────────────────────────────────────

/// Default Conductor test app. State hierarchy in `NodeState::Playing`,
/// `ActiveProtocols` initialised, `BumpPerformed` message registered,
/// canonical `ConductorConfig` inserted, and `register` called.
///
/// Does NOT seed `ActiveProtocols` with Conductor — tests that need the
/// protocol active call [`seed_active_protocols_with_conductor`].
pub(super) fn build_conductor_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveProtocols>()
        .with_message::<BumpPerformed>()
        .build();
    app.world_mut()
        .insert_resource(canonical_conductor_config());
    register(&mut app);
    app
}

/// Same as [`build_conductor_app`] but omits `ConductorConfig`. Used to
/// exercise the harness-safe `Option<Res<ConductorConfig>>` early-return guard
/// path in `conductor_swap_on_perfect_bump`.
pub(super) fn build_conductor_app_no_config() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveProtocols>()
        .with_message::<BumpPerformed>()
        .build();
    register(&mut app);
    app
}

/// Same as [`build_conductor_app`] but uses `ChipSelectState::Selecting`
/// instead of `NodeState::Playing`. Used for `in_state(NodeState::Playing)`
/// gate tests.
pub(super) fn build_conductor_app_in_chip_selecting() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_chip_selecting()
        .with_resource::<ActiveProtocols>()
        .with_message::<BumpPerformed>()
        .build();
    app.world_mut()
        .insert_resource(canonical_conductor_config());
    register(&mut app);
    app
}

// ── Config helpers ──────────────────────────────────────────────────────────

/// Canonical Conductor config used across tests. Matches the presence-marker
/// unit struct installed by `activate` and referenced by the harness-safety
/// gate in `conductor_swap_on_perfect_bump`.
pub(super) const fn canonical_conductor_config() -> ConductorConfig {
    ConductorConfig
}

/// Inserts a Conductor `ProtocolDefinition` into `ActiveProtocols` so the
/// `protocol_active(Conductor)` run-condition passes.
pub(super) fn seed_active_protocols_with_conductor(app: &mut App) {
    app.world_mut()
        .resource_mut::<ActiveProtocols>()
        .insert(ProtocolDefinition {
            name:        "Conductor".into(),
            description: String::new(),
            unlock_tier: 0,
            tuning:      ProtocolTuning::Conductor,
        });
}

/// Inserts a non-Conductor `ProtocolDefinition` (Greed) into `ActiveProtocols`.
/// Used to prove that an unrelated active protocol does NOT gate-open
/// Conductor's swap system.
pub(super) fn seed_active_protocols_with_greed(app: &mut App) {
    app.world_mut()
        .resource_mut::<ActiveProtocols>()
        .insert(ProtocolDefinition {
            name:        "Greed".into(),
            description: String::new(),
            unlock_tier: 0,
            tuning:      ProtocolTuning::Greed {
                rarity_boost_per_skip: 0.05,
            },
        });
}

/// Invokes `conductor::activate` directly via a fresh `CommandQueue` so each
/// call has an independent, deterministic flush. Mirrors the Reckless Dash /
/// Burnout `activate_now` helpers.
pub(super) fn activate_now(app: &mut App, tuning: &ProtocolTuning) {
    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, app.world());
        activate(tuning, &mut commands);
    }
    queue.apply(app.world_mut());
}

/// Inserts the canonical `ConductorConfig` directly (no `activate` call).
/// Used by tests that want to observe `activate` preserving a pre-existing
/// config across a mismatched-tuning call.
pub(super) fn install_conductor_config(app: &mut App, cfg: ConductorConfig) {
    app.world_mut().insert_resource(cfg);
}

// ── Entity spawners ─────────────────────────────────────────────────────────

/// Spawns `(Bolt, PrimaryBolt)`.
pub(super) fn spawn_primary_bolt(app: &mut App) -> Entity {
    app.world_mut().spawn((Bolt, PrimaryBolt)).id()
}

/// Spawns `(Bolt, ExtraBolt)`.
pub(super) fn spawn_extra_bolt(app: &mut App) -> Entity {
    app.world_mut().spawn((Bolt, ExtraBolt)).id()
}

/// Spawns `(Bolt, PrimaryBolt, BoundEffects(...))`.
pub(super) fn spawn_primary_bolt_with_bound(app: &mut App, bound: BoundEffects) -> Entity {
    app.world_mut().spawn((Bolt, PrimaryBolt, bound)).id()
}

/// Spawns `(Bolt, ExtraBolt, BoundEffects(...))`.
pub(super) fn spawn_extra_bolt_with_bound(app: &mut App, bound: BoundEffects) -> Entity {
    app.world_mut().spawn((Bolt, ExtraBolt, bound)).id()
}

/// Spawns `(Bolt, PrimaryBolt, BoundEffects(...), StagedEffects(...))`.
pub(super) fn spawn_primary_bolt_with_bound_and_staged(
    app: &mut App,
    bound: BoundEffects,
    staged: StagedEffects,
) -> Entity {
    app.world_mut()
        .spawn((Bolt, PrimaryBolt, bound, staged))
        .id()
}

/// Spawns `(Bolt, ExtraBolt, BoundEffects(...), StagedEffects(...))`.
pub(super) fn spawn_extra_bolt_with_bound_and_staged(
    app: &mut App,
    bound: BoundEffects,
    staged: StagedEffects,
) -> Entity {
    app.world_mut().spawn((Bolt, ExtraBolt, bound, staged)).id()
}

/// Spawns an empty entity. Used as the `BumpPerformed.breaker` placeholder
/// (Conductor's swap system does NOT inspect the breaker).
pub(super) fn spawn_dummy_breaker(app: &mut App) -> Entity {
    app.world_mut().spawn_empty().id()
}

// ── Fingerprint constructors ────────────────────────────────────────────────

/// Returns `BoundEffects(vec![(tag, Tree::Sequence(vec![]))])`. The tag is a
/// fingerprint that tests can assert on without depending on `PartialEq` of
/// `BoundEffects` itself (which does not derive `PartialEq`).
pub(super) fn make_distinct_bound(tag: &str) -> BoundEffects {
    BoundEffects(vec![(tag.to_string(), Tree::Sequence(vec![]))])
}

/// Returns `StagedEffects(vec![(tag, Tree::Sequence(vec![]))])`. Mirror of
/// [`make_distinct_bound`] for the staged component.
pub(super) fn make_distinct_staged(tag: &str) -> StagedEffects {
    StagedEffects(vec![(tag.to_string(), Tree::Sequence(vec![]))])
}

// ── Message writers ─────────────────────────────────────────────────────────

/// Writes a single `BumpPerformed` message with the supplied grade / bolt /
/// breaker. Conductor ignores `breaker` but the message schema requires it.
pub(super) fn write_bump_performed(
    app: &mut App,
    breaker: Entity,
    bolt: Option<Entity>,
    grade: BumpGrade,
) {
    app.world_mut()
        .resource_mut::<Messages<BumpPerformed>>()
        .write(BumpPerformed {
            grade,
            bolt,
            breaker,
        });
}

// ── Assertion helpers ───────────────────────────────────────────────────────

/// Returns `true` if the entity carries the `PrimaryBolt` marker.
pub(super) fn has_primary(app: &App, bolt: Entity) -> bool {
    app.world().get::<PrimaryBolt>(bolt).is_some()
}

/// Returns `true` if the entity carries the `ExtraBolt` marker.
pub(super) fn has_extra(app: &App, bolt: Entity) -> bool {
    app.world().get::<ExtraBolt>(bolt).is_some()
}

/// Returns the list of `BoundEffects` fingerprint tags on the entity, in
/// insertion order. Empty vec if the component is absent.
pub(super) fn bound_fingerprints(app: &App, bolt: Entity) -> Vec<String> {
    app.world()
        .get::<BoundEffects>(bolt)
        .map(|b| b.0.iter().map(|(n, _)| n.clone()).collect())
        .unwrap_or_default()
}

/// Returns the list of `StagedEffects` fingerprint tags on the entity, in
/// insertion order. Empty vec if the component is absent.
pub(super) fn staged_fingerprints(app: &App, bolt: Entity) -> Vec<String> {
    app.world()
        .get::<StagedEffects>(bolt)
        .map(|s| s.0.iter().map(|(n, _)| n.clone()).collect())
        .unwrap_or_default()
}
