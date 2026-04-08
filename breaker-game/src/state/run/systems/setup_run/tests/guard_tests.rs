//! Tests for the guard path: `setup_run` early-returns when a breaker already exists.

use bevy::{ecs::world::CommandQueue, prelude::*};

use super::helpers::test_app;
use crate::{
    bolt::{components::Bolt, messages::BoltSpawned},
    breaker::{components::Breaker, definition::BreakerDefinition, messages::BreakerSpawned},
};

fn spawn_in_world(world: &mut World, f: impl FnOnce(&mut Commands) -> Entity) -> Entity {
    let mut queue = CommandQueue::default();
    let entity = {
        let mut commands = Commands::new(&mut queue, world);
        f(&mut commands)
    };
    queue.apply(world);
    entity
}

// ── Behavior 1: setup_run early-returns when a breaker already exists ──

#[test]
fn setup_run_early_returns_when_breaker_already_exists() {
    let mut app = test_app();
    // Pre-spawn a breaker entity
    let def = BreakerDefinition::default();
    spawn_in_world(app.world_mut(), |commands| {
        Breaker::builder()
            .definition(&def)
            .headless()
            .primary()
            .spawn(commands)
    });
    app.update();

    let breaker_count = app
        .world_mut()
        .query::<&Breaker>()
        .iter(app.world())
        .count();
    assert_eq!(
        breaker_count, 1,
        "should still have exactly 1 breaker (no new one spawned)"
    );

    let bolt_count = app.world_mut().query::<&Bolt>().iter(app.world()).count();
    assert_eq!(
        bolt_count, 0,
        "should not spawn a bolt when breaker already exists"
    );

    let breaker_msgs = app.world().resource::<Messages<BreakerSpawned>>();
    assert_eq!(
        breaker_msgs.iter_current_update_messages().count(),
        0,
        "should NOT send BreakerSpawned when breaker already exists"
    );

    let bolt_msgs = app.world().resource::<Messages<BoltSpawned>>();
    assert_eq!(
        bolt_msgs.iter_current_update_messages().count(),
        0,
        "should NOT send BoltSpawned when breaker already exists"
    );
}

#[test]
fn setup_run_early_returns_when_two_breakers_exist() {
    let mut app = test_app();
    // Pre-spawn two breaker entities
    let def = BreakerDefinition::default();
    spawn_in_world(app.world_mut(), |commands| {
        Breaker::builder()
            .definition(&def)
            .headless()
            .primary()
            .spawn(commands)
    });
    spawn_in_world(app.world_mut(), |commands| {
        Breaker::builder()
            .definition(&def)
            .headless()
            .extra()
            .spawn(commands)
    });
    app.update();

    let breaker_count = app
        .world_mut()
        .query::<&Breaker>()
        .iter(app.world())
        .count();
    assert_eq!(
        breaker_count, 2,
        "should still have exactly 2 breakers (no new ones spawned)"
    );

    let bolt_count = app.world_mut().query::<&Bolt>().iter(app.world()).count();
    assert_eq!(
        bolt_count, 0,
        "should not spawn a bolt when breakers already exist"
    );
}
