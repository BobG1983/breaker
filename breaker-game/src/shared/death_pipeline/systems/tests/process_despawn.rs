//! Tests for `process_despawn_requests`.

use bevy::prelude::*;

use super::helpers::{PendingDespawns, build_despawn_app};
use crate::{prelude::*, shared::death_pipeline::despawn_entity::DespawnEntity};

#[test]
fn process_despawn_requests_despawns_entity() {
    let mut app = build_despawn_app();
    let entity = app.world_mut().spawn_empty().id();

    app.insert_resource(PendingDespawns(vec![DespawnEntity { entity }]));
    tick(&mut app);

    assert!(
        app.world().get_entity(entity).is_err(),
        "Entity should be despawned after process_despawn_requests"
    );
}

#[test]
fn process_despawn_requests_handles_already_despawned() {
    // try_despawn should not panic if entity is already gone.
    let mut app = build_despawn_app();
    let entity = app.world_mut().spawn_empty().id();
    app.world_mut().despawn(entity);

    app.insert_resource(PendingDespawns(vec![DespawnEntity { entity }]));
    // Should not panic
    tick(&mut app);
}

#[test]
fn process_despawn_requests_handles_multiple() {
    let mut app = build_despawn_app();
    let entity_a = app.world_mut().spawn_empty().id();
    let entity_b = app.world_mut().spawn_empty().id();

    app.insert_resource(PendingDespawns(vec![
        DespawnEntity { entity: entity_a },
        DespawnEntity { entity: entity_b },
    ]));
    tick(&mut app);

    assert!(
        app.world().get_entity(entity_a).is_err(),
        "Entity A should be despawned"
    );
    assert!(
        app.world().get_entity(entity_b).is_err(),
        "Entity B should be despawned"
    );
}

#[test]
fn process_despawn_requests_duplicate_entity_does_not_panic() {
    // Same entity in two messages — try_despawn handles the second gracefully.
    let mut app = build_despawn_app();
    let entity = app.world_mut().spawn_empty().id();

    app.insert_resource(PendingDespawns(vec![
        DespawnEntity { entity },
        DespawnEntity { entity },
    ]));
    // Should not panic
    tick(&mut app);

    assert!(
        app.world().get_entity(entity).is_err(),
        "Entity should be despawned"
    );
}
