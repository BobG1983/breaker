//! Debt Collector protocol — production code.
//!
//! Design doc: `docs/todos/detail/mod-system-design/protocols/debt_collector.md`.
//!
//! Owns:
//! - [`DebtCollectorConfig`] — per-run tuning inserted from
//!   `ProtocolTuning::DebtCollector` at activation time.
//! - [`DebtStack`] — per-bolt accumulated debt multiplier.
//! - [`DebtCashOut`] — per-bolt single-shot marker carrying the captured stack
//!   value (consumed on the next cell impact).
//! - [`DEBT_COLLECTOR_SENTINEL`] — `source_chip` tag stamped on bonus
//!   `DamageDealt<Cell>` messages.
//! - [`activate`] — parses `ProtocolTuning::DebtCollector`, inserts
//!   `DebtCollectorConfig`.
//! - [`register`] — wires the five runtime systems with schedules, run-ifs,
//!   and ordering constraints.
//! - Five runtime systems: [`debt_collector_on_bump`],
//!   [`debt_collector_on_impact`], [`debt_collector_on_bolt_lost`],
//!   [`debt_collector_attach_stack`], [`debt_collector_cleanup_node`].

use std::marker::PhantomData;

use bevy::prelude::*;

use crate::{
    bolt::{components::BoltBaseDamage, resources::DEFAULT_BOLT_BASE_DAMAGE, sets::BoltSystems},
    breaker::{
        messages::{BumpGrade, BumpPerformed},
        sets::BreakerSystems,
    },
    prelude::*,
    protocol::{
        definition::{ProtocolKind, ProtocolTuning},
        resources::protocol_active,
    },
};

// ── Constants ───────────────────────────────────────────────────────────────

/// Sentinel tag stamped into `DamageDealt<Cell>.source_chip` on cash-out so
/// downstream stat tracking / FX can identify Debt Collector damage.
pub(crate) const DEBT_COLLECTOR_SENTINEL: &str = "protocol:debt_collector";

// ── DebtCollectorConfig ─────────────────────────────────────────────────────

/// Per-run Debt Collector tuning extracted from
/// [`ProtocolTuning::DebtCollector`] at activation time.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub(crate) struct DebtCollectorConfig {
    /// Multiplier added to the stack per Early or Late bump.
    pub(crate) stack_per_bump: f32,
}

// ── DebtStack ───────────────────────────────────────────────────────────────

/// Per-bolt debt multiplier accumulated from Early/Late bumps. Reset on
/// Perfect cash-out or `BoltLost`.
#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub(crate) struct DebtStack(pub(crate) f32);

// ── DebtCashOut ─────────────────────────────────────────────────────────────

/// Per-bolt single-shot marker carrying the captured stack value at the
/// moment of a Perfect bump. Consumed by `debt_collector_on_impact` on the
/// next cell impact.
///
/// Intentionally does NOT implement `Default` — this component is always
/// inserted with an explicit stack value.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub(crate) struct DebtCashOut(pub(crate) f32);

// ── activate ────────────────────────────────────────────────────────────────

/// Inserts [`DebtCollectorConfig`] from `ProtocolTuning::DebtCollector`.
/// Warns and no-ops on a non-DebtCollector tuning variant, leaving any
/// existing `DebtCollectorConfig` intact.
pub(crate) fn activate(tuning: &ProtocolTuning, commands: &mut Commands) {
    let ProtocolTuning::DebtCollector { stack_per_bump } = *tuning else {
        warn!("debt_collector::activate called with non-DebtCollector tuning");
        return;
    };
    commands.insert_resource(DebtCollectorConfig { stack_per_bump });
}

// ── register ────────────────────────────────────────────────────────────────

/// Registers Debt Collector's runtime systems with the correct schedules,
/// run-ifs, and ordering.
///
/// - Three reader systems share both run-ifs:
///   `protocol_active(DebtCollector)` + `in_state(NodeState::Playing)`.
/// - `debt_collector_attach_stack` runs whenever the protocol is active
///   (no `NodeState` gate).
/// - `debt_collector_cleanup_node` runs on `OnExit(NodeState::Playing)`
///   unconditionally (no run-if).
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            debt_collector_on_bump.after(BreakerSystems::GradeBump),
            debt_collector_on_impact.after(BoltSystems::CellCollision),
            debt_collector_on_bolt_lost.after(BoltSystems::BoltLost),
        )
            .run_if(protocol_active(ProtocolKind::DebtCollector))
            .run_if(in_state(NodeState::Playing)),
    );
    app.add_systems(
        FixedUpdate,
        debt_collector_attach_stack.run_if(protocol_active(ProtocolKind::DebtCollector)),
    );
    app.add_systems(OnExit(NodeState::Playing), debt_collector_cleanup_node);
}

// ── Systems ─────────────────────────────────────────────────────────────────

