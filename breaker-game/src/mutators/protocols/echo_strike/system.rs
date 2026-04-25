//! Echo Strike protocol — production scaffold.
//!
//! Design doc: `docs/design/protocols/echo_strike.md`.
//!
//! Owns:
//! - [`EchoStrikeConfig`] — per-run tuning inserted from
//!   `ProtocolTuning::EchoStrike` at activation time.
//! - [`EchoNetwork`] — per-bolt FIFO echo deque.
//! - [`EchoPrimed`] — per-bolt single-shot "next impact echoes" marker.
//! - [`activate`] — parses `ProtocolTuning::EchoStrike`, inserts
//!   `EchoStrikeConfig`.
//! - [`wire`] — wires the three runtime systems with schedules, run-ifs,
//!   and ordering constraints.
//!
//! Systems:
//! - [`echo_strike_on_bump`] — primes the bolt on Perfect bumps.
//! - [`echo_strike_emit_siblings`] — runs in `DmgSystems::PostApplyDamage`; reads
//!   post-apply `DamageDealt<Cell>` messages, resolves the candidate bolt
//!   via `msg.dealer.or(msg.attributed_to)`, consumes `EchoPrimed`, emits
//!   echo siblings with `source = "protocol:echo_strike"`, and mutates the
//!   bolt's `EchoNetwork`.
//! - [`echo_strike_cleanup_destroyed_echoes`] — evicts destroyed cells from
//!   every bolt's `EchoNetwork`.
//! - [`echo_strike_cleanup_node`] — removes `EchoNetwork` and `EchoPrimed`
//!   from every bolt on `OnExit(NodeState::Playing)`.

use std::{collections::VecDeque, marker::PhantomData};

use bevy::{ecs::message::MessageCursor, prelude::*};

use crate::{
    breaker::{
        messages::{BumpGrade, BumpPerformed},
        sets::BreakerSystems,
    },
    mutators::protocols::{
        definition::{ProtocolKind, ProtocolTuning},
        resources::{ActiveProtocols, protocol_active},
    },
    prelude::*,
};

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
/// `echo_strike_emit_siblings`.
#[derive(Component, Debug, Default, Clone)]
pub struct EchoNetwork {
    /// Tracked echoes; front is oldest, back is newest.
    pub echoes: VecDeque<Entity>,
}

// ── EchoPrimed ──────────────────────────────────────────────────────────────

/// Per-bolt single-shot marker inserted by a Perfect bump and consumed by
/// the next `DamageDealt<Cell>` that the bolt participates in. No fields.
#[derive(Component, Debug)]
pub struct EchoPrimed;

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

// ── wire ────────────────────────────────────────────────────────────────

/// Registers Echo Strike's runtime systems with the correct schedules and
/// ordering.
///
/// `echo_strike_on_bump` stays in-body-gated (drains the reader on inactive
/// state to prevent buffered-message leak). `echo_strike_emit_siblings` uses
/// `.run_if(protocol_active(ProtocolKind::EchoStrike))` and
/// `.run_if(in_state(NodeState::Playing))` as first-class run-conditions
/// because it drives off `ResMut<Messages<DamageDealt<Cell>>>` rather than a
/// dedicated reader.
pub(crate) fn wire(app: &mut App) {
    // Deliberate late emitter (`echo_strike_emit_siblings`): reads
    // DamageDealt<Cell> / Dead state from the current tick to cascade
    // follow-up damage. MUST stay in DmgSystems::PostApplyDamage so it runs
    // after the primary damage emitters in DmgSystems::EmitDamage and the
    // applicators in DmgSystems::ApplyDamage.
    app.add_systems(
        FixedUpdate,
        (
            echo_strike_on_bump.after(BreakerSystems::GradeBump),
            echo_strike_emit_siblings
                .in_set(DmgSystems::PostApplyDamage)
                .in_set(crate::game::PostApplyRipple::EchoStrike)
                .run_if(protocol_active(ProtocolKind::EchoStrike))
                .run_if(in_state(NodeState::Playing)),
            echo_strike_cleanup_destroyed_echoes.after(DmgSystems::ApplyKill),
        ),
    );
    app.add_systems(OnExit(NodeState::Playing), echo_strike_cleanup_node);
}

// ── Systems ─────────────────────────────────────────────────────────────────

/// Consumes `BumpPerformed` messages. On a Perfect bump with a known bolt,
/// inserts `EchoPrimed` on the bolt so the next `DamageDealt<Cell>`
/// participated-in by the bolt triggers echo emission. Early/Late bumps and
/// spectator bumps (`bolt == None`) are ignored.
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

