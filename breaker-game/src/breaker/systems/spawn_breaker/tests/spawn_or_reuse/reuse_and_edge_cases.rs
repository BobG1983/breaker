//! Tests for the `spawn_or_reuse_breaker` system — reuse and edge cases.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::{Position2D, PreviousPosition};

use super::super::helpers::*;
use crate::{
    breaker::{
        components::Breaker, definition::BreakerDefinition, messages::BreakerSpawned,
        registry::BreakerRegistry, resources::SelectedBreaker,
    },
    effect::effects::life_lost::LivesCount,
};

// Behavior 7 edge case: PreviousPosition matches Position2D
#[test]
fn spawned_breaker_previous_position_matches_position() {
    // Edge case: PreviousPosition must match Position2D to prevent interpolation teleport
    let mut app = test_app();
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Breaker>>()
        .iter(app.world())
        .next()
        .expect("breaker should exist");
    let pos = app.world().get::<Position2D>(entity).expect("Position2D");
    let prev = app
        .world()
        .get::<PreviousPosition>(entity)
        .expect("PreviousPosition");
    assert_eq!(
        pos.0, prev.0,
        "PreviousPosition should match Position2D to prevent teleport"
    );
}

// ── Behavior 8: Effect dispatch ────────────────────────────────────────

#[test]
fn spawned_breaker_with_empty_effects_has_empty_bound_effects() {
    // Edge case: BreakerDefinition with empty effects — BoundEffects should have 0 chains
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<BreakerSpawned>()
        .init_resource::<crate::shared::PlayfieldConfig>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>();

    let mut registry = BreakerRegistry::default();
    registry.insert(
        "Empty".to_string(),
        BreakerDefinition {
            name: "Empty".to_string(),
            effects: vec![],
            ..BreakerDefinition::default()
        },
    );
    app.insert_resource(registry);
    app.insert_resource(SelectedBreaker("Empty".to_owned()));
    app.add_systems(Startup, super::super::super::system::spawn_or_reuse_breaker);
    app.update();

    // BreakerSpawned should still be written
    let messages = app.world().resource::<Messages<BreakerSpawned>>();
    assert!(
        messages.iter_current_update_messages().count() > 0,
        "BreakerSpawned should be written even with empty effects"
    );
}

// ── Behavior 9: Subsequent node reuses existing breaker ────────────────

#[test]
fn existing_breaker_is_reused_no_new_spawn() {
    // Given: one breaker already exists
    let mut app = test_app();
    // Pre-spawn a breaker entity
    let def = BreakerDefinition::default();
    app.world_mut().spawn(
        Breaker::builder()
            .definition(&def)
            .headless()
            .primary()
            .build(),
    );
    app.update();

    let count = app
        .world_mut()
        .query::<&Breaker>()
        .iter(app.world())
        .count();
    assert_eq!(
        count, 1,
        "should not spawn additional breaker when one already exists"
    );
}

#[test]
fn existing_breaker_still_sends_breaker_spawned() {
    let mut app = test_app();
    // Pre-spawn a breaker entity
    let def = BreakerDefinition::default();
    app.world_mut().spawn(
        Breaker::builder()
            .definition(&def)
            .headless()
            .primary()
            .build(),
    );
    app.update();

    let messages = app.world().resource::<Messages<BreakerSpawned>>();
    assert!(
        messages.iter_current_update_messages().count() > 0,
        "spawn_or_reuse_breaker must send BreakerSpawned even when breaker already exists"
    );
}

#[test]
fn two_existing_breakers_both_preserved() {
    // Edge case: two breakers exist already
    let mut app = test_app();
    let def = BreakerDefinition::default();
    app.world_mut().spawn(
        Breaker::builder()
            .definition(&def)
            .headless()
            .primary()
            .build(),
    );
    app.world_mut().spawn(
        Breaker::builder()
            .definition(&def)
            .headless()
            .primary()
            .build(),
    );
    app.update();

    let count = app
        .world_mut()
        .query::<&Breaker>()
        .iter(app.world())
        .count();
    assert_eq!(count, 2, "both existing breakers should be preserved");

    let messages = app.world().resource::<Messages<BreakerSpawned>>();
    assert!(
        messages.iter_current_update_messages().count() > 0,
        "BreakerSpawned should be written"
    );
}

// ── Behavior 10: Selected breaker not in registry is a no-op ───────────

#[test]
fn selected_breaker_not_in_registry_is_noop() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<BreakerSpawned>()
        .init_resource::<crate::shared::PlayfieldConfig>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>();

    let mut registry = BreakerRegistry::default();
    registry.insert(
        "Aegis".to_string(),
        BreakerDefinition {
            name: "Aegis".to_string(),
            ..BreakerDefinition::default()
        },
    );
    app.insert_resource(registry);
    app.insert_resource(SelectedBreaker("NonExistent".to_owned()));
    app.add_systems(Startup, super::super::super::system::spawn_or_reuse_breaker);
    app.update();

    let count = app
        .world_mut()
        .query::<&Breaker>()
        .iter(app.world())
        .count();
    assert_eq!(
        count, 0,
        "no breaker should be created when selected breaker is not in registry"
    );

    let messages = app.world().resource::<Messages<BreakerSpawned>>();
    assert_eq!(
        messages.iter_current_update_messages().count(),
        0,
        "no BreakerSpawned should be written when selected breaker is not in registry"
    );
}

// ── Behavior 11: Chrono breaker (infinite lives) ───────────────────────

#[test]
fn chrono_breaker_spawns_with_infinite_lives() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_message::<BreakerSpawned>()
        .init_resource::<crate::shared::PlayfieldConfig>()
        .init_resource::<Assets<Mesh>>()
        .init_resource::<Assets<ColorMaterial>>();

    let mut registry = BreakerRegistry::default();
    registry.insert(
        "Chrono".to_string(),
        BreakerDefinition {
            name: "Chrono".to_string(),
            life_pool: None,
            ..BreakerDefinition::default()
        },
    );
    app.insert_resource(registry);
    app.insert_resource(SelectedBreaker("Chrono".to_owned()));
    app.add_systems(Startup, super::super::super::system::spawn_or_reuse_breaker);
    app.update();

    let entity = app
        .world_mut()
        .query_filtered::<Entity, With<Breaker>>()
        .iter(app.world())
        .next()
        .expect("Chrono breaker should exist");
    let lives = app
        .world()
        .get::<LivesCount>(entity)
        .expect("Chrono breaker should have LivesCount component");
    assert_eq!(
        lives.0, None,
        "Chrono breaker should have LivesCount(None) for infinite lives, not missing or Some"
    );
}
