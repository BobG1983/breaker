//! Iron Curtain protocol — production code.
//!
//! Design doc: `docs/design/protocols/iron_curtain.md`.
//!
//! Owns:
//! - [`IronCurtainConfig`] — per-run tuning inserted from
//!   `ProtocolTuning::IronCurtain` at activation time.
//! - [`IRON_CURTAIN_SENTINEL`] — `source_chip` tag stamped on emitted
//!   `DamageDealt<Cell>` messages so downstream stat tracking / FX can
//!   identify Iron Curtain wave damage.
//! - [`activate`] — parses `ProtocolTuning::IronCurtain`, inserts
//!   [`IronCurtainConfig`].
//! - [`register`] — wires [`iron_curtain_on_bolt_lost`] with the correct
//!   schedule, run-ifs, and ordering.
//! - One runtime system: [`iron_curtain_on_bolt_lost`].

use std::marker::PhantomData;

use bevy::{ecs::system::SystemParam, prelude::*};

use crate::{
    bolt::{
        components::BoltBaseDamage, messages::BoltLost, resources::DEFAULT_BOLT_BASE_DAMAGE,
        sets::BoltSystems,
    },
    prelude::*,
    protocol::{
        definition::{ProtocolKind, ProtocolTuning},
        systems::ProtocolGate,
    },
};

// ── Constants ───────────────────────────────────────────────────────────────

/// Sentinel tag stamped into `DamageDealt<Cell>.source` on Iron Curtain
/// wave damage so downstream stat tracking / FX can identify the source.
pub(crate) const IRON_CURTAIN_SENTINEL: &str = "protocol:iron_curtain";

// ── IronCurtainConfig ───────────────────────────────────────────────────────

/// Per-run Iron Curtain tuning extracted from [`ProtocolTuning::IronCurtain`]
/// at activation time.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub(crate) struct IronCurtainConfig {
    /// Fraction of the bolt's base damage dealt at the wave origin (breaker
    /// position). For example, `0.5` means cells within `falloff_start` of
    /// the breaker receive 50% of the bolt's base damage.
    pub(crate) damage_fraction: f32,
    /// Distance from the breaker within which damage is NOT reduced (flat
    /// full-damage zone). Beyond this distance, linear falloff to zero
    /// begins.
    pub(crate) falloff_start:   f32,
}

// ── activate ────────────────────────────────────────────────────────────────

/// Inserts [`IronCurtainConfig`] from `ProtocolTuning::IronCurtain`. Warns
/// and no-ops on a non-IronCurtain tuning variant, leaving any existing
/// `IronCurtainConfig` intact.
pub(crate) fn activate(tuning: &ProtocolTuning, commands: &mut Commands) {
    let ProtocolTuning::IronCurtain {
        damage_fraction,
        falloff_start,
    } = *tuning
    else {
        warn!("iron_curtain::activate called with non-IronCurtain tuning");
        return;
    };
    commands.insert_resource(IronCurtainConfig {
        damage_fraction,
        falloff_start,
    });
}

// ── register ────────────────────────────────────────────────────────────────

/// Registers Iron Curtain's runtime system with the correct schedule,
/// run-ifs, and ordering.
///
/// - [`iron_curtain_on_bolt_lost`] runs in `FixedUpdate` after
///   `BoltSystems::BoltLost`.
/// - Intentionally ungated at the registration level — enforces the
///   `ActiveProtocols` / `NodeState::Playing` gate in-body via an immediate
///   `reader.clear()` + return when inactive. A `.run_if(...)` gate would
///   suppress the system but not advance its `MessageReader` cursor, leaving
///   buffered `BoltLost` messages available for retroactive consumption when
///   the protocol activates on a later frame.
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        iron_curtain_on_bolt_lost.after(BoltSystems::BoltLost),
    );
}

// ── Systems ─────────────────────────────────────────────────────────────────

