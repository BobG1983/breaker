use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use super::helpers::default_playfield;
use crate::{
    shared::GameDrawLayer,
    wall::{components::Wall, definition::WallDefinition},
};

// ── Behavior 36: Side transition + definition in any order ──

#[test]
fn definition_after_side_produces_correct_components() {
    let pf = default_playfield();
    let def = WallDefinition {
        half_thickness: 45.0,
        ..Default::default()
    };
    let mut world = World::new();

    let bundle = Wall::builder().left(&pf).definition(&def).build();
    let entity = world.spawn(bundle).id();

    let pos = world.get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.x - (-445.0)).abs() < f32::EPSILON,
        "Left with def ht=45.0 should produce x=-445.0, got {}",
        pos.0.x
    );
}

#[test]
fn definition_works_on_all_four_sides() {
    let pf = default_playfield();
    let def = WallDefinition {
        half_thickness: 45.0,
        ..Default::default()
    };
    let mut world = World::new();

    // All four sides should compile with .definition()
    let _left = world
        .spawn(Wall::builder().left(&pf).definition(&def).build())
        .id();
    let _right = world
        .spawn(Wall::builder().right(&pf).definition(&def).build())
        .id();
    let _ceiling = world
        .spawn(Wall::builder().ceiling(&pf).definition(&def).build())
        .id();
    let _floor = world
        .spawn(Wall::builder().floor(&pf).definition(&def).build())
        .id();
}

// ── Behavior 37: Optional methods can be chained in any order after side ──

#[test]
fn optional_methods_chainable_in_any_order() {
    let pf = default_playfield();
    let def = WallDefinition {
        half_thickness: 45.0,
        ..Default::default()
    };
    let mut world = World::new();

    let bundle = Wall::builder()
        .left(&pf)
        .with_half_thickness(60.0)
        .with_color([1.0, 0.0, 0.0])
        .definition(&def)
        .build();
    let entity = world.spawn(bundle).id();

    let pos = world.get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.x - (-460.0)).abs() < f32::EPSILON,
        "Override ht=60.0 should win over definition ht=45.0: x=-460.0, got {}",
        pos.0.x
    );
}

#[test]
fn definition_then_overrides_then_build() {
    let pf = default_playfield();
    let def = WallDefinition {
        half_thickness: 45.0,
        ..Default::default()
    };
    let mut world = World::new();

    let bundle = Wall::builder()
        .left(&pf)
        .definition(&def)
        .with_effects(vec![])
        .with_half_thickness(60.0)
        .build();
    let entity = world.spawn(bundle).id();

    let pos = world.get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.x - (-460.0)).abs() < f32::EPSILON,
        "All overrides should apply: x=-460.0, got {}",
        pos.0.x
    );
}

// ── Behavior 38: Build without definition uses only override and default values ──

#[test]
fn build_without_definition_with_override() {
    let pf = default_playfield();
    let mut world = World::new();

    let bundle = Wall::builder().left(&pf).with_half_thickness(60.0).build();
    let entity = world.spawn(bundle).id();

    let pos = world.get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.x - (-460.0)).abs() < f32::EPSILON,
        "No definition, override ht=60.0: x=-460.0, got {}",
        pos.0.x
    );
}

#[test]
fn build_without_definition_without_override_uses_default() {
    let pf = default_playfield();
    let mut world = World::new();

    let bundle = Wall::builder().left(&pf).build();
    let entity = world.spawn(bundle).id();

    let pos = world.get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.x - (-490.0)).abs() < f32::EPSILON,
        "No definition, no override: default ht=90.0, x=-490.0, got {}",
        pos.0.x
    );
}

// ── Behavior 39: Floor wall with .one_shot() combined with definition and .visible() ──

#[test]
fn floor_one_shot_with_definition_and_visible() {
    let pf = default_playfield();
    let def = WallDefinition::default();
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Mesh>()
        .init_asset::<ColorMaterial>();

    app.add_systems(Update, {
        move |mut commands: Commands,
              mut meshes: ResMut<Assets<Mesh>>,
              mut materials: ResMut<Assets<ColorMaterial>>| {
            let bundle = Wall::builder()
                .floor(&pf)
                .definition(&def)
                .one_shot()
                .visible(&mut meshes, &mut materials)
                .build();
            commands.spawn(bundle);
        }
    });
    app.update();

    let mut query = app.world_mut().query_filtered::<Entity, With<Wall>>();
    let entities: Vec<Entity> = query.iter(app.world()).collect();
    assert_eq!(entities.len(), 1);
    let entity = entities[0];

    assert!(
        app.world().get::<Wall>(entity).is_some(),
        "should have Wall marker"
    );
    let pos = app.world().get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.y - (-300.0)).abs() < f32::EPSILON,
        "Floor position y should be -300.0, got {}",
        pos.0.y
    );
    assert!(
        app.world().get::<Mesh2d>(entity).is_some(),
        "should have Mesh2d"
    );
    assert!(
        app.world()
            .get::<MeshMaterial2d<ColorMaterial>>(entity)
            .is_some(),
        "should have MeshMaterial2d"
    );
    assert!(
        matches!(
            app.world().get::<GameDrawLayer>(entity),
            Some(GameDrawLayer::Wall)
        ),
        "should have GameDrawLayer::Wall"
    );
}

#[test]
fn floor_one_shot_before_definition_produces_same_result() {
    let pf = default_playfield();
    let def = WallDefinition::default();
    let mut world = World::new();

    let bundle = Wall::builder()
        .floor(&pf)
        .one_shot()
        .definition(&def)
        .build();
    let entity = world.spawn(bundle).id();

    let pos = world.get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.y - (-300.0)).abs() < f32::EPSILON,
        "one_shot before definition: Floor y should be -300.0, got {}",
        pos.0.y
    );
    assert!(
        world.get::<Wall>(entity).is_some(),
        "should have Wall marker"
    );
}
