use bevy::prelude::*;

use super::helpers::default_playfield;
use crate::{shared::GameDrawLayer, wall::components::Wall};

fn visual_test_app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Mesh>()
        .init_asset::<ColorMaterial>();
    app
}

// ── Behavior 13: .visible() stores mesh and material handles ──

#[test]
fn visible_produces_mesh_and_material_components() {
    let pf = default_playfield();
    let mut app = visual_test_app();

    app.add_systems(Update, {
        move |mut commands: Commands,
              mut meshes: ResMut<Assets<Mesh>>,
              mut materials: ResMut<Assets<ColorMaterial>>| {
            let bundle = Wall::builder()
                .left(&pf)
                .visible(&mut meshes, &mut materials)
                .build();
            commands.spawn(bundle);
        }
    });
    app.update();

    let mut query = app.world_mut().query_filtered::<Entity, With<Wall>>();
    let entities: Vec<Entity> = query.iter(app.world()).collect();
    assert_eq!(entities.len(), 1, "should have exactly 1 wall entity");
    let entity = entities[0];

    assert!(
        app.world().get::<Mesh2d>(entity).is_some(),
        "visible wall should have Mesh2d"
    );
    assert!(
        app.world()
            .get::<MeshMaterial2d<ColorMaterial>>(entity)
            .is_some(),
        "visible wall should have MeshMaterial2d<ColorMaterial>"
    );
    assert!(
        matches!(
            app.world().get::<GameDrawLayer>(entity),
            Some(GameDrawLayer::Wall)
        ),
        "visible wall should have GameDrawLayer::Wall"
    );
}

#[test]
fn visible_without_color_uses_white_fallback() {
    #[derive(Resource)]
    struct SpawnedEntity(Entity);

    let pf = default_playfield();
    let mut app = visual_test_app();

    app.add_systems(Update, {
        move |mut commands: Commands,
              mut meshes: ResMut<Assets<Mesh>>,
              mut materials: ResMut<Assets<ColorMaterial>>| {
            let bundle = Wall::builder()
                .left(&pf)
                .visible(&mut meshes, &mut materials)
                .build();
            let e = commands.spawn(bundle).id();
            commands.insert_resource(SpawnedEntity(e));
        }
    });
    app.update();

    let entity = app.world().resource::<SpawnedEntity>().0;
    let mat_handle = app
        .world()
        .get::<MeshMaterial2d<ColorMaterial>>(entity)
        .expect("should have MeshMaterial2d");
    let materials = app.world().resource::<Assets<ColorMaterial>>();
    let material = materials
        .get(&mat_handle.0)
        .expect("material handle should be valid");

    // White fallback: [1.0, 1.0, 1.0] -> Color::srgb(1.0, 1.0, 1.0)
    let expected = Color::srgb(1.0, 1.0, 1.0);
    assert_eq!(
        material.color, expected,
        "fallback color should be white (srgb 1.0, 1.0, 1.0), got {:?}",
        material.color
    );
}

// ── Behavior 14: Not calling .visible() produces no visual components ──

#[test]
fn no_visible_call_produces_no_mesh_or_material() {
    let pf = default_playfield();
    let mut world = World::new();

    let bundle = Wall::builder().left(&pf).build();
    let entity = world.spawn(bundle).id();

    assert!(
        world.get::<Mesh2d>(entity).is_none(),
        "invisible wall should NOT have Mesh2d"
    );
    assert!(
        world.get::<MeshMaterial2d<ColorMaterial>>(entity).is_none(),
        "invisible wall should NOT have MeshMaterial2d<ColorMaterial>"
    );
    // GameDrawLayer::Wall should still be present
    assert!(
        matches!(
            world.get::<GameDrawLayer>(entity),
            Some(GameDrawLayer::Wall)
        ),
        "invisible wall should still have GameDrawLayer::Wall"
    );
}

#[test]
fn invisible_call_also_produces_no_mesh_or_material() {
    let pf = default_playfield();
    let mut world = World::new();

    let bundle = Wall::builder().left(&pf).invisible().build();
    let entity = world.spawn(bundle).id();

    assert!(
        world.get::<Mesh2d>(entity).is_none(),
        "explicitly invisible wall should NOT have Mesh2d"
    );
    assert!(
        world.get::<MeshMaterial2d<ColorMaterial>>(entity).is_none(),
        "explicitly invisible wall should NOT have MeshMaterial2d<ColorMaterial>"
    );
}

// ── Behavior 15: .invisible() is a no-op for self-documentation ──

#[test]
fn invisible_returns_self_unchanged() {
    let pf = default_playfield();
    // Just verify it compiles and can proceed to build
    let _bundle = Wall::builder().left(&pf).invisible().build();
}

#[test]
fn invisible_then_visible_produces_visual_components() {
    let pf = default_playfield();
    let mut app = visual_test_app();

    app.add_systems(Update, {
        move |mut commands: Commands,
              mut meshes: ResMut<Assets<Mesh>>,
              mut materials: ResMut<Assets<ColorMaterial>>| {
            let bundle = Wall::builder()
                .left(&pf)
                .invisible()
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
        app.world().get::<Mesh2d>(entity).is_some(),
        ".invisible().visible() should produce Mesh2d"
    );
    assert!(
        app.world()
            .get::<MeshMaterial2d<ColorMaterial>>(entity)
            .is_some(),
        ".invisible().visible() should produce MeshMaterial2d"
    );
}
