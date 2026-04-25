//! Greed protocol — production implementation.
//!
//! Design doc: `docs/design/protocols/greed.md`.
//!
//! Owns:
//! - [`GreedConfig`] — per-run tuning (percent-unit `rarity_boost_per_skip`,
//!   translated from the fractional authoring value at activation time).
//! - [`GreedStacks`] — per-run counter of chip-offer skips, cleared by
//!   `reset_run_state`.
//! - [`activate`] — parses `ProtocolTuning::Greed`, inserts `GreedConfig`.
//! - [`register`] — wires [`greed_on_skip`] ungated; the system enforces the
//!   `ActiveProtocols` gate in-body via `reader.clear()` + return when Greed
//!   is inactive so buffered `ChipOfferSkipped` messages cannot leak across
//!   runs.
//! - [`greed_on_skip`] — increments `GreedStacks.skips` per `ChipOfferSkipped`.
//! - [`apply_greed_boost`] — re-weights a rarity map in place (called by
//!   `generate_chip_offerings`).

use std::collections::HashMap;

use bevy::prelude::*;

use crate::{
    chips::definition::Rarity,
    mutators::protocols::{
        definition::{ProtocolKind, ProtocolTuning},
        resources::ActiveProtocols,
    },
    state::run::chip_select::messages::ChipOfferSkipped,
};

/// Per-run Greed tuning extracted from `ProtocolTuning::Greed` at activation
/// time. Translates from the fractional authoring field (`0.05` = 5%) to
/// percent units (`5.0` = 5%) via `* 100.0` at activation — matches the
/// Tether convention so design-doc formulas read in percent.
///
/// Inserted by [`activate`]; cleared implicitly when `ActiveProtocols` is
/// cleared via `reset_run_state` (the run-if gate hides Greed's systems
/// outside an active run).
#[derive(Resource, Debug, Clone, Copy)]
pub struct GreedConfig {
    /// Rarity boost percent per skip. 5.0 means +5% weight shift per skip.
    pub rarity_boost_per_skip: f32,
}

/// Per-run count of chip-offer skips. Inserted via `init_resource` in
/// [`register`] (default = 0); incremented by [`greed_on_skip`] on each
/// `ChipOfferSkipped` message; read by [`apply_greed_boost`] during chip
/// offering generation. Cleared by `reset_run_state` as part of
/// `RunInventories::clear_all`.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct GreedStacks {
    /// Number of chip offers the player has skipped this run.
    pub skips: u32,
}

impl GreedStacks {
    /// Total rarity boost percent (units matching [`GreedConfig`]).
    /// Returns 0.0 when `skips == 0`. Takes `self` + `config` by value —
    /// both are `Copy` and small (4 bytes each); clippy's
    /// `trivially_copy_pass_by_ref` prefers the by-value form.
    #[must_use]
    pub fn rarity_boost(self, config: GreedConfig) -> f32 {
        self.skips as f32 * config.rarity_boost_per_skip
    }
}

/// Inserts [`GreedConfig`] from `ProtocolTuning::Greed`, translating the
/// fractional `rarity_boost_per_skip` to percent units (× 100) at activation
/// time. Called by the parent `protocols::activate` dispatch on Greed pick;
/// last write wins. Warns and no-ops on a non-Greed tuning, leaving any
/// existing `GreedConfig` intact.
pub(crate) fn activate(tuning: &ProtocolTuning, commands: &mut Commands) {
    let ProtocolTuning::Greed {
        rarity_boost_per_skip,
    } = *tuning
    else {
        warn!("greed::activate called with non-Greed tuning");
        return;
    };
    commands.insert_resource(GreedConfig {
        rarity_boost_per_skip: rarity_boost_per_skip * 100.0,
    });
}

