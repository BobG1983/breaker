//! Shared test fixtures for Debt Collector protocol tests.
//!
//! App builders, canonical `DebtCollectorConfig`, per-bolt component
//! installers, `ActiveProtocols` seeder, `BumpPerformed` / `BoltImpactCell` /
//! `BoltLost` message writers, `DamageDealt<Cell>` bonus collector reader,
//! and the `activate_now` CommandQueue-flush helper.
//!
//! Canonical config is `stack_per_bump: 0.5` — the design-doc worked-example
//! value. The RON asset's `stack_per_bump: 0.1` is exercised only by the
//! `ron_asset.rs` drift-guard tests via `include_str!`.

use bevy::{
    ecs::{message::Messages, world::CommandQueue},
    prelude::*,
};

use super::super::system::{DebtCashOut, DebtCollectorConfig, DebtStack, activate, register};
use crate::{
    bolt::components::BoltBaseDamage,
    breaker::messages::{BumpGrade, BumpPerformed},
    prelude::*,
    protocol::{
        definition::{ProtocolDefinition, ProtocolKind, ProtocolTuning},
        resources::ActiveProtocols,
    },
};

// ── App builders ────────────────────────────────────────────────────────────

/// Default Debt Collector test app. State hierarchy in `NodeState::Playing`,
/// `ActiveProtocols` initialised, reader messages registered,
/// `DamageDealt<Cell>` capture installed, canonical `DebtCollectorConfig`
/// inserted, and `register` called.
///
/// Does NOT seed `ActiveProtocols` with Debt Collector — tests that need the
/// protocol active call [`seed_active_protocols_with_debt_collector`].
pub(super) fn build_debt_collector_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveProtocols>()
        .with_message::<BumpPerformed>()
        .with_message::<BoltImpactCell>()
        .with_message::<BoltLost>()
        .with_message_capture::<DamageDealt<Cell>>()
        .build();
    app.world_mut()
        .insert_resource(canonical_debt_collector_config());
    register(&mut app);
    app
}

/// Same as [`build_debt_collector_app`] but omits `DebtCollectorConfig`.
/// Used to exercise the harness-safe `Option<Res<DebtCollectorConfig>>`
/// guard path on `debt_collector_on_bump`.
pub(super) fn build_debt_collector_app_no_config() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveProtocols>()
        .with_message::<BumpPerformed>()
        .with_message::<BoltImpactCell>()
        .with_message::<BoltLost>()
        .with_message_capture::<DamageDealt<Cell>>()
        .build();
    register(&mut app);
    app
}

/// Same as [`build_debt_collector_app`] but uses `ChipSelectState::Selecting`
/// instead of `NodeState::Playing`. Used for `in_state(NodeState::Playing)`
/// gate tests and for `attach_stack` cross-state tests.
pub(super) fn build_debt_collector_app_in_chip_selecting() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_chip_selecting()
        .with_resource::<ActiveProtocols>()
        .with_message::<BumpPerformed>()
        .with_message::<BoltImpactCell>()
        .with_message::<BoltLost>()
        .with_message_capture::<DamageDealt<Cell>>()
        .build();
    app.world_mut()
        .insert_resource(canonical_debt_collector_config());
    register(&mut app);
    app
}

// ── Config helpers ──────────────────────────────────────────────────────────

/// Canonical Debt Collector config used across all system-behavior tests.
/// Matches the design-doc worked example: `stack_per_bump: 0.5`.
///
/// NOTE: The RON asset's `stack_per_bump: 0.1` is exercised only by the
/// `ron_asset.rs` drift-guard tests via `include_str!` and does NOT use
/// this helper.
pub(super) const fn canonical_debt_collector_config() -> DebtCollectorConfig {
    DebtCollectorConfig {
        stack_per_bump: 0.5,
    }
}

/// Inserts the given `DebtCollectorConfig` as a resource.
pub(super) fn install_debt_collector_config(app: &mut App, cfg: DebtCollectorConfig) {
    app.world_mut().insert_resource(cfg);
}

