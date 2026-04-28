//! Shared test fixtures for Siphon protocol tests.
//!
//! App builders, canonical `SiphonConfig`, `SiphonStreak` installers,
//! `ActiveProtocols` seeder, `Destroyed<Cell>` writers, `IncreaseNodeTimer`
//! collector reader, and the `activate_now` CommandQueue-flush helper.
//!
//! Canonical config is `streak_window: 2.0`, `time_per_kill: 0.5` — the
//! design-doc's worked-example values. The RON asset's `time_per_kill: 0.25`
//! is exercised only by the `ron_asset.rs` drift-guard tests via
//! `include_str!`.

#![allow(
    dead_code,
    reason = "shared helpers; not every test file uses every helper"
)]

use std::marker::PhantomData;

use bevy::{
    ecs::{message::Messages, world::CommandQueue},
    prelude::*,
};

use super::super::system::{SiphonConfig, SiphonStreak, activate, wire};
use crate::{
    mutators::protocols::{
        definition::{ProtocolDefinition, ProtocolKind, ProtocolTuning},
        resources::ActiveProtocols,
    },
    prelude::*,
    state::run::node::messages::IncreaseNodeTimer,
};

// ── App builders ────────────────────────────────────────────────────────────

/// Default Siphon test app. State hierarchy in `NodeState::Playing`,
/// `ActiveProtocols` + `SiphonStreak` initialised, `Destroyed<Cell>`
/// registered, `IncreaseNodeTimer` capture installed, `SiphonConfig`
/// inserted at the canonical values, and `wire` called.
pub(super) fn build_siphon_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveProtocols>()
        .with_resource::<SiphonStreak>()
        .with_message::<Destroyed<Cell>>()
        .with_message_capture::<IncreaseNodeTimer>()
        .build();
    app.world_mut().insert_resource(canonical_siphon_config());
    wire(&mut app);
    app
}

/// Same as [`build_siphon_app`] but omits `SiphonStreak` initialisation. Used
/// to test the harness-safe `Option<ResMut<SiphonStreak>>` guard path.
pub(super) fn build_siphon_app_no_streak() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveProtocols>()
        .with_message::<Destroyed<Cell>>()
        .with_message_capture::<IncreaseNodeTimer>()
        .build();
    app.world_mut().insert_resource(canonical_siphon_config());
    wire(&mut app);
    app
}

/// Same as [`build_siphon_app`] but does NOT insert `SiphonConfig`. Used to
/// test the harness-safe `Option<Res<SiphonConfig>>` guard path.
pub(super) fn build_siphon_app_no_config() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_resource::<ActiveProtocols>()
        .with_resource::<SiphonStreak>()
        .with_message::<Destroyed<Cell>>()
        .with_message_capture::<IncreaseNodeTimer>()
        .build();
    wire(&mut app);
    app
}

// ── Config / streak helpers ─────────────────────────────────────────────────

/// Canonical Siphon config used across all system-behavior tests. Matches the
/// design-doc's worked examples: `streak_window: 2.0`, `time_per_kill: 0.5`.
///
/// NOTE: This DIFFERS from the RON asset's `time_per_kill: 0.25` — the RON
/// drift-guard tests read the file literal via `include_str!` and do not use
/// this helper.
pub(super) const fn canonical_siphon_config() -> SiphonConfig {
    SiphonConfig {
        streak_window: 2.0,
        time_per_kill: 0.5,
    }
}

/// Inserts the given `SiphonConfig` as a resource.
pub(super) fn install_siphon_config(app: &mut App, cfg: SiphonConfig) {
    app.world_mut().insert_resource(cfg);
}

/// Replaces the app's `SiphonStreak` with
/// `SiphonStreak { window_remaining, kill_count }`.
pub(super) fn install_siphon_streak(app: &mut App, window_remaining: f32, kill_count: u32) {
    app.world_mut().insert_resource(SiphonStreak {
        window_remaining,
        kill_count,
    });
}

/// Inserts a Siphon `ProtocolDefinition` into `ActiveProtocols` so the
/// `protocol_active(Siphon)` run-condition passes.
pub(super) fn seed_active_protocols_with_siphon(
    app: &mut App,
    streak_window: f32,
    time_per_kill: f32,
) {
    app.world_mut()
        .resource_mut::<ActiveProtocols>()
        .insert(ProtocolDefinition {
            name:        "Siphon".into(),
            description: String::new(),
            unlock_tier: 0,
            tuning:      ProtocolTuning::Siphon {
                streak_window,
                time_per_kill,
            },
        });
}

/// Invokes `siphon::activate` directly via a fresh `CommandQueue` so each call
/// has an independent, deterministic flush. Mirrors
/// `greed::tests::helpers::activate_now`.
pub(super) fn activate_now(app: &mut App, tuning: &ProtocolTuning) {
    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, app.world());
        activate(tuning, &mut commands);
    }
    queue.apply(app.world_mut());
}

// ── Message writers ─────────────────────────────────────────────────────────

/// Writes a single `Destroyed<Cell>` message with placeholder payload.
/// The Siphon reader does not inspect fields; values are irrelevant.
pub(super) fn write_cell_destroyed(app: &mut App) {
    app.world_mut()
        .resource_mut::<Messages<Destroyed<Cell>>>()
        .write(Destroyed::<Cell> {
            victim:     Entity::PLACEHOLDER,
            killer:     None,
            victim_pos: Vec2::ZERO,
            killer_pos: None,
            _marker:    PhantomData,
        });
}

/// Writes `count` `Destroyed<Cell>` messages, each with the same placeholder
/// payload as [`write_cell_destroyed`].
pub(super) fn write_n_cell_destroyed(app: &mut App, count: u32) {
    for _ in 0..count {
        write_cell_destroyed(app);
    }
}

// ── Assertion helpers ───────────────────────────────────────────────────────

/// Returns every captured `IncreaseNodeTimer` from the
/// `MessageCollector<IncreaseNodeTimer>` resource. Tests that only need the
/// count call `.len()`; tests that need the `delta` sum map through
/// `.iter().map(|m| m.delta).sum()`.
pub(super) fn collected_increase_node_timers(app: &App) -> Vec<IncreaseNodeTimer> {
    app.world()
        .resource::<MessageCollector<IncreaseNodeTimer>>()
        .0
        .clone()
}

// ── Shorthand for ProtocolKind ──────────────────────────────────────────────

pub(super) const SIPHON_KIND: ProtocolKind = ProtocolKind::Siphon;