/// Registers Greed's runtime systems and the per-run stacks resource.
///
/// - `GreedStacks` initialised via `init_resource` (default = 0).
/// - `greed_on_skip` → `Update`, intentionally ungated. The system enforces
///   the `ActiveProtocols` gate in-body via `reader.clear()` + return when
///   Greed is inactive.
pub(crate) fn register(app: &mut App) {
    // NOTE: `GreedStacks` is NOT inserted here. The plugin is responsible for
    // `init_resource::<GreedStacks>()`. `register` only wires the runtime system
    // so tests that exercise isolated harness configurations (without
    // `GreedStacks`) exercise the `Option<ResMut<_>>` guard path without the
    // resource being silently inserted by this function.
    //
    // Defensively register `ChipOfferSkipped`. The chip-select plugin owns
    // canonical registration (see
    // `breaker-game/src/state/run/chip_select/plugin.rs`), but since the
    // retrofit removed the `.run_if(protocol_active(Greed))` gate from
    // `greed_on_skip`, Bevy now validates `MessageReader<ChipOfferSkipped>`
    // every tick — so any app that wires `ProtocolPlugin` without the
    // chip-select plugin would panic with "Message not initialized".
    // `add_message` is idempotent; calling twice is safe.
    app.add_message::<ChipOfferSkipped>();
    app.add_systems(Update, greed_on_skip);
}

/// Reads `ChipOfferSkipped` messages and increments `GreedStacks.skips` by
/// one per message. Runs in `Update` while `Greed` is in `ActiveProtocols`.
///
/// The `stacks` parameter is `Option<ResMut<GreedStacks>>` for harness-safety:
/// tests construct some `App`s without `GreedStacks` installed, and the
/// system must not panic. On `None`, clears the reader (to prevent message
/// buildup) and returns.
pub(crate) fn greed_on_skip(
    mut reader: MessageReader<ChipOfferSkipped>,
    active_protocols: Option<Res<ActiveProtocols>>,
    stacks: Option<ResMut<GreedStacks>>,
) {
    if active_protocols
        .as_ref()
        .is_none_or(|ap| !ap.contains(ProtocolKind::Greed))
    {
        reader.clear();
        return;
    }
    let Some(mut stacks) = stacks else {
        reader.clear();
        return;
    };
    for _ in reader.read() {
        stacks.skips = stacks.skips.saturating_add(1);
    }
}

/// Re-weights the rarity map to favour higher rarities when the player has
/// accumulated Greed skips. Leaves the map untouched when boost is 0.
///
/// Algorithm:
/// - `boost = stacks.rarity_boost(config)` (percent units).
/// - If `boost <= 0.0`, return immediately.
/// - `common_original = weights[Common]` (0.0 if missing — defensive).
/// - `common_floor = common_original * 0.10` (Common weight never drops below
///   10% of its original value).
/// - `common_new = (common_original - boost).max(common_floor)`.
/// - `removed = common_original - common_new`.
/// - `weights[Common] = common_new`.
/// - Add `removed * 0.5` to `Uncommon` (via `entry().and_modify`).
/// - Add `removed * 0.5` to `Rare` (via `entry().and_modify`).
///
/// The `entry().and_modify` form intentionally only adds to entries that
/// already exist; missing tiers are left missing (no insert).
pub(crate) fn apply_greed_boost(
    rarity_weights: &mut HashMap<Rarity, f32>,
    stacks: GreedStacks,
    config: GreedConfig,
) {
    let boost = stacks.rarity_boost(config);
    if boost <= 0.0 {
        return;
    }
    let common_original = rarity_weights.get(&Rarity::Common).copied().unwrap_or(0.0);
    let common_floor = common_original * 0.10;
    let common_new = (common_original - boost).max(common_floor);
    let removed = common_original - common_new;
    rarity_weights.insert(Rarity::Common, common_new);
    rarity_weights
        .entry(Rarity::Uncommon)
        .and_modify(|v| *v += removed * 0.5);
    rarity_weights
        .entry(Rarity::Rare)
        .and_modify(|v| *v += removed * 0.5);
}
