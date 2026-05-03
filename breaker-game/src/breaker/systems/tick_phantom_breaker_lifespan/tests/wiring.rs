//! Test W-1: `BreakerPlugin` registers `tick_phantom_breaker_lifespan` in `FixedUpdate`,
//! gated by `run_if(in_state(NodeState::Playing))`.

use bevy::prelude::*;

use super::helpers::spawn_phantom;
use crate::{
    breaker::{BreakerPlugin, components::Breaker},
    prelude::*,
};

/// Builds an `App` with `BreakerPlugin` fully registered plus the full state
/// hierarchy driven into `NodeState::Playing`. The plugin registers
/// `tick_phantom_breaker_lifespan` internally; this test verifies it
/// end-to-end via behavioral inspection.
fn breaker_plugin_wiring_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_dmg_pipeline()
        .with_physics()
        .with_playfield()
        .with_resource::<crate::input::resources::InputActions>()
        .with_message::<crate::bolt::messages::BoltLost>()
        .with_message::<crate::bolt::messages::BoltImpactBreaker>()
        .with_message::<crate::state::run::node::messages::ReduceNodeTimer>()
        .build();
    app.add_plugins(BreakerPlugin);
    attach_message_capture::<DespawnEntity>(&mut app);
    app
}

// ── W-1: BreakerPlugin registers tick_phantom_breaker_lifespan in FixedUpdate ──

#[test]
fn breaker_plugin_registers_tick_phantom_lifespan_in_fixed_update() {
    let mut app = breaker_plugin_wiring_app();
    let phantom = spawn_phantom(&mut app, 0.01);

    tick(&mut app);

    // The phantom must be despawned through the chain:
    // tick_phantom_breaker_lifespan → DespawnEntity → process_despawn_requests.
    // If the system is not registered, the entity will still be alive.
    assert!(
        app.world().get_entity(phantom).is_err(),
        "BreakerPlugin must register tick_phantom_breaker_lifespan; \
         phantom with lifespan 0.01 must be despawned after one tick in NodeState::Playing"
    );
}

// ── W-1 edge: system runs in Playing but is gated off outside it ──
//
// Two-phase test: first tick while in Playing (asserts decrement — fails with the
// no-op stub, making this test genuinely RED), then exit Playing and tick again
// (asserts remaining unchanged). Persistent phantom has no CleanupOnExit<NodeState>
// so it survives the state transition.

#[test]
fn tick_phantom_lifespan_runs_in_playing_not_outside() {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_dmg_pipeline()
        .with_physics()
        .with_playfield()
        .with_resource::<crate::input::resources::InputActions>()
        .with_message::<crate::bolt::messages::BoltLost>()
        .with_message::<crate::bolt::messages::BoltImpactBreaker>()
        .with_message::<crate::state::run::node::messages::ReduceNodeTimer>()
        .build();
    app.add_plugins(BreakerPlugin);
    attach_message_capture::<DespawnEntity>(&mut app);

    let persistent_phantom = app
        .world_mut()
        .spawn((
            Breaker,
            crate::breaker::components::PhantomBreaker,
            crate::shared::Lifespan { remaining: 1.0 },
        ))
        .id();

    // Phase 1: tick while in Playing — system must run and decrement.
    tick(&mut app);

    let after_playing = app
        .world()
        .get::<crate::shared::Lifespan>(persistent_phantom)
        .expect("Lifespan present after Playing tick")
        .remaining;
    assert!(
        after_playing < 1.0,
        "BreakerPlugin must register tick_phantom_breaker_lifespan; \
         Lifespan.remaining must decrease during Playing tick; got {after_playing} (no-op stub fails here)"
    );

    // Phase 2: exit Playing then tick — system must be gated off.
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::Loading);
    app.update();

    assert!(
        app.world().get_entity(persistent_phantom).is_ok(),
        "persistent phantom must survive state transition"
    );

    tick(&mut app);

    let after_transition = app
        .world()
        .get::<crate::shared::Lifespan>(persistent_phantom)
        .expect("Lifespan present after transition tick")
        .remaining;
    assert!(
        (after_transition - after_playing).abs() < 1e-6,
        "Lifespan.remaining must not change after exiting Playing \
         (run_if(in_state(NodeState::Playing)) gate must be respected)"
    );

    let collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert!(
        collector.0.is_empty(),
        "no DespawnEntity must be emitted post-transition"
    );
}
