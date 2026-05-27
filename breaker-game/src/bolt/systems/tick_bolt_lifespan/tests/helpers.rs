//! Shared test helpers for `tick_bolt_lifespan` tests.

use bevy::prelude::*;

use super::super::system::tick_bolt_lifespan;
use crate::{
    bolt::BoltPlugin, prelude::*, shared::birthing::BIRTHING_DURATION,
    state::run::resources::NodeOutcome,
};

// ── Constants ────────────────────────────────────────────────────────────────

pub(super) const FIXED_DT: f32 = 1.0 / 64.0;

// ── KillYourself<Bolt> regression-guard capture ──────────────────────────────

#[derive(Resource, Default)]
pub(super) struct CapturedKillYourselfBolt(pub(super) Vec<KillYourself<Bolt>>);

pub(super) fn capture_kill_yourself_bolt(
    mut reader: MessageReader<KillYourself<Bolt>>,
    mut captured: ResMut<CapturedKillYourselfBolt>,
) {
    for m in reader.read() {
        captured.0.push(m.clone());
    }
}

// ── Test app helpers ──────────────────────────────────────────────────────────

/// Emit-only app: `tick_bolt_lifespan` in `FixedUpdate` with `DespawnEntity` capture
/// and `KillYourself<Bolt>` capture (so regression-guard "zero" assertions work).
/// No `RantzDmgPlugin` — proves the system emits but does NOT despawn directly.
pub(super) fn lifespan_emit_app() -> App {
    TestAppBuilder::new()
        .with_message_capture::<DespawnEntity>()
        .with_message::<KillYourself<Bolt>>()
        .with_resource::<CapturedKillYourselfBolt>()
        .with_system(
            FixedUpdate,
            (tick_bolt_lifespan, capture_kill_yourself_bolt).chain(),
        )
        .build()
}

/// End-to-end app: `with_dmg_pipeline()` and `tick_bolt_lifespan` in `FixedUpdate`
/// with `DespawnEntity` capture and `KillYourself<Bolt>` capture.
/// Proves expiry → `DespawnEntity` → `process_despawn_requests` within one tick.
pub(super) fn lifespan_despawn_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_dmg_pipeline()
        .with_message::<KillYourself<Bolt>>()
        .with_resource::<CapturedKillYourselfBolt>()
        .build();
    app.add_systems(
        FixedUpdate,
        tick_bolt_lifespan.before(DmgSystems::ApplyKill),
    );
    app.add_systems(FixedUpdate, capture_kill_yourself_bolt);
    attach_message_capture::<DespawnEntity>(&mut app);
    app
}

/// Full plugin-wiring app for Behavior 6 — mirrors `scheduling_test_app()` at
/// `bolt/systems/bolt_cell_collision/tests/scheduling.rs:40-54`.
pub(super) fn bolt_plugin_wiring_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_physics()
        .with_playfield()
        .with_bolt_registry()
        .with_breaker_registry()
        .with_cell_registry()
        .with_resource::<crate::input::resources::InputActions>()
        .with_resource::<NodeOutcome>()
        .with_effects_pipeline()
        .build();
    app.add_plugins(BoltPlugin);
    attach_message_capture::<DespawnEntity>(&mut app);
    app
}

// ── Birthing helper ──────────────────────────────────────────────────────────

pub(super) fn test_birthing() -> Birthing {
    Birthing {
        timer:          Timer::from_seconds(BIRTHING_DURATION, TimerMode::Once),
        target_scale:   Scale2D { x: 8.0, y: 8.0 },
        stashed_layers: CollisionLayers::default(),
    }
}
