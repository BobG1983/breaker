use bevy::{ecs::world::CommandQueue, prelude::*};
use rantzsoft_stateflow::CleanupOnExit;

use super::helpers::test_breaker_definition;
use crate::{
    breaker::components::{Breaker, ExtraBreaker, PrimaryBreaker},
    state::types::{NodeState, RunState},
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

// ── Behavior 36: .primary() produces PrimaryBreaker + CleanupOnExit<RunState> ──

#[test]
fn primary_produces_primary_breaker_and_cleanup_on_run_end() {
    let def = test_breaker_definition();
    let mut world = World::new();
    let entity = spawn_in_world(&mut world, |commands| {
        Breaker::builder()
            .definition(&def)
            .headless()
            .primary()
            .spawn(commands)
    });

    assert!(
        world.get::<PrimaryBreaker>(entity).is_some(),
        "should have PrimaryBreaker"
    );
    assert!(
        world.get::<CleanupOnExit<RunState>>(entity).is_some(),
        "should have CleanupOnExit<RunState>"
    );
    assert!(
        world.get::<ExtraBreaker>(entity).is_none(),
        "should NOT have ExtraBreaker"
    );
    assert!(
        world.get::<CleanupOnExit<NodeState>>(entity).is_none(),
        "should NOT have CleanupOnExit<NodeState>"
    );
}

// ── Behavior 37: .extra() produces ExtraBreaker + CleanupOnExit<NodeState> ──

#[test]
fn extra_produces_extra_breaker_and_cleanup_on_node_exit() {
    let def = test_breaker_definition();
    let mut world = World::new();
    let entity = spawn_in_world(&mut world, |commands| {
        Breaker::builder()
            .definition(&def)
            .headless()
            .extra()
            .spawn(commands)
    });

    assert!(
        world.get::<ExtraBreaker>(entity).is_some(),
        "should have ExtraBreaker"
    );
    assert!(
        world.get::<CleanupOnExit<NodeState>>(entity).is_some(),
        "should have CleanupOnExit<NodeState>"
    );
    assert!(
        world.get::<PrimaryBreaker>(entity).is_none(),
        "should NOT have PrimaryBreaker"
    );
    assert!(
        world.get::<CleanupOnExit<RunState>>(entity).is_none(),
        "should NOT have CleanupOnExit<RunState>"
    );
}

// ── Behavior 38: .headless() build does NOT insert mesh/material components ──

#[test]
fn headless_build_has_no_mesh_or_material() {
    let def = test_breaker_definition();
    let mut world = World::new();
    let entity = spawn_in_world(&mut world, |commands| {
        Breaker::builder()
            .definition(&def)
            .headless()
            .primary()
            .spawn(commands)
    });

    assert!(
        world.get::<Mesh2d>(entity).is_none(),
        "headless should NOT have Mesh2d"
    );
    assert!(
        world.get::<MeshMaterial2d<ColorMaterial>>(entity).is_none(),
        "headless should NOT have MeshMaterial2d<ColorMaterial>"
    );
    // Verify the entity still has gameplay components (guard against stub false pass)
    assert!(
        world.get::<Breaker>(entity).is_some(),
        "should still have Breaker marker"
    );
    assert!(
        world.get::<PrimaryBreaker>(entity).is_some(),
        "should still have PrimaryBreaker (guard against stub false pass)"
    );
}

// ── Behavior 39: .rendered() build inserts mesh/material components ──

#[test]
fn rendered_build_has_mesh_and_material() {
    let def = test_breaker_definition();
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Mesh>()
        .init_asset::<ColorMaterial>();
    app.add_systems(Update, {
        move |mut commands: Commands,
              mut meshes: ResMut<Assets<Mesh>>,
              mut materials: ResMut<Assets<ColorMaterial>>| {
            Breaker::builder()
                .definition(&def)
                .rendered(&mut meshes, &mut materials)
                .primary()
                .spawn(&mut commands);
        }
    });
    app.update();

    let mut query = app.world_mut().query_filtered::<Entity, With<Breaker>>();
    let entities: Vec<Entity> = query.iter(app.world()).collect();
    assert_eq!(entities.len(), 1);
    let entity = entities[0];

    assert!(
        app.world().get::<Mesh2d>(entity).is_some(),
        "rendered should have Mesh2d"
    );
    assert!(
        app.world()
            .get::<MeshMaterial2d<ColorMaterial>>(entity)
            .is_some(),
        "rendered should have MeshMaterial2d<ColorMaterial>"
    );
    // Verify gameplay components are also present
    assert!(
        app.world().get::<Breaker>(entity).is_some(),
        "should still have Breaker marker"
    );
    // Guard: check non-#[require] component
    assert!(
        app.world().get::<PrimaryBreaker>(entity).is_some(),
        "should have PrimaryBreaker (guard against stub false pass)"
    );
    // Rendered builds include GameDrawLayer
    assert!(
        matches!(
            app.world().get::<crate::shared::GameDrawLayer>(entity),
            Some(crate::shared::GameDrawLayer::Breaker)
        ),
        "rendered build should have GameDrawLayer::Breaker"
    );
}