type EchoBoltQuery<'w, 's> = Query<'w, 's, Option<&'static EchoNetwork>, With<EchoPrimed>>;

/// `DmgSystems::PostApplyDamage` — emits echo sibling `DamageDealt<Cell>` messages
/// from an already-applied primary message.
///
/// Rules:
/// - Skip messages whose `source` already carries the
///   `protocol:echo_strike` source (loop protection).
/// - Skip messages with `amount <= 0.0` — the invulnerable-filter zeroed
///   the primary; enforces the unified "invulnerable source → no ripple"
///   rule alongside diffusion and tether.
/// - Resolve the candidate bolt entity via `msg.dealer.or(msg.attributed_to)`.
///   This lets echoes fire on BOTH primary bump-damage (`dealer = Some(bolt)`)
///   AND ripple damage (`dealer = None, attributed_to = Some(bolt)`) from
///   diffusion / tether.
/// - Skip when the resolved bolt is `None`, not queryable, or not primed.
/// - Skip when the bolt was already processed this tick (pierce guard).
/// - Emit one `DamageDealt<Cell>` per existing echo in the bolt's network,
///   with amount = `msg.amount * fraction` and `source = "protocol:echo_strike"`.
///   Fractions follow the deque-size rule:
///   * 0 echoes → no emits
///   * 1 echo  → `newest_fraction`
///   * 2 echoes → `[oldest_fraction, newest_fraction]`
///   * 3+      → `[oldest, middle, newest]`
/// - Mutate the `EchoNetwork` FIFO: dedup then `push_back(msg.target)`; `pop_front`
///   if over `max_echoes`.
/// - Remove `EchoPrimed` from the bolt.
pub(crate) fn echo_strike_emit_siblings(
    config: Option<Res<EchoStrikeConfig>>,
    mut cursor: Local<MessageCursor<DamageDealt<Cell>>>,
    mut messages: ResMut<Messages<DamageDealt<Cell>>>,
    bolts: EchoBoltQuery,
    mut commands: Commands,
) {
    let Some(config) = config else { return };

    let echo_source = SourceId::protocol(ProtocolKind::EchoStrike).build();

    // Snapshot every unread `DamageDealt<Cell>` message via a local cursor.
    // Mirrors `tether_emit_partner`'s pattern: `MessageCursor` spans both
    // buffers without consuming them, and the `Local` advances the reader
    // position each tick so echoed siblings don't re-echo.
    let snapshot: Vec<DamageDealt<Cell>> = cursor.read(&*messages).cloned().collect();

    let mut processed_this_frame: Vec<Entity> = Vec::new();

    for msg in snapshot {
        if msg.source.as_ref() == Some(&echo_source) {
            continue;
        }
        if msg.amount <= 0.0 {
            continue;
        }
        let Some(bolt) = msg.dealer.or(msg.attributed_to) else {
            continue;
        };
        if processed_this_frame.contains(&bolt) {
            continue;
        }
        let Ok(net_opt) = bolts.get(bolt) else {
            continue;
        };

        // Build the working deque (empty if the bolt lacks EchoNetwork).
        let mut deque: VecDeque<Entity> = net_opt.map_or_else(VecDeque::new, |n| n.echoes.clone());

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

        let emits: Vec<(Entity, f32)> = deque
            .iter()
            .zip(fractions.iter())
            .filter_map(|(echo, fraction)| {
                let amount = msg.amount * fraction;
                if amount > 0.0 {
                    Some((*echo, amount))
                } else {
                    None
                }
            })
            .collect();

        for (target, amount) in emits {
            messages.write(DamageDealt::<Cell> {
                dealer: None,
                attributed_to: msg.attributed_to.or(msg.dealer),
                target,
                amount,
                source: Some(echo_source.clone()),
                _marker: PhantomData,
            });
        }

        // Mutate the FIFO: dedup, push, FIFO-evict.
        deque.retain(|&e| e != msg.target);
        deque.push_back(msg.target);
        if deque.len() as u32 > config.max_echoes {
            deque.pop_front();
        }

        commands
            .entity(bolt)
            .insert(EchoNetwork { echoes: deque })
            .remove::<EchoPrimed>();
        processed_this_frame.push(bolt);
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
pub(crate) fn echo_strike_cleanup_node(mut commands: Commands, bolts: EchoTaggedBoltQuery) {
    for entity in &bolts {
        commands
            .entity(entity)
            .remove::<EchoNetwork>()
            .remove::<EchoPrimed>();
    }
}
