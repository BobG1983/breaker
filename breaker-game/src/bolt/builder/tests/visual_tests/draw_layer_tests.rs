use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;

use super::helpers::test_bolt_definition;
use crate::{bolt::components::Bolt, shared::GameDrawLayer};

// ── Behavior 27: Rendered bolt has GameDrawLayer::Bolt ──

#[test]
fn rendered_primary_serving_has_game_draw_layer() {
    let def = test_bolt_definition();
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Mesh>()
        .init_asset::<ColorMaterial>();
    app.add_systems(Update, {
        move |mut commands: Commands,
              mut meshes: ResMut<Assets<Mesh>>,
              mut materials: ResMut<Assets<ColorMaterial>>| {
            let bundle = Bolt::builder()
                .definition(&def)
                .at_position(Vec2::new(0.0, 50.0))
                .serving()
                .primary()
                .rendered(&mut meshes, &mut materials)
                .build();
            commands.spawn(bundle);
        }
    });
    app.update();

    let mut query = app.world_mut().query_filtered::<Entity, With<Bolt>>();
    let entities: Vec<Entity> = query.iter(app.world()).collect();
    assert_eq!(entities.len(), 1);
    let entity = entities[0];

    let layer = app
        .world()
        .get::<GameDrawLayer>(entity)
        .expect("rendered bolt should have GameDrawLayer");
    assert!(
        matches!(layer, GameDrawLayer::Bolt),
        "GameDrawLayer should be Bolt"
    );
}

#[test]
fn rendered_extra_velocity_has_game_draw_layer() {
    let def = test_bolt_definition();
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Mesh>()
        .init_asset::<ColorMaterial>();
    app.add_systems(Update, {
        move |mut commands: Commands,
              mut meshes: ResMut<Assets<Mesh>>,
              mut materials: ResMut<Assets<ColorMaterial>>| {
            let bundle = Bolt::builder()
                .definition(&def)
                .at_position(Vec2::ZERO)
                .with_velocity(Velocity2D(Vec2::new(0.0, 400.0)))
                .extra()
                .rendered(&mut meshes, &mut materials)
                .build();
            commands.spawn(bundle);
        }
    });
    app.update();

    let mut query = app.world_mut().query_filtered::<Entity, With<Bolt>>();
    let entities: Vec<Entity> = query.iter(app.world()).collect();
    assert_eq!(entities.len(), 1);
    let entity = entities[0];

    let layer = app
        .world()
        .get::<GameDrawLayer>(entity)
        .expect("rendered extra velocity bolt should have GameDrawLayer");
    assert!(
        matches!(layer, GameDrawLayer::Bolt),
        "GameDrawLayer should be Bolt"
    );
}