/// Inserts a Debt Collector `ProtocolDefinition` into `ActiveProtocols` so
/// the `protocol_active(DebtCollector)` run-condition passes.
pub(super) fn seed_active_protocols_with_debt_collector(app: &mut App, stack_per_bump: f32) {
    app.world_mut()
        .resource_mut::<ActiveProtocols>()
        .insert(ProtocolDefinition {
            name:        "Debt Collector".into(),
            description: String::new(),
            unlock_tier: 0,
            tuning:      ProtocolTuning::DebtCollector { stack_per_bump },
        });
}

/// Invokes `debt_collector::activate` directly via a fresh `CommandQueue` so
/// each call has an independent, deterministic flush. Mirrors
/// `siphon::tests::helpers::activate_now`.
pub(super) fn activate_now(app: &mut App, tuning: &ProtocolTuning) {
    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, app.world());
        activate(tuning, &mut commands);
    }
    queue.apply(app.world_mut());
}

// ── Entity spawners & component installers ─────────────────────────────────-

/// Spawns a `Bolt` entity with `DebtStack(stack)` pre-attached.
pub(super) fn spawn_bolt_with_stack(app: &mut App, stack: f32) -> Entity {
    app.world_mut().spawn((Bolt, DebtStack(stack))).id()
}

/// Spawns a `Bolt` entity with `BoltBaseDamage(base)` + `DebtCashOut(stack)`.
/// Does NOT attach `DebtStack` — tests that need it seed it separately via
/// [`install_debt_stack`].
pub(super) fn spawn_bolt_with_base_damage_and_cashout(
    app: &mut App,
    base: f32,
    stack: f32,
) -> Entity {
    app.world_mut()
        .spawn((Bolt, BoltBaseDamage(base), DebtCashOut(stack)))
        .id()
}

/// Inserts/overwrites `DebtStack(value)` on the given entity.
pub(super) fn install_debt_stack(app: &mut App, bolt: Entity, value: f32) {
    app.world_mut().entity_mut(bolt).insert(DebtStack(value));
}

/// Inserts/overwrites `DebtCashOut(value)` on the given entity.
pub(super) fn install_debt_cash_out(app: &mut App, bolt: Entity, value: f32) {
    app.world_mut().entity_mut(bolt).insert(DebtCashOut(value));
}

// ── Message writers ─────────────────────────────────────────────────────────

/// Writes a single `BumpPerformed` message. `breaker` field is
/// `Entity::PLACEHOLDER` (Debt Collector does not inspect it).
pub(super) fn write_bump_performed(app: &mut App, bolt: Option<Entity>, grade: BumpGrade) {
    app.world_mut()
        .resource_mut::<Messages<BumpPerformed>>()
        .write(BumpPerformed {
            grade,
            bolt,
            breaker: Entity::PLACEHOLDER,
        });
}

/// Writes a single `BoltLost` message. `breaker` field is
/// `Entity::PLACEHOLDER` (Debt Collector does not inspect it).
pub(super) fn write_bolt_lost(app: &mut App, bolt: Entity) {
    app.world_mut()
        .resource_mut::<Messages<BoltLost>>()
        .write(BoltLost {
            bolt,
            breaker: Entity::PLACEHOLDER,
        });
}

/// Writes a single `BoltImpactCell` message with placeholder `impact_normal`
/// and zero `piercing_remaining` (Debt Collector does not inspect either).
pub(super) fn write_bolt_impact_cell(app: &mut App, bolt: Entity, cell: Entity) {
    app.world_mut()
        .resource_mut::<Messages<BoltImpactCell>>()
        .write(BoltImpactCell {
            cell,
            bolt,
            impact_normal: Vec2::ZERO,
            piercing_remaining: 0,
        });
}

// ── Assertion helpers ───────────────────────────────────────────────────────

/// Returns every captured `DamageDealt<Cell>` whose `source` matches the
/// builder-produced `protocol:debt_collector` source. Isolates Debt
/// Collector's bonus emissions from any other `DamageDealt<Cell>` messages.
pub(super) fn collected_bonus_damage(app: &App) -> Vec<DamageDealt<Cell>> {
    let dc_source = SourceId::protocol(ProtocolKind::DebtCollector).build();
    app.world()
        .resource::<MessageCollector<DamageDealt<Cell>>>()
        .0
        .iter()
        .filter(|msg| msg.source.as_ref() == Some(&dc_source))
        .cloned()
        .collect()
}