/// Query alias for alive, non-invulnerable cells with their positions.
/// Factored out to satisfy `clippy::type_complexity`.
type LiveCellQuery<'w, 's> = Query<
    'w,
    's,
    (Entity, &'static Position2D),
    (With<Cell>, Without<Dead>, Without<Invulnerable>),
>;

/// Bundles Iron Curtain's harness-optional resource dependencies behind a
/// single `SystemParam` so `iron_curtain_on_bolt_lost` stays below the
/// `clippy::too_many_arguments` limit after the pre-gate-drain retrofit.
#[derive(SystemParam)]
pub(crate) struct IronCurtainDeps<'w> {
    pub(crate) config:    Option<Res<'w, IronCurtainConfig>>,
    pub(crate) playfield: Option<Res<'w, PlayfieldConfig>>,
}

/// Consumes `BoltLost` messages. For each message, fans out a single-frame
/// damage wave from the breaker position across all alive, non-invulnerable
/// cells, emitting `DamageDealt<Cell>` messages per the design-doc's linear
/// falloff formula.
///
/// Harness-safe: if `IronCurtainConfig` or `PlayfieldConfig` is absent, clears
/// the reader and returns so buffered messages do not leak into a later frame
/// that does have the resources.
///
/// Missing breaker: if `breakers.single()` returns `Err` (0 or 2+ breakers),
/// the iteration `continue`s the `for msg in reader.read()` loop. Because the
/// `continue` sits INSIDE the reader iteration, each iteration still advances
/// the reader's internal cursor — effectively draining every `BoltLost` in
/// the frame without action.
///
/// Bolt base damage: uses the bolt's `BoltBaseDamage` component if present,
/// otherwise falls back to `DEFAULT_BOLT_BASE_DAMAGE` (covers both the
/// missing-component and bolt-despawned-before-this-system cases).
pub(crate) fn iron_curtain_on_bolt_lost(
    mut reader: MessageReader<BoltLost>,
    deps: IronCurtainDeps,
    gate: ProtocolGate,
    breakers: Query<&Position2D, With<Breaker>>,
    bolts: Query<&BoltBaseDamage>,
    cells: LiveCellQuery,
    mut damage_writer: MessageWriter<DamageDealt<Cell>>,
) {
    if gate.is_closed_for(ProtocolKind::IronCurtain) {
        reader.clear();
        return;
    }
    let Some(config) = deps.config else {
        reader.clear();
        return;
    };
    let Some(playfield) = deps.playfield else {
        reader.clear();
        return;
    };
    for msg in reader.read() {
        let Ok(breaker_pos) = breakers.single() else {
            // `continue` here still advances the `reader.read()` iterator,
            // which drains this `BoltLost` message just like `reader.clear()`
            // would. No leakage: by the end of the `for` loop every message
            // in this frame has been consumed, even if every iteration took
            // this branch.
            continue;
        };
        let bolt_base = bolts
            .get(msg.bolt)
            .map_or(DEFAULT_BOLT_BASE_DAMAGE, |b| b.0);
        let wave_origin_damage = bolt_base * config.damage_fraction;
        let max_distance = playfield.height - config.falloff_start;
        for (cell, cell_pos) in cells.iter() {
            let cell_distance = (cell_pos.0.y - breaker_pos.0.y).abs();
            let damage = if cell_distance <= config.falloff_start {
                wave_origin_damage
            } else {
                let falloff_distance = cell_distance - config.falloff_start;
                let falloff_factor = 1.0 - (falloff_distance / max_distance).clamp(0.0, 1.0);
                wave_origin_damage * falloff_factor
            };
            if damage <= 0.0 {
                continue;
            }
            damage_writer.write(DamageDealt::<Cell> {
                dealer:        None,
                attributed_to: None,
                target:        cell,
                amount:        damage,
                source:        Some(SourceId::from(IRON_CURTAIN_SENTINEL)),
                _marker:       PhantomData,
            });
        }
    }
}
