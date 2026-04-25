//! Shared test fixtures for Greed protocol tests.
//!
//! App builders, `activate_now` (CommandQueue-flush pattern), `ChipOfferSkipped`
//! writers, `GreedConfig` / `GreedStacks` / `ActiveProtocols` installers.

#![allow(
    dead_code,
    reason = "shared helpers; not every test file uses every helper"
)]

use bevy::{
    ecs::{message::Messages, world::CommandQueue},
    prelude::*,
};

use super::super::system::{GreedConfig, GreedStacks, activate, register};
use crate::{
    mutators::protocols::{
        definition::{ProtocolDefinition, ProtocolKind, ProtocolTuning},
        resources::ActiveProtocols,
    },
    prelude::*,
    state::run::chip_select::messages::ChipOfferSkipped,
};

// ── App builders ────────────────────────────────────────────────────────────

/// Default Greed test app. Registers `ActiveProtocols`, `GreedStacks`,
/// `ChipOfferSkipped` message, then calls `greed::register`.
pub(super) fn build_greed_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_resource::<ActiveProtocols>()
        .with_resource::<GreedStacks>()
        .with_message::<ChipOfferSkipped>()
        .build();
    register(&mut app);
    app
}

/// Same as `build_greed_app` but without initializing `GreedStacks` as a
/// resource. Used to test the harness-safe `Option<ResMut<GreedStacks>>`
/// guard path.
pub(super) fn build_greed_app_no_stacks() -> App {
    let mut app = TestAppBuilder::new()
        .with_resource::<ActiveProtocols>()
        .with_message::<ChipOfferSkipped>()
        .build();
    register(&mut app);
    app
}

// ── Config / stack / active helpers ─────────────────────────────────────────

/// Canonical Greed config matching `greed.protocol.ron` after the `* 100.0`
/// activation translation: `rarity_boost_per_skip: 5.0`.
pub(super) const fn canonical_greed_config() -> GreedConfig {
    GreedConfig {
        rarity_boost_per_skip: 5.0,
    }
}

/// Inserts a `GreedConfig` as a resource.
pub(super) fn install_greed_config(app: &mut App, cfg: GreedConfig) {
    app.world_mut().insert_resource(cfg);
}

/// Replaces `GreedStacks` with `GreedStacks { skips }`.
pub(super) fn install_greed_stacks(app: &mut App, skips: u32) {
    app.world_mut().insert_resource(GreedStacks { skips });
}

/// Inserts a Greed `ProtocolDefinition` into `ActiveProtocols` so the
/// `protocol_active(Greed)` run-condition passes.
pub(super) fn seed_active_protocols_with_greed(app: &mut App, rarity_boost_per_skip: f32) {
    app.world_mut()
        .resource_mut::<ActiveProtocols>()
        .insert(ProtocolDefinition {
            name:        "Greed".into(),
            description: String::new(),
            unlock_tier: 0,
            tuning:      ProtocolTuning::Greed {
                rarity_boost_per_skip,
            },
        });
}

/// Invokes `greed::activate` directly via a fresh `CommandQueue` so each call
/// has an independent, deterministic flush. Mirrors other domains'
/// `activate_now` helpers.
pub(super) fn activate_now(app: &mut App, tuning: &ProtocolTuning) {
    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, app.world());
        activate(tuning, &mut commands);
    }
    queue.apply(app.world_mut());
}

// ── Message writers ─────────────────────────────────────────────────────────

/// Writes a single `ChipOfferSkipped` message.
pub(super) fn write_skip(app: &mut App) {
    app.world_mut()
        .resource_mut::<Messages<ChipOfferSkipped>>()
        .write(ChipOfferSkipped);
}

/// Writes `count` `ChipOfferSkipped` messages.
pub(super) fn write_n_skips(app: &mut App, count: u32) {
    let mut msgs = app.world_mut().resource_mut::<Messages<ChipOfferSkipped>>();
    for _ in 0..count {
        msgs.write(ChipOfferSkipped);
    }
}

// ── Shorthand for ProtocolKind ──────────────────────────────────────────────

pub(super) const GREED_KIND: ProtocolKind = ProtocolKind::Greed;
