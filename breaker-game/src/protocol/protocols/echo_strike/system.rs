//! Echo Strike protocol — production scaffold.
//!
//! Design doc: `docs/todos/detail/mod-system-design/protocols/echo_strike.md`.
//!
//! Owns:
//! - [`ECHO_STRIKE_SENTINEL`] — `source_chip` tag stamped on echo-damage
//!   `DamageDealt<Cell>` messages.
//! - [`EchoStrikeConfig`] — per-run tuning inserted from
//!   `ProtocolTuning::EchoStrike` at activation time.
//! - [`EchoNetwork`] — per-bolt FIFO echo deque.
//! - [`EchoPrimed`] — per-bolt single-shot "next impact echoes" marker.
//! - [`activate`] — parses `ProtocolTuning::EchoStrike`, inserts
//!   `EchoStrikeConfig`.
//! - [`register`] — wires the four runtime systems with schedules, run-ifs,
//!   and ordering constraints.
//!
//! Systems:
//! - [`echo_strike_on_bump`] — primes the bolt on Perfect bumps.
//! - [`echo_strike_on_impact`] — emits echo damage, maintains the FIFO
//!   network, and consumes the `EchoPrimed` marker.
//! - [`echo_strike_cleanup_destroyed_echoes`] — evicts destroyed cells from
//!   every bolt's `EchoNetwork`.
//! - [`echo_strike_cleanup_node`] — removes `EchoNetwork` and `EchoPrimed`
//!   from every bolt on `OnExit(NodeState::Playing)`.

use std::{collections::VecDeque, marker::PhantomData};

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
        resources::ActiveProtocols,
    },
    shared::death_pipeline::sets::DeathPipelineSystems,
};

// ── Constants ───────────────────────────────────────────────────────────────

/// Sentinel tag stamped into `DamageDealt<Cell>.source_chip` on every
/// echo-damage message so downstream stat tracking / FX can identify Echo
/// Strike damage.
pub(crate) const ECHO_STRIKE_SENTINEL: &str = "protocol:echo_strike";

// ── EchoStrikeConfig ────────────────────────────────────────────────────────

/// Per-run Echo Strike tuning extracted from
/// [`ProtocolTuning::EchoStrike`] at activation time.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub(crate) struct EchoStrikeConfig {
    /// Maximum number of echoes tracked per bolt.
    pub(crate) max_echoes:      u32,
    /// Fraction of impact damage dealt to the newest echo.
    pub(crate) newest_fraction: f32,
    /// Fraction of impact damage dealt to the middle echo (3-echo networks only).
    pub(crate) middle_fraction: f32,
    /// Fraction of impact damage dealt to the oldest echo.
    pub(crate) oldest_fraction: f32,
}

// ── EchoNetwork ─────────────────────────────────────────────────────────────

/// Per-bolt FIFO echo deque. Front = oldest, back = newest. Maximum length
/// enforced against `EchoStrikeConfig.max_echoes` at push time by
/// `echo_strike_on_impact`.
#[derive(Component, Debug, Default, Clone)]
pub(crate) struct EchoNetwork {
    /// Tracked echoes; front is oldest, back is newest.
    pub(crate) echoes: VecDeque<Entity>,
}

// ── EchoPrimed ──────────────────────────────────────────────────────────────

/// Per-bolt single-shot marker inserted by a Perfect bump and consumed by
/// the next `BoltImpactCell`. No fields.
#[derive(Component, Debug)]
pub(crate) struct EchoPrimed;

// ── activate ────────────────────────────────────────────────────────────────

/// Inserts [`EchoStrikeConfig`] from `ProtocolTuning::EchoStrike`. Warns and
/// no-ops on a non-EchoStrike tuning variant, leaving any existing
/// `EchoStrikeConfig` intact.
pub(crate) fn activate(tuning: &ProtocolTuning, commands: &mut Commands) {
    let ProtocolTuning::EchoStrike {
        max_echoes,
        newest_fraction,
        middle_fraction,
        oldest_fraction,
    } = *tuning
    else {
        warn!("echo_strike::activate called with non-EchoStrike tuning");
        return;
    };
    commands.insert_resource(EchoStrikeConfig {
        max_echoes,
        newest_fraction,
        middle_fraction,
        oldest_fraction,
    });
}

// ── register ────────────────────────────────────────────────────────────────

