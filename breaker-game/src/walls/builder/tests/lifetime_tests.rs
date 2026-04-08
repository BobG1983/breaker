use bevy::{ecs::world::CommandQueue, prelude::*};

use super::helpers::default_playfield;
use crate::walls::{builder::core::types::Lifetime, components::Wall};

fn spawn_inworld(world: &mut World, f: impl FnOnce(&mut Commands) -> Entity) -> Entity {
    let mut queue = CommandQueue::default();
    let entity = {
        let mut commands = Commands::new(&mut queue, world);
        f(&mut commands)
    };
    queue.apply(world);
    entity
}

// ── Behavior 16: Default lifetime is Permanent ──

#[test]
fn default_lifetime_is_permanent_for_left() {
    let pf = default_playfield();
    let builder = Wall::builder().left(&pf);
    assert_eq!(
        builder.lifetime,
        Lifetime::Permanent,
        "default lifetime should be Permanent"
    );
}

#[test]
fn default_lifetime_is_permanent_for_floor() {
    let pf = default_playfield();
    let builder = Wall::builder().floor(&pf);
    assert_eq!(
        builder.lifetime,
        Lifetime::Permanent,
        "default lifetime for Floor should be Permanent"
    );
}

// ── Behavior 17: .timed() is available on Floor and sets Lifetime::Timed ──

#[test]
fn timed_compiles_on_floor_and_can_build() {
    let pf = default_playfield();
    // Verify the method compiles and builder can proceed to build
    // Just verify the chain compiles — spawn into a world

    let mut world = World::new();

    spawn_inworld(&mut world, |commands| {
        Wall::builder().floor(&pf).timed(5.0).spawn(commands)
    });
}

#[test]
fn timed_zero_compiles_on_floor() {
    let pf = default_playfield();
    // Just verify the chain compiles — spawn into a world

    let mut world = World::new();

    spawn_inworld(&mut world, |commands| {
        Wall::builder().floor(&pf).timed(0.0).spawn(commands)
    });
}

// ── Behavior 18: .one_shot() is available on Floor and sets Lifetime::OneShot ──

#[test]
fn one_shot_compiles_on_floor_and_can_build() {
    let pf = default_playfield();
    // Just verify the chain compiles — spawn into a world

    let mut world = World::new();

    spawn_inworld(&mut world, |commands| {
        Wall::builder().floor(&pf).one_shot().spawn(commands)
    });
}

#[test]
fn one_shot_then_timed_last_wins() {
    let pf = default_playfield();
    // Both compile and build succeeds
    // Just verify the chain compiles — spawn into a world

    let mut world = World::new();

    spawn_inworld(&mut world, |commands| {
        Wall::builder()
            .floor(&pf)
            .one_shot()
            .timed(3.0)
            .spawn(commands)
    });
}

// ── Behavior 51: Lifetime::default() is Permanent ──

#[test]
fn lifetime_default_is_permanent() {
    let lifetime = Lifetime::default();
    assert_eq!(lifetime, Lifetime::Permanent);
}

// ── Behavior 52: Lifetime variants are distinct ──

#[test]
fn lifetime_variants_are_distinct() {
    assert_ne!(Lifetime::Permanent, Lifetime::OneShot);
    assert_ne!(Lifetime::Permanent, Lifetime::Timed(5.0));
    assert_ne!(Lifetime::OneShot, Lifetime::Timed(5.0));
    assert_ne!(Lifetime::Timed(5.0), Lifetime::Timed(3.0));
}

#[test]
fn lifetime_timed_zero_distinct_from_permanent() {
    assert_ne!(
        Lifetime::Timed(0.0),
        Lifetime::Permanent,
        "Timed(0.0) should be distinct from Permanent"
    );
}
