//! T17 — `CleanupOnExit<NodeState>` despawns a phantom bolt on
//! `OnExit(NodeState::Playing)` regardless of remaining lifespan.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;
use rantzsoft_stateflow::cleanup_on_exit;

use super::test_bolt_definition;
use crate::{
    bolt::components::{LifetimeEndBehavior, PhantomDedupKey, PhantomParams},
    prelude::*,
};

fn build_cleanup_app() -> App {
    let mut app = TestAppBuilder::new()
        .with_state_hierarchy()
        .in_state_node_playing()
        .build();
    app.init_asset::<Mesh>();
    app.init_asset::<ColorMaterial>();
    app.add_systems(OnEnter(NodeState::Teardown), cleanup_on_exit::<NodeState>);
    app
}

fn drive_to_teardown(app: &mut App) {
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::AnimateOut);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<NodeState>>()
        .set(NodeState::Teardown);
    app.update();
}

// ── T17 — phantom bolt with remaining lifespan is despawned on node exit ──

#[test]
fn cleanup_on_exit_despawns_phantom_bolt_with_remaining_lifespan() {
    let def = test_bolt_definition();
    let mut app = build_cleanup_app();

    // Spawn via the W2 builder headless extra terminal — entity carries
    // CleanupOnExit<NodeState> from the .extra() role and has non-zero lifespan.
    let e = {
        let mut commands = app.world_mut().commands();
        Bolt::builder()
            .definition(&def)
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .phantom(PhantomParams {
                dedup_key: PhantomDedupKey::Bolt(Entity::PLACEHOLDER),
            })
            .with_lifespan(10.0)
            .with_lifetime_end_behavior(LifetimeEndBehavior::Despawn)
            .headless()
            .spawn(&mut commands)
    };
    app.world_mut().flush();

    // Pre-condition: entity exists and still has non-zero lifespan.
    assert!(
        app.world().get_entity(e).is_ok(),
        "pre-condition: phantom bolt must exist before teardown"
    );
    assert!(
        app.world().get::<CleanupOnExit<NodeState>>(e).is_some(),
        "pre-condition: CleanupOnExit<NodeState> must be present on phantom bolt"
    );

    drive_to_teardown(&mut app);

    // T17: entity despawned — lifespan > 0 did NOT prevent cleanup.
    assert!(
        app.world().get_entity(e).is_err(),
        "phantom bolt with remaining lifespan must be despawned on OnExit(Playing)"
    );
}

// ── T17 (edge case) — two phantom bolts with different lifespans are both despawned ──

#[test]
fn cleanup_on_exit_despawns_all_phantom_bolts_regardless_of_lifespan() {
    let def = test_bolt_definition();
    let mut app = build_cleanup_app();

    let e1 = {
        let mut commands = app.world_mut().commands();
        Bolt::builder()
            .definition(&def)
            .at_position(Vec2::ZERO)
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .phantom(PhantomParams {
                dedup_key: PhantomDedupKey::Bolt(Entity::PLACEHOLDER),
            })
            .with_lifespan(10.0)
            .with_lifetime_end_behavior(LifetimeEndBehavior::Despawn)
            .headless()
            .spawn(&mut commands)
    };
    let e2 = {
        let mut commands = app.world_mut().commands();
        Bolt::builder()
            .definition(&def)
            .at_position(Vec2::new(100.0, 0.0))
            .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
            .extra()
            .phantom(PhantomParams {
                dedup_key: PhantomDedupKey::Bolt(Entity::PLACEHOLDER),
            })
            .with_lifespan(0.5)
            .with_lifetime_end_behavior(LifetimeEndBehavior::Despawn)
            .headless()
            .spawn(&mut commands)
    };
    app.world_mut().flush();

    drive_to_teardown(&mut app);

    assert!(
        app.world().get_entity(e1).is_err(),
        "phantom bolt with lifespan 10.0 must be despawned on exit"
    );
    assert!(
        app.world().get_entity(e2).is_err(),
        "phantom bolt with lifespan 0.5 must be despawned on exit"
    );
}
