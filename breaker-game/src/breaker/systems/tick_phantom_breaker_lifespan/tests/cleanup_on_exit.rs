//! Tests #11a–#11b: `CleanupOnExit<NodeState>` despawns phantom on node exit,
//! independent of lifespan.

use bevy::prelude::*;
use rantzsoft_stateflow::cleanup_on_exit;

use super::helpers::spawn_phantom;
use crate::prelude::*;

/// Builds an `App` with state hierarchy in `NodeState::Playing`, dmg pipeline,
/// `tick_phantom_breaker_lifespan` in `FixedUpdate` gated on `NodeState::Playing`,
/// and `cleanup_on_exit::<NodeState>` registered on `OnEnter(NodeState::Teardown)`.
/// The explicit cleanup-system wiring is required — `with_state_hierarchy()` does
/// not include node-cleanup systems.
fn cleanup_test_app() -> App {
    use super::super::tick_phantom_breaker_lifespan;
    TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .with_dmg_pipeline()
        .with_message_capture::<DespawnEntity>()
        .with_system(
            FixedUpdate,
            tick_phantom_breaker_lifespan.run_if(in_state(NodeState::Playing)),
        )
        .with_system(OnEnter(NodeState::Teardown), cleanup_on_exit::<NodeState>)
        .build()
}

// ── Test #11a — Extra phantom with long lifespan is despawned on NodeState exit ──

#[test]
fn phantom_extra_despawned_by_cleanup_on_node_exit_regardless_of_lifespan() {
    let mut app = cleanup_test_app();
    let phantom = spawn_phantom(&mut app, 999.0);

    // Sanity tick: phantom still alive, lifespan decremented but >> 0.
    tick(&mut app);
    assert!(
        app.world().get_entity(phantom).is_ok(),
        "phantom with lifespan 999.0 must still be alive after one tick"
    );
    {
        let lifespan = app
            .world()
            .get::<crate::shared::Lifespan>(phantom)
            .expect("Lifespan present");
        assert!(
            lifespan.remaining > 0.0,
            "remaining must still be positive, got {}",
            lifespan.remaining
        );
    }

    // Transition out of Playing → Teardown triggers OnEnter(NodeState::Teardown) cleanup.
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::Teardown);
    app.update();

    assert!(
        app.world().get_entity(phantom).is_err(),
        "phantom must be despawned by cleanup_on_exit<NodeState> on node exit"
    );

    // cleanup_on_exit despawns directly — must NOT go through DespawnEntity.
    let collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert!(
        collector.0.is_empty(),
        "DespawnEntity must NOT be emitted by the cleanup path — cleanup_on_exit despawns directly"
    );
}

// ── Test #11a edge — Playing → Loading does NOT despawn the phantom ──
//
// Production wires cleanup_on_exit on OnEnter(Teardown), not OnExit(Playing).
// This test documents that contract and guards against accidentally reverting to
// an OnExit(Playing) trigger.

#[test]
fn phantom_extra_survives_playing_to_loading_transition() {
    let mut app = cleanup_test_app();
    let phantom = spawn_phantom(&mut app, 999.0);

    // Transition to Loading — OnEnter(Teardown) does NOT fire; phantom must survive.
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::Loading);
    app.update();

    assert!(
        app.world().get_entity(phantom).is_ok(),
        "Playing → Loading must NOT despawn the phantom — cleanup_on_exit fires only on OnEnter(Teardown)"
    );

    let collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert!(
        collector.0.is_empty(),
        "no DespawnEntity expected on Playing → Loading"
    );
}

// ── Test #11b — System runs in Playing but not after NodeState exits ──
//
// A persistent phantom (no CleanupOnExit<NodeState>) is live before and after the
// state transition. The test first ticks while in Playing (proving the system ran:
// Lifespan.remaining was decremented — this assertion fails with the no-op stub,
// making the test genuinely RED). Then it records the state, transitions out of
// Playing, and ticks again — asserting Lifespan.remaining did NOT decrease further.

#[test]
fn system_decrements_in_playing_but_not_after_exit() {
    let mut app = cleanup_test_app();

    let persistent_phantom = app
        .world_mut()
        .spawn((
            Breaker,
            crate::breaker::components::PhantomBreaker,
            crate::shared::Lifespan { remaining: 1.0 },
        ))
        .id();

    // Tick while in Playing — system must run and decrement.
    tick(&mut app);

    let remaining_after_playing_tick = app
        .world()
        .get::<crate::shared::Lifespan>(persistent_phantom)
        .expect("Lifespan present after playing tick")
        .remaining;
    assert!(
        remaining_after_playing_tick < 1.0,
        "Lifespan.remaining must be decremented during Playing tick; got {remaining_after_playing_tick} (no-op stub fails here)"
    );

    // Transition out of Playing.
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::Teardown);
    app.update();

    assert!(
        app.world().get_entity(persistent_phantom).is_ok(),
        "persistent phantom must survive state transition (no CleanupOnExit)"
    );

    // Tick after transition — system must be gated off; remaining must not change.
    tick(&mut app);

    let remaining_after_post_transition_tick = app
        .world()
        .get::<crate::shared::Lifespan>(persistent_phantom)
        .expect("Lifespan present after post-transition tick")
        .remaining;
    assert!(
        (remaining_after_post_transition_tick - remaining_after_playing_tick).abs() < 1e-6,
        "Lifespan.remaining must not change after NodeState exits Playing \
         (run_if gate must block the system)"
    );

    let collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert!(
        collector.0.is_empty(),
        "DespawnEntity must not be emitted post-transition"
    );
}

// ── Test #11b edge — full timeline: Playing tick decrements, post-exit tick does not ──
//
// Same two-phase structure; fails at RED on the Playing-tick decrement assertion.
#[test]
fn no_despawn_entity_emitted_across_full_transition_plus_tick_timeline() {
    let mut app = cleanup_test_app();

    let persistent_phantom = app
        .world_mut()
        .spawn((
            Breaker,
            crate::breaker::components::PhantomBreaker,
            crate::shared::Lifespan { remaining: 1.0 },
        ))
        .id();

    // Phase 1: tick while in Playing.
    tick(&mut app);

    let after_playing = app
        .world()
        .get::<crate::shared::Lifespan>(persistent_phantom)
        .expect("Lifespan present")
        .remaining;
    assert!(
        after_playing < 1.0,
        "Lifespan.remaining must decrease during Playing; got {after_playing} (fails with no-op stub)"
    );

    // Phase 2: state transition then tick.
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::Teardown);
    app.update();

    tick(&mut app);

    let after_transition = app
        .world()
        .get::<crate::shared::Lifespan>(persistent_phantom)
        .expect("Lifespan present after transition tick")
        .remaining;
    assert!(
        (after_transition - after_playing).abs() < 1e-6,
        "Lifespan.remaining must not change after exiting Playing"
    );

    let collector = app.world().resource::<MessageCollector<DespawnEntity>>();
    assert!(
        collector.0.is_empty(),
        "no DespawnEntity should be emitted across the entire timeline for a long-lifespan phantom"
    );
}