/// Registers Echo Strike's runtime systems with the correct schedules and
/// ordering.
///
/// All `FixedUpdate` reader systems are intentionally ungated at the
/// registration level. Each enforces the `ActiveProtocols` /
/// `NodeState::Playing` gate in-body via an immediate `reader.clear()` +
/// return when inactive, draining the buffer every tick so buffered
/// messages cannot leak retroactively when the protocol activates on a
/// later frame.
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (
            echo_strike_on_bump.after(BreakerSystems::GradeBump),
            echo_strike_on_impact.after(BoltSystems::CellCollision),
            echo_strike_cleanup_destroyed_echoes.after(DeathPipelineSystems::HandleKill),
        ),
    );
    app.add_systems(OnExit(NodeState::Playing), echo_strike_cleanup_node);
}

// ── Systems ─────────────────────────────────────────────────────────────────

/// Consumes `BumpPerformed` messages. On a Perfect bump with a known bolt,
/// inserts `EchoPrimed` on the bolt so the next `BoltImpactCell` echoes.
/// Early/Late bumps and spectator bumps (`bolt == None`) are ignored.
///
/// Idempotent: re-inserting `EchoPrimed` on an already-primed bolt is a
/// no-op (Bevy component inserts overwrite).
///
/// Harness-safe: if `EchoStrikeConfig` is absent, clears the reader and
/// returns so buffered messages don't leak into a later frame that does
/// have the resource.
pub(crate) fn echo_strike_on_bump(
    mut reader: MessageReader<BumpPerformed>,
    config: Option<Res<EchoStrikeConfig>>,
    active_protocols: Option<Res<ActiveProtocols>>,
    node_state: Option<Res<State<NodeState>>>,
    mut commands: Commands,
) {
    if active_protocols
        .as_ref()
        .is_none_or(|ap| !ap.contains(ProtocolKind::EchoStrike))
        || node_state
            .as_ref()
            .is_none_or(|s| *s.get() != NodeState::Playing)
    {
        reader.clear();
        return;
    }
    if config.is_none() {
        reader.clear();
        return;
    }
    for msg in reader.read() {
        if msg.grade != BumpGrade::Perfect {
            continue;
        }
        let Some(bolt) = msg.bolt else { continue };
        // `get_entity` returns Err if the entity is despawned — silently skip
        // rather than panicking at command-flush time.
        if let Ok(mut entity) = commands.get_entity(bolt) {
            entity.insert(EchoPrimed);
        }
    }
}

/// Consumes `BoltImpactCell` messages. On a primed bolt, emits echo
/// `DamageDealt<Cell>` to every existing echo (falloff assigned by position),
/// pushes the new impact onto the FIFO network (dedup + max-length eviction),
/// and removes the `EchoPrimed` marker.
///
/// Non-primed bolts are ignored entirely — no echo emit, no network mutation.
///
/// Pierce guard: `bolt_cell_collision` can emit multiple `BoltImpactCell`
/// messages for the same bolt in a single frame (pierce-through).
/// `EchoPrimed` removal is deferred via `commands`, so subsequent iterations
/// in the same invocation would otherwise see `has_primed == true` again.
/// `processed_this_frame` tracks already-processed bolts within THIS system
/// invocation so only ONE echo burst is emitted per prime.
///
/// Harness-safe: if `EchoStrikeConfig` is absent, clears the reader and
/// returns.
pub(crate) fn echo_strike_on_impact(
    mut reader: MessageReader<BoltImpactCell>,
    config: Option<Res<EchoStrikeConfig>>,
    active_protocols: Option<Res<ActiveProtocols>>,
    node_state: Option<Res<State<NodeState>>>,
    mut commands: Commands,
    bolts: Query<(
        Option<&BoltBaseDamage>,
        Option<&EchoNetwork>,
        Has<EchoPrimed>,
    )>,
    mut damage_writer: MessageWriter<DamageDealt<Cell>>,
) {
    if active_protocols
        .as_ref()
        .is_none_or(|ap| !ap.contains(ProtocolKind::EchoStrike))
        || node_state
            .as_ref()
            .is_none_or(|s| *s.get() != NodeState::Playing)
    {
        reader.clear();
        return;
    }
    let Some(config) = config else {
        reader.clear();
        return;
    };
    let mut processed_this_frame: Vec<Entity> = Vec::with_capacity(1);
    for msg in reader.read() {
        if processed_this_frame.contains(&msg.bolt) {
            continue;
        }
        let Ok((base_opt, net_opt, has_primed)) = bolts.get(msg.bolt) else {
            continue;
        };
        if !has_primed {
            continue;
        }
        let impact_damage = base_opt.map_or(DEFAULT_BOLT_BASE_DAMAGE, |b| b.0);

        // Build the working deque. Damage is emitted against the PRE-mutation
        // snapshot, and the POST-mutation deque is written back below.
        let mut deque: VecDeque<Entity> = net_opt.map_or_else(VecDeque::new, |n| n.echoes.clone());

        // Emit echo damage based on the PRE-mutation deque. Falloff is
        // assigned by position, not by fixed slot — a 2-echo network skips
        // `middle_fraction`.
        let fractions: &[f32] = match deque.len() {
            0 => &[],
            1 => &[config.newest_fraction],
            2 => &[config.oldest_fraction, config.newest_fraction],
            _ => &[
                config.oldest_fraction,
                config.middle_fraction,
                config.newest_fraction,
            ],
        };
        for (echo, fraction) in deque.iter().zip(fractions.iter()) {
            let amount = impact_damage * fraction;
            if amount > 0.0 {
                damage_writer.write(DamageDealt::<Cell> {
                    dealer: Some(msg.bolt),
                    target: *echo,
                    amount,
                    source_chip: Some(ECHO_STRIKE_SENTINEL.into()),
                    _marker: PhantomData,
                });
            }
        }

        // Mutate: dedup then push; FIFO-evict if over the cap.
        deque.retain(|&e| e != msg.cell);
        deque.push_back(msg.cell);
        if deque.len() as u32 > config.max_echoes {
            deque.pop_front();
        }

        commands
            .entity(msg.bolt)
            .insert(EchoNetwork { echoes: deque })
            .remove::<EchoPrimed>();
        processed_this_frame.push(msg.bolt);
    }
}

