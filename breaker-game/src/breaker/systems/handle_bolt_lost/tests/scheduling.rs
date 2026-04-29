//! Behaviors 20, 21: scheduling and ordering for `handle_bolt_lost`.
//!
//! Behavior 20 — `handle_bolt_lost` is wired in `BreakerPlugin` (no manual
//! system addition allowed; that would defeat the wiring assertion).
//! Behavior 21 — `handle_bolt_lost` runs after `BoltSystems::BoltLost` and
//! before `NodeSystems::ReduceNodeTimer`, validated end-to-end via a single
//! `NodeTimer.remaining` assertion.

use bevy::prelude::*;

use crate::{
    breaker::{
        BreakerPlugin,
        components::{BoltLossBehavior, Breaker},
    },
    prelude::*,
    state::run::node::{
        messages::{ReduceNodeTimer, TimerExpired},
        sets::NodeSystems,
        systems::apply_reduce_node_timer,
    },
};

/// Builds an `App` with `BreakerPlugin` registered and the state hierarchy
/// driven into `NodeState::Playing`. The plugin's `FixedUpdate` systems require
/// `InputActions` (movement), `PlayfieldConfig` (bounds), and a physics
/// `CollisionQuadtree` (broad-phase). `with_physics()` provides the quadtree
/// via `RantzPhysics2dPlugin`; `with_playfield()` provides `PlayfieldConfig`.
/// Both are installed through `TestAppBuilder`.
fn breaker_plugin_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_physics()
        .with_playfield()
        .with_resource::<InputActions>()
        // Messages BreakerPlugin's systems read but don't register themselves
        // (added by other plugins in production wiring).
        .with_message::<BoltLost>()
        .with_message::<ReduceNodeTimer>()
        .with_message::<BoltImpactBreaker>()
        .build();
    app.add_plugins(BreakerPlugin);
    app
}

/// Same as [`breaker_plugin_app`] but also registers `apply_reduce_node_timer`
/// (the consumer of `ReduceNodeTimer`) in the `NodeSystems::ReduceNodeTimer`
/// set so behavior 21 can validate the end-to-end ordering: `handle_bolt_lost`
/// writes the message AND `apply_reduce_node_timer` consumes it in the same
/// `FixedUpdate` step. Wiring the consumer into its production set lets the
/// `.before(NodeSystems::ReduceNodeTimer)` ordering in `BreakerPlugin` (added
/// by writer-code in GREEN) take effect against this test consumer.
fn breaker_plugin_app_with_timer_consumer() -> App {
    let mut app = breaker_plugin_app();
    app.add_message::<TimerExpired>();
    app.add_systems(
        FixedUpdate,
        apply_reduce_node_timer.in_set(NodeSystems::ReduceNodeTimer),
    );
    app
}

// ── Behavior 20: handle_bolt_lost is registered in BreakerPlugin in FixedUpdate ──

#[test]
fn handle_bolt_lost_is_wired_in_breaker_plugin_fixed_update() {
    let mut app = breaker_plugin_app();

    // Spawn a Breaker manually with a known Hp.
    let breaker = app
        .world_mut()
        .spawn((
            Breaker,
            BoltLossBehavior::LifeLoss(1),
            Hp {
                current:  3.0,
                starting: 3.0,
                max:      None,
            },
        ))
        .id();

    // Write a BoltLost targeting that breaker.
    app.world_mut()
        .resource_mut::<Messages<BoltLost>>()
        .write(BoltLost {
            bolt: Entity::PLACEHOLDER,
            breaker,
        });

    // Advance one FixedUpdate tick.
    tick(&mut app);

    let hp = app
        .world()
        .get::<Hp>(breaker)
        .expect("breaker should still have Hp");
    assert!(
        (hp.current - 2.0).abs() < f32::EPSILON,
        "Hp.current should be 2.0 after BreakerPlugin advances one FixedUpdate; got {}. \
         If this fails the new system is not registered in BreakerPlugin.",
        hp.current,
    );
}

// ── Behavior 21: handle_bolt_lost runs after BoltSystems::BoltLost and before
//                 NodeSystems::ReduceNodeTimer ──

#[test]
fn handle_bolt_lost_writes_reduce_message_and_apply_consumes_in_same_tick() {
    let mut app = breaker_plugin_app_with_timer_consumer();

    // Set up a NodeTimer with 10.0 remaining (mirrors the construction pattern
    // shown in apply_reduce_node_timer.rs::tests::test_app_with_send).
    app.world_mut().insert_resource(NodeTimer {
        remaining: 10.0,
        total:     10.0,
    });

    // Spawn breaker with TimeLoss(5.0).
    let breaker = app
        .world_mut()
        .spawn((Breaker, BoltLossBehavior::TimeLoss(5.0)))
        .id();

    // Write a synthetic BoltLost.
    app.world_mut()
        .resource_mut::<Messages<BoltLost>>()
        .write(BoltLost {
            bolt: Entity::PLACEHOLDER,
            breaker,
        });

    // Advance one FixedUpdate.
    tick(&mut app);

    let timer = app.world().resource::<NodeTimer>();
    assert!(
        (timer.remaining - 5.0).abs() < f32::EPSILON,
        "NodeTimer.remaining should be 5.0 after handle_bolt_lost writes ReduceNodeTimer \
         AND apply_reduce_node_timer consumes it in the same FixedUpdate step; got {}",
        timer.remaining,
    );
}
