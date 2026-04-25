//! Siphon protocol — production code.
//!
//! Design doc: `docs/design/protocols/siphon.md`.
//!
//! Owns:
//! - [`SiphonConfig`] — per-run tuning (fields in seconds, inserted verbatim
//!   from `ProtocolTuning::Siphon` — no percent translation).
//! - [`SiphonStreak`] — per-run kill-streak tracker. Cleared by
//!   [`siphon_cleanup_node`] on `OnExit(NodeState::Playing)`.
//! - [`activate`] — parses `ProtocolTuning::Siphon`, inserts `SiphonConfig`.
//! - [`register`] — wires the three runtime systems with the design-doc
//!   schedules, run-ifs, and ordering.
//! - [`siphon_on_cell_destroyed`] — consumes `Destroyed<Cell>`, updates
//!   `SiphonStreak`, emits `ReverseTimePenalty` on non-first kills.
//! - [`siphon_tick_streak`] — decrements the streak window each frame, clamps
//!   to zero on expiry and resets `kill_count`.
//! - [`siphon_cleanup_node`] — resets `SiphonStreak` to default on node exit.

use bevy::prelude::*;

use crate::{
    mutators::protocols::{
        definition::{ProtocolKind, ProtocolTuning},
        resources::{ActiveProtocols, protocol_active},
    },
    prelude::*,
    state::run::node::{messages::ReverseTimePenalty, sets::NodeSystems},
};

// ── SiphonConfig ────────────────────────────────────────────────────────────

/// Per-run Siphon tuning extracted from [`ProtocolTuning::Siphon`] at
/// activation time. Fields are in seconds and pass through verbatim from the
/// authoring value (no percent translation — unlike Greed / Sympathy).
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub(crate) struct SiphonConfig {
    /// Seconds a streak remains alive before expiring.
    pub(crate) streak_window: f32,
    /// Seconds added back to the node timer per non-first kill.
    pub(crate) time_per_kill: f32,
}

// ── SiphonStreak ────────────────────────────────────────────────────────────

/// Per-run kill-streak tracker. Global (not per-bolt — all kill sources
/// count). Cleared by [`siphon_cleanup_node`] on node exit.
///
/// `window_remaining == 0.0` AND `kill_count == 0` is the "no active streak"
/// sentinel produced by [`Default`].
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct SiphonStreak {
    /// Seconds remaining in the current streak window. `0.0` = no active
    /// streak.
    pub window_remaining: f32,
    /// Number of kills in the current streak.
    /// `0` = no active streak; `1` = first (silent) kill; `>=2` = each kill
    /// awarded time.
    pub kill_count:       u32,
}

// ── activate ────────────────────────────────────────────────────────────────

/// Inserts [`SiphonConfig`] from `ProtocolTuning::Siphon`. Called by the
/// parent `protocols::activate` dispatch on Siphon pick; last write wins.
/// Warns and no-ops on a non-Siphon tuning variant, leaving any existing
/// `SiphonConfig` intact.
pub(crate) fn activate(tuning: &ProtocolTuning, commands: &mut Commands) {
    let ProtocolTuning::Siphon {
        streak_window,
        time_per_kill,
    } = *tuning
    else {
        warn!("siphon::activate called with non-Siphon tuning");
        return;
    };
    commands.insert_resource(SiphonConfig {
        streak_window,
        time_per_kill,
    });
}

// ── register ────────────────────────────────────────────────────────────────