/// Consumes `Destroyed<Cell>` messages. Removes every destroyed cell entity
/// from every bolt's `EchoNetwork` deque so dangling `Entity` handles don't
/// accumulate. No damage emit, no `EchoPrimed` change. Bolts without an
/// `EchoNetwork` are simply not in the query — tolerated.
///
/// Harness-safe: if `EchoStrikeConfig` is absent, clears the reader and
/// returns.
pub(crate) fn echo_strike_cleanup_destroyed_echoes(
    mut reader: MessageReader<Destroyed<Cell>>,
    config: Option<Res<EchoStrikeConfig>>,
    active_protocols: Option<Res<ActiveProtocols>>,
    node_state: Option<Res<State<NodeState>>>,
    mut networks: Query<&mut EchoNetwork>,
) {
    if active_protocols
        .as_ref()
        .is_none_or(|ap| !ap.contains(ProtocolKind::EchoStrike))
        || node_state
            .as_ref()
            .is_none_or(|s| *s.get() != NodeState::Playing)
    {
        reader.clear();
        return;
    }
    if config.is_none() {
        reader.clear();
        return;
    }
    let victims: Vec<Entity> = reader.read().map(|msg| msg.victim).collect();
    if victims.is_empty() {
        return;
    }
    for mut net in &mut networks {
        net.echoes.retain(|e| !victims.contains(e));
    }
}

/// Query returning every bolt entity that Echo Strike has tagged with
/// either `EchoNetwork` or `EchoPrimed`. Note the `Or<...>` filter — a bolt
/// matches if it has EITHER component (a bolt can be primed without a
/// network, or carry a network without being primed). Used by
/// `echo_strike_cleanup_node` on node exit. Type alias satisfies
/// `clippy::type_complexity`.
type EchoTaggedBoltQuery<'w, 's> = Query<'w, 's, Entity, Or<(With<EchoNetwork>, With<EchoPrimed>)>>;

/// Runs on `OnExit(NodeState::Playing)`. Removes `EchoNetwork` and
/// `EchoPrimed` from every bolt so echoes do not persist across nodes.
///
/// NOTE: This is implemented (not a stub) because test fixtures exercise
/// the cleanup path directly — e.g. re-entry tests rely on the cleanup
/// actually firing on exit to prove echoes don't leak across a node.
pub(crate) fn echo_strike_cleanup_node(mut commands: Commands, bolts: EchoTaggedBoltQuery) {
    for entity in bolts.iter() {
        commands
            .entity(entity)
            .remove::<EchoNetwork>()
            .remove::<EchoPrimed>();
    }
}
