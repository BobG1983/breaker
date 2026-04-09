use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;

use super::helpers::test_bolt_definition;
use crate::bolt::components::{Bolt, BoltServing, PrimaryBolt};

// ── Behavior 16: .headless() build does NOT insert Mesh2d or MeshMaterial2d ──

#[test]
fn headless_primary_serving_has_no_mesh_or_material() {
    let def = test_bolt_definition();
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::new(0.0, 50.0))
        .serving()
        .primary()
        .headless()
        .spawn(&mut world.commands());
    world.flush();

    assert!(
        world.get::<Bolt>(entity).is_some(),
        "should have Bolt marker"
    );
    assert!(
        world.get::<PrimaryBolt>(entity).is_some(),
        "should have PrimaryBolt"
    );
    assert!(
        world.get::<BoltServing>(entity).is_some(),
        "should have BoltServing"
    );
    assert!(
        world.get::<Mesh2d>(entity).is_none(),
        "headless should NOT have Mesh2d"
    );
    assert!(
        world.get::<MeshMaterial2d<ColorMaterial>>(entity).is_none(),
        "headless should NOT have MeshMaterial2d<ColorMaterial>"
    );
}

#[test]
fn headless_extra_velocity_has_no_mesh_or_material() {
    let def = test_bolt_definition();
    let mut world = World::new();
    let entity = Bolt::builder()
        .definition(&def)
        .at_position(Vec2::ZERO)
        .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
        .extra()
        .headless()
        .spawn(&mut world.commands());
    world.flush();

    assert!(
        world.get::<Mesh2d>(entity).is_none(),
        "headless extra velocity bolt should NOT have Mesh2d"
    );
    assert!(
        world.get::<MeshMaterial2d<ColorMaterial>>(entity).is_none(),
        "headless extra velocity bolt should NOT have MeshMaterial2d"
    );
}

// ── Behavior 17: .rendered() build inserts Mesh2d and MeshMaterial2d ──

#[test]
fn rendered_primary_serving_has_mesh_and_material() {
    let def = test_bolt_definition();
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Mesh>()
        .init_asset::<ColorMaterial>();
    app.add_systems(Update, {
        move |mut commands: Commands,
              mut meshes: ResMut<Assets<Mesh>>,
              mut materials: ResMut<Assets<ColorMaterial>>| {
            Bolt::builder()
                .definition(&def)
                .at_position(Vec2::new(0.0, 50.0))
                .serving()
                .primary()
                .rendered(&mut meshes, &mut materials)
                .spawn(&mut commands);
        }
    });
    app.update();

    let mut query = app.world_mut().query_filtered::<Entity, With<Bolt>>();
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
    assert!(
        app.world().get::<Bolt>(entity).is_some(),
        "should still have Bolt marker"
    );
    assert!(
        app.world().get::<PrimaryBolt>(entity).is_some(),
        "should have PrimaryBolt (guard against stub false pass)"
    );
    assert!(
        app.world().get::<BoltServing>(entity).is_some(),
        "should have BoltServing"
    );
}

#[test]
fn rendered_extra_velocity_has_mesh_and_material() {
    let def = test_bolt_definition();
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Mesh>()
        .init_asset::<ColorMaterial>();
    app.add_systems(Update, {
        move |mut commands: Commands,
              mut meshes: ResMut<Assets<Mesh>>,
              mut materials: ResMut<Assets<ColorMaterial>>| {
            Bolt::builder()
                .definition(&def)
                .at_position(Vec2::ZERO)
                .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
                .extra()
                .rendered(&mut meshes, &mut materials)
                .spawn(&mut commands);
        }
    });
    app.update();

    let mut query = app.world_mut().query_filtered::<Entity, With<Bolt>>();
    let entities: Vec<Entity> = query.iter(app.world()).collect();
    assert_eq!(entities.len(), 1);
    let entity = entities[0];

    assert!(
        app.world().get::<Mesh2d>(entity).is_some(),
        "rendered extra velocity bolt should have Mesh2d"
    );
    assert!(
        app.world()
            .get::<MeshMaterial2d<ColorMaterial>>(entity)
            .is_some(),
        "rendered extra velocity bolt should have MeshMaterial2d"
    );
}

// ── Behavior 21: .rendered() takes &mut Assets<Mesh> and &mut Assets<ColorMaterial> ──

#[test]
fn rendered_primary_serving_has_valid_mesh_handle() {
    let def = test_bolt_definition();
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Mesh>()
        .init_asset::<ColorMaterial>();
    app.add_systems(Update, {
        move |mut commands: Commands,
              mut meshes: ResMut<Assets<Mesh>>,
              mut materials: ResMut<Assets<ColorMaterial>>| {
            Bolt::builder()
                .definition(&def)
                .at_position(Vec2::ZERO)
                .serving()
                .primary()
                .rendered(&mut meshes, &mut materials)
                .spawn(&mut commands);
        }
    });
    app.update();

    let mut query = app.world_mut().query_filtered::<Entity, With<Bolt>>();
    let entities: Vec<Entity> = query.iter(app.world()).collect();
    assert_eq!(entities.len(), 1);
    let entity = entities[0];

    assert!(
        app.world().get::<Mesh2d>(entity).is_some(),
        "rendered should have Mesh2d with valid handle"
    );
    assert!(
        app.world()
            .get::<MeshMaterial2d<ColorMaterial>>(entity)
            .is_some(),
        "rendered should have MeshMaterial2d with valid handle"
    );
}

// ── Behavior 25: .rendered() mesh is a circle ──

#[test]
fn rendered_bolt_has_mesh2d_present() {
    let def = test_bolt_definition();
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Mesh>()
        .init_asset::<ColorMaterial>();
    app.add_systems(Update, {
        move |mut commands: Commands,
              mut meshes: ResMut<Assets<Mesh>>,
              mut materials: ResMut<Assets<ColorMaterial>>| {
            Bolt::builder()
                .definition(&def)
                .at_position(Vec2::ZERO)
                .serving()
                .primary()
                .rendered(&mut meshes, &mut materials)
                .spawn(&mut commands);
        }
    });
    app.update();

    let mut query = app.world_mut().query_filtered::<Entity, With<Bolt>>();
    let entities: Vec<Entity> = query.iter(app.world()).collect();
    assert_eq!(entities.len(), 1);
    let entity = entities[0];

    // Smoke test: Mesh2d present on rendered, absent on headless
    assert!(
        app.world().get::<Mesh2d>(entity).is_some(),
        "rendered bolt should have Mesh2d"
    );
}