/// Registers Siphon's runtime systems.
///
/// - `siphon_tick_streak` → `FixedUpdate`, before `siphon_on_cell_destroyed`,
///   under `protocol_active(Siphon)` + `in_state(NodeState::Playing)`.
/// - `siphon_on_cell_destroyed` → `FixedUpdate`, before
///   `NodeSystems::ApplyTimePenalty`, intentionally ungated at the
///   registration level. Enforces the `ActiveProtocols` /
///   `NodeState::Playing` gate in-body via an immediate `reader.clear()` +
///   return when inactive so buffered `Destroyed<Cell>` messages cannot
///   leak retroactively when the protocol activates on a later frame.
/// - `siphon_cleanup_node` → `OnExit(NodeState::Playing)` with NO run-if.
///
/// NOTE: `SiphonStreak` is NOT inserted here; the plugin owns
/// `init_resource::<SiphonStreak>()`. This keeps the `Option<ResMut<...>>`
/// guard path exercisable by isolated harness configurations.
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        siphon_tick_streak
            .before(siphon_on_cell_destroyed)
            .run_if(protocol_active(ProtocolKind::Siphon))
            .run_if(in_state(NodeState::Playing)),
    );
    app.add_systems(
        FixedUpdate,
        siphon_on_cell_destroyed.before(NodeSystems::ApplyTimePenalty),
    );
    app.add_systems(OnExit(NodeState::Playing), siphon_cleanup_node);
}

// ── Systems ─────────────────────────────────────────────────────────────────

/// Consumes `Destroyed<Cell>`, updates [`SiphonStreak`], emits
/// `ReverseTimePenalty` for every non-first kill.
///
/// Harness-safe: clears the reader and early-returns when `SiphonStreak` or
/// `SiphonConfig` is absent so buffered messages don't leak into a later frame
/// that does have the resources.
pub(crate) fn siphon_on_cell_destroyed(
    mut reader: MessageReader<Destroyed<Cell>>,
    active_protocols: Option<Res<ActiveProtocols>>,
    node_state: Option<Res<State<NodeState>>>,
    streak: Option<ResMut<SiphonStreak>>,
    config: Option<Res<SiphonConfig>>,
    mut penalty_writer: MessageWriter<ReverseTimePenalty>,
) {
    if active_protocols
        .as_ref()
        .is_none_or(|ap| !ap.contains(ProtocolKind::Siphon))
        || node_state
            .as_ref()
            .is_none_or(|s| *s.get() != NodeState::Playing)
    {
        reader.clear();
        return;
    }
    let Some(mut streak) = streak else {
        reader.clear();
        return;
    };
    let Some(config) = config else {
        reader.clear();
        return;
    };
    for _msg in reader.read() {
        if streak.window_remaining <= 0.0 {
            // Starting a new streak — first kill, no time added.
            streak.kill_count = 1;
            streak.window_remaining = config.streak_window;
        } else {
            // Continuing — each subsequent kill HARD-SETS the window and adds
            // time.
            streak.kill_count = streak.kill_count.saturating_add(1);
            streak.window_remaining = config.streak_window;
            penalty_writer.write(ReverseTimePenalty {
                seconds: config.time_per_kill,
            });
        }
    }
}

/// Decrements `SiphonStreak.window_remaining` by `time.delta_secs()`; resets
/// both fields to zero on expiry.
///
/// Harness-safe: early-returns when `SiphonStreak` or `SiphonConfig` is
/// absent. The `_config` binding enforces "protocol not activated ⇒ no tick"
/// symmetrically with `siphon_on_cell_destroyed`.
pub(crate) fn siphon_tick_streak(
    time: Res<Time>,
    streak: Option<ResMut<SiphonStreak>>,
    config: Option<Res<SiphonConfig>>,
) {
    let Some(mut streak) = streak else { return };
    let Some(_config) = config else { return };
    if streak.window_remaining > 0.0 {
        streak.window_remaining -= time.delta_secs();
        if streak.window_remaining <= 0.0 {
            streak.window_remaining = 0.0;
            streak.kill_count = 0;
        }
    }
}

/// Resets `SiphonStreak` to default on `OnExit(NodeState::Playing)`.
///
/// Harness-safe: plain early-return when `SiphonStreak` is absent. Does NOT
/// insert the resource if it's missing — state never leaks across nodes, but
/// a harness without the resource stays without it.
pub(crate) fn siphon_cleanup_node(streak: Option<ResMut<SiphonStreak>>) {
    let Some(mut streak) = streak else { return };
    *streak = SiphonStreak::default();
}