/// Consumes `BumpPerformed` messages and updates per-bolt debt state.
///
/// - `Early` / `Late`: add `config.stack_per_bump` to the bolt's `DebtStack`.
/// - `Perfect`: insert `DebtCashOut(stack)` and reset `DebtStack` to 0.
/// - `msg.bolt == None` (spectator bump): ignored.
/// - Bolt not in query (no `DebtStack` component): skip — `attach_stack`
///   catches it next frame.
///
/// Harness-safe: if `DebtCollectorConfig` is absent, clears the reader and
/// returns so buffered messages do not leak into a later frame that does
/// have the resource.
pub(crate) fn debt_collector_on_bump(
    mut reader: MessageReader<BumpPerformed>,
    config: Option<Res<DebtCollectorConfig>>,
    mut commands: Commands,
    mut bolts: Query<&mut DebtStack>,
) {
    let Some(config) = config else {
        reader.clear();
        return;
    };
    for msg in reader.read() {
        let Some(bolt) = msg.bolt else { continue };
        let Ok(mut stack) = bolts.get_mut(bolt) else {
            continue;
        };
        match msg.grade {
            BumpGrade::Early | BumpGrade::Late => stack.0 += config.stack_per_bump,
            BumpGrade::Perfect => {
                commands.entity(bolt).insert(DebtCashOut(stack.0));
                stack.0 = 0.0;
            }
        }
    }
}

/// Consumes `BoltImpactCell` messages. If the impacting bolt has a
/// `DebtCashOut`, writes a bonus `DamageDealt<Cell>` with
/// `amount = base_damage * cashout.0` and removes the `DebtCashOut` marker
/// (single-shot; only the NEXT cell impact gets the bonus).
///
/// `base_damage` is read from the bolt's optional `BoltBaseDamage` component,
/// falling back to `DEFAULT_BOLT_BASE_DAMAGE` when absent — matches the same
/// derivation used by `bolt_cell_collision`.
///
/// Pierce guard: `bolt_cell_collision` can emit multiple `BoltImpactCell`
/// messages for the same bolt in a single frame (pierce-through). The
/// `cashed_this_frame` vec tracks already-cashed bolts within THIS system
/// invocation so only ONE bonus is emitted per `DebtCashOut`.
pub(crate) fn debt_collector_on_impact(
    mut reader: MessageReader<BoltImpactCell>,
    mut commands: Commands,
    bolts: Query<(&DebtCashOut, Option<&BoltBaseDamage>)>,
    mut damage_writer: MessageWriter<DamageDealt<Cell>>,
) {
    // `commands.entity(...).remove::<DebtCashOut>()` (line 188) is deferred
    // until `apply_deferred`, so subsequent iterations in the same invocation
    // still see `DebtCashOut` on the query. The `cashed_this_frame` guard
    // prevents double-emission on same-frame pierce. Linear scan is fine —
    // pierce counts are bounded by `piercing_remaining` (typically ≤ 4).
    let mut cashed_this_frame: Vec<Entity> = Vec::with_capacity(1);
    for msg in reader.read() {
        if cashed_this_frame.contains(&msg.bolt) {
            continue;
        }
        let Ok((cashout, base_opt)) = bolts.get(msg.bolt) else {
            continue;
        };
        let base_damage = base_opt.map_or(DEFAULT_BOLT_BASE_DAMAGE, |b| b.0);
        let amount = base_damage * cashout.0;
        damage_writer.write(DamageDealt::<Cell> {
            dealer: Some(msg.bolt),
            target: msg.cell,
            amount,
            source_chip: Some(DEBT_COLLECTOR_SENTINEL.into()),
            _marker: PhantomData,
        });
        commands.entity(msg.bolt).remove::<DebtCashOut>();
        cashed_this_frame.push(msg.bolt);
    }
}

/// Consumes `BoltLost` messages. Resets the lost bolt's `DebtStack` to 0
/// and removes any `DebtCashOut` marker. Punishment for losing a bolt
/// mid-build.
///
/// `Commands.entity().remove::<T>()` is a no-op when T isn't present, so
/// unconditional remove is safe.
pub(crate) fn debt_collector_on_bolt_lost(
    mut reader: MessageReader<BoltLost>,
    mut commands: Commands,
    mut bolts: Query<&mut DebtStack>,
) {
    for msg in reader.read() {
        if let Ok(mut stack) = bolts.get_mut(msg.bolt) {
            stack.0 = 0.0;
        }
        commands.entity(msg.bolt).remove::<DebtCashOut>();
    }
}

/// Attaches `DebtStack::default()` to any bolt entity that doesn't already
/// have one. Ensures newly spawned bolts get tracked once Debt Collector is
/// active. Idempotent: `Without<DebtStack>` filter drops already-attached
/// bolts.
pub(crate) fn debt_collector_attach_stack(
    mut commands: Commands,
    new_bolts: Query<Entity, (With<Bolt>, Without<DebtStack>)>,
) {
    for entity in new_bolts.iter() {
        commands.entity(entity).insert(DebtStack::default());
    }
}

/// Query returning every bolt entity that Debt Collector has tagged with
/// either `DebtStack` or `DebtCashOut`. Used by `debt_collector_cleanup_node`
/// on node exit. Aliased to satisfy `clippy::type_complexity`.
type TaggedBoltQuery<'w, 's> = Query<'w, 's, Entity, Or<(With<DebtStack>, With<DebtCashOut>)>>;

/// Runs on `OnExit(NodeState::Playing)`. Removes `DebtStack` and
/// `DebtCashOut` from every bolt so stacks do not persist across nodes.
pub(crate) fn debt_collector_cleanup_node(mut commands: Commands, bolts: TaggedBoltQuery) {
    for entity in bolts.iter() {
        commands
            .entity(entity)
            .remove::<DebtStack>()
            .remove::<DebtCashOut>();
    }
}
