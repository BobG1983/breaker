use bevy::{ecs::world::CommandQueue, prelude::*};
use rantzsoft_physics2d::{aabb::Aabb2D, collision_layers::CollisionLayers};
use rantzsoft_spatial2d::components::{Position2D, Scale2D};

use super::helpers::default_playfield;
use crate::{
    shared::{BOLT_LAYER, GameDrawLayer, WALL_LAYER},
    walls::{components::Wall, definition::WallDefinition},
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

// ── Behavior 19: build() on Left wall produces correct core components ──

#[test]
fn build_left_wall_has_wall_marker() {
    let pf = default_playfield();
    let mut world = World::new();

    let entity = spawn_in_world(&mut world, |commands| {
        Wall::builder().left(&pf).spawn(commands)
    });

    assert!(
        world.get::<Wall>(entity).is_some(),
        "should have Wall marker"
    );
}

#[test]
fn build_left_wall_has_correct_position() {
    let pf = default_playfield();
    let mut world = World::new();

    let entity = spawn_in_world(&mut world, |commands| {
        Wall::builder().left(&pf).spawn(commands)
    });

    let pos = world.get::<Position2D>(entity);
    assert!(pos.is_some(), "should have Position2D");
    let pos = pos.unwrap();
    assert!(
        (pos.0.x - (-490.0)).abs() < f32::EPSILON,
        "Position2D.x should be -490.0, got {}",
        pos.0.x
    );
    assert!(
        pos.0.y.abs() < f32::EPSILON,
        "Position2D.y should be 0.0, got {}",
        pos.0.y
    );
}

#[test]
fn build_left_wall_has_correct_scale() {
    let pf = default_playfield();
    let mut world = World::new();

    let entity = spawn_in_world(&mut world, |commands| {
        Wall::builder().left(&pf).spawn(commands)
    });

    let scale = world.get::<Scale2D>(entity);
    assert!(scale.is_some(), "should have Scale2D");
    let scale = scale.unwrap();
    assert!(
        (scale.x - 90.0).abs() < f32::EPSILON,
        "Scale2D.x should be 90.0, got {}",
        scale.x
    );
    assert!(
        (scale.y - 300.0).abs() < f32::EPSILON,
        "Scale2D.y should be 300.0, got {}",
        scale.y
    );
}

#[test]
fn build_left_wall_has_correct_aabb() {
    let pf = default_playfield();
    let mut world = World::new();

    let entity = spawn_in_world(&mut world, |commands| {
        Wall::builder().left(&pf).spawn(commands)
    });

    let aabb = world.get::<Aabb2D>(entity);
    assert!(aabb.is_some(), "should have Aabb2D");
    let aabb = aabb.unwrap();
    assert!(
        (aabb.half_extents.x - 90.0).abs() < f32::EPSILON,
        "Aabb2D.half_extents.x should be 90.0, got {}",
        aabb.half_extents.x
    );
    assert!(
        (aabb.half_extents.y - 300.0).abs() < f32::EPSILON,
        "Aabb2D.half_extents.y should be 300.0, got {}",
        aabb.half_extents.y
    );
}

#[test]
fn build_left_wall_has_collision_layers() {
    let pf = default_playfield();
    let mut world = World::new();

    let entity = spawn_in_world(&mut world, |commands| {
        Wall::builder().left(&pf).spawn(commands)
    });

    let layers = world.get::<CollisionLayers>(entity);
    assert!(layers.is_some(), "should have CollisionLayers");
    let layers = layers.unwrap();
    assert_eq!(layers.membership, WALL_LAYER);
    assert_eq!(layers.mask, BOLT_LAYER);
}

#[test]
fn build_left_wall_has_game_draw_layer() {
    let pf = default_playfield();
    let mut world = World::new();

    let entity = spawn_in_world(&mut world, |commands| {
        Wall::builder().left(&pf).spawn(commands)
    });

    assert!(
        matches!(
            world.get::<GameDrawLayer>(entity),
            Some(GameDrawLayer::Wall)
        ),
        "should have GameDrawLayer::Wall"
    );
}

// ── Behavior 20: build() on Right wall produces correct position and extents ──

#[test]
fn build_right_wall_has_correct_position_and_extents() {
    let pf = default_playfield();
    let mut world = World::new();

    let entity = spawn_in_world(&mut world, |commands| {
        Wall::builder().right(&pf).spawn(commands)
    });

    let pos = world.get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.x - 490.0).abs() < f32::EPSILON,
        "Right Position2D.x should be 490.0, got {}",
        pos.0.x
    );
    assert!(
        pos.0.y.abs() < f32::EPSILON,
        "Right Position2D.y should be 0.0"
    );

    let scale = world.get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 90.0).abs() < f32::EPSILON,
        "Right Scale2D.x should be 90.0"
    );
    assert!(
        (scale.y - 300.0).abs() < f32::EPSILON,
        "Right Scale2D.y should be 300.0"
    );

    let aabb = world.get::<Aabb2D>(entity).unwrap();
    assert!(
        (aabb.half_extents.x - 90.0).abs() < f32::EPSILON,
        "Right Aabb2D.half_extents.x should be 90.0"
    );
    assert!(
        (aabb.half_extents.y - 300.0).abs() < f32::EPSILON,
        "Right Aabb2D.half_extents.y should be 300.0"
    );
}

// ── Behavior 21: build() on Ceiling wall produces correct position and extents ──

#[test]
fn build_ceiling_wall_has_correct_position_and_extents() {
    let pf = default_playfield();
    let mut world = World::new();

    let entity = spawn_in_world(&mut world, |commands| {
        Wall::builder().ceiling(&pf).spawn(commands)
    });

    let pos = world.get::<Position2D>(entity).unwrap();
    assert!(
        pos.0.x.abs() < f32::EPSILON,
        "Ceiling Position2D.x should be 0.0, got {}",
        pos.0.x
    );
    assert!(
        (pos.0.y - 390.0).abs() < f32::EPSILON,
        "Ceiling Position2D.y should be 390.0, got {}",
        pos.0.y
    );

    let scale = world.get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 400.0).abs() < f32::EPSILON,
        "Ceiling Scale2D.x should be 400.0"
    );
    assert!(
        (scale.y - 90.0).abs() < f32::EPSILON,
        "Ceiling Scale2D.y should be 90.0"
    );

    let aabb = world.get::<Aabb2D>(entity).unwrap();
    assert!(
        (aabb.half_extents.x - 400.0).abs() < f32::EPSILON,
        "Ceiling Aabb2D.half_extents.x should be 400.0"
    );
    assert!(
        (aabb.half_extents.y - 90.0).abs() < f32::EPSILON,
        "Ceiling Aabb2D.half_extents.y should be 90.0"
    );
}

// ── Behavior 22: build() on Floor wall produces correct position and extents ──

#[test]
fn build_floor_wall_has_correct_position_and_extents() {
    let pf = default_playfield();
    let mut world = World::new();

    let entity = spawn_in_world(&mut world, |commands| {
        Wall::builder().floor(&pf).spawn(commands)
    });

    let pos = world.get::<Position2D>(entity).unwrap();
    assert!(
        pos.0.x.abs() < f32::EPSILON,
        "Floor Position2D.x should be 0.0, got {}",
        pos.0.x
    );
    assert!(
        (pos.0.y - (-300.0)).abs() < f32::EPSILON,
        "Floor Position2D.y should be -300.0, got {}",
        pos.0.y
    );

    let scale = world.get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 400.0).abs() < f32::EPSILON,
        "Floor Scale2D.x should be 400.0"
    );
    assert!(
        (scale.y - 90.0).abs() < f32::EPSILON,
        "Floor Scale2D.y should be 90.0"
    );

    let aabb = world.get::<Aabb2D>(entity).unwrap();
    assert!(
        (aabb.half_extents.x - 400.0).abs() < f32::EPSILON,
        "Floor Aabb2D.half_extents.x should be 400.0"
    );
    assert!(
        (aabb.half_extents.y - 90.0).abs() < f32::EPSILON,
        "Floor Aabb2D.half_extents.y should be 90.0"
    );
}

// ── Behavior 23: build() with definition half_thickness uses definition value ──

#[test]
fn build_left_with_definition_half_thickness_45() {
    let pf = default_playfield();
    let def = WallDefinition {
        half_thickness: 45.0,
        ..Default::default()
    };
    let mut world = World::new();

    let entity = spawn_in_world(&mut world, |commands| {
        Wall::builder().left(&pf).definition(&def).spawn(commands)
    });

    let pos = world.get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.x - (-445.0)).abs() < f32::EPSILON,
        "Left with ht=45.0: Position2D.x should be -445.0, got {}",
        pos.0.x
    );

    let scale = world.get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 45.0).abs() < f32::EPSILON,
        "Scale2D.x should be 45.0"
    );
    assert!(
        (scale.y - 300.0).abs() < f32::EPSILON,
        "Scale2D.y should be 300.0"
    );

    let aabb = world.get::<Aabb2D>(entity).unwrap();
    assert!(
        (aabb.half_extents.x - 45.0).abs() < f32::EPSILON,
        "Aabb2D.half_extents.x should be 45.0"
    );
}

#[test]
fn build_ceiling_with_definition_half_thickness_45() {
    let pf = default_playfield();
    let def = WallDefinition {
        half_thickness: 45.0,
        ..Default::default()
    };
    let mut world = World::new();

    let entity = spawn_in_world(&mut world, |commands| {
        Wall::builder()
            .ceiling(&pf)
            .definition(&def)
            .spawn(commands)
    });

    let pos = world.get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.y - 345.0).abs() < f32::EPSILON,
        "Ceiling with ht=45.0: Position2D.y should be 345.0, got {}",
        pos.0.y
    );

    let scale = world.get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 400.0).abs() < f32::EPSILON,
        "Ceiling Scale2D.x should be 400.0"
    );
    assert!(
        (scale.y - 45.0).abs() < f32::EPSILON,
        "Ceiling Scale2D.y should be 45.0"
    );
}

// ── Behavior 24: build() with override half_thickness uses override ──

#[test]
fn build_left_with_override_half_thickness_60() {
    let pf = default_playfield();
    let mut world = World::new();

    let entity = spawn_in_world(&mut world, |commands| {
        Wall::builder()
            .left(&pf)
            .with_half_thickness(60.0)
            .spawn(commands)
    });

    let pos = world.get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.x - (-460.0)).abs() < f32::EPSILON,
        "Override ht=60.0: Position2D.x should be -460.0, got {}",
        pos.0.x
    );

    let scale = world.get::<Scale2D>(entity).unwrap();
    assert!(
        (scale.x - 60.0).abs() < f32::EPSILON,
        "Scale2D.x should be 60.0"
    );
}

#[test]
fn build_left_override_beats_definition_half_thickness() {
    let pf = default_playfield();
    let def = WallDefinition {
        half_thickness: 45.0,
        ..Default::default()
    };
    let mut world = World::new();

    let entity = spawn_in_world(&mut world, |commands| {
        Wall::builder()
            .left(&pf)
            .definition(&def)
            .with_half_thickness(60.0)
            .spawn(commands)
    });

    let pos = world.get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.x - (-460.0)).abs() < f32::EPSILON,
        "Override 60.0 should win over definition 45.0: x = -460.0, got {}",
        pos.0.x
    );
}

// ── Behavior 25: build() with .visible() includes mesh and material ──

#[test]
fn build_with_visible_has_mesh_and_material() {
    let pf = default_playfield();
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default()))
        .init_asset::<Mesh>()
        .init_asset::<ColorMaterial>();

    app.add_systems(Update, {
        move |mut commands: Commands,
              mut meshes: ResMut<Assets<Mesh>>,
              mut materials: ResMut<Assets<ColorMaterial>>| {
            Wall::builder()
                .left(&pf)
                .visible(&mut meshes, &mut materials)
                .spawn(&mut commands);
        }
    });
    app.update();

    let mut query = app.world_mut().query_filtered::<Entity, With<Wall>>();
    let entities: Vec<Entity> = query.iter(app.world()).collect();
    assert_eq!(entities.len(), 1);
    let entity = entities[0];

    assert!(
        app.world().get::<Mesh2d>(entity).is_some(),
        "visible build should have Mesh2d"
    );
    assert!(
        app.world()
            .get::<MeshMaterial2d<ColorMaterial>>(entity)
            .is_some(),
        "visible build should have MeshMaterial2d"
    );
}

#[test]
fn build_without_visible_has_no_mesh_or_material() {
    let pf = default_playfield();
    let mut world = World::new();

    let entity = spawn_in_world(&mut world, |commands| {
        Wall::builder().left(&pf).spawn(commands)
    });

    assert!(
        world.get::<Mesh2d>(entity).is_none(),
        "non-visible build should NOT have Mesh2d"
    );
    assert!(
        world.get::<MeshMaterial2d<ColorMaterial>>(entity).is_none(),
        "non-visible build should NOT have MeshMaterial2d"
    );
}

// ── Behavior 26: build() on Floor with .one_shot() still produces correct position ──

#[test]
fn build_floor_one_shot_has_correct_position() {
    let pf = default_playfield();
    let mut world = World::new();

    let entity = spawn_in_world(&mut world, |commands| {
        Wall::builder().floor(&pf).one_shot().spawn(commands)
    });

    let pos = world.get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.y - (-300.0)).abs() < f32::EPSILON,
        "Floor one_shot Position2D.y should be -300.0, got {}",
        pos.0.y
    );
    assert!(
        world.get::<Wall>(entity).is_some(),
        "should have Wall marker"
    );
}

#[test]
fn build_floor_timed_has_correct_position() {
    let pf = default_playfield();
    let mut world = World::new();

    let entity = spawn_in_world(&mut world, |commands| {
        Wall::builder().floor(&pf).timed(5.0).spawn(commands)
    });

    let pos = world.get::<Position2D>(entity).unwrap();
    assert!(
        (pos.0.y - (-300.0)).abs() < f32::EPSILON,
        "Floor timed Position2D.y should be -300.0, got {}",
        pos.0.y
    );
}

// ── Behavior 27: build() always includes CollisionLayers(WALL_LAYER, BOLT_LAYER) ──

#[test]
fn build_all_sides_have_collision_layers() {
    let pf = default_playfield();
    let mut world = World::new();

    let left = spawn_in_world(&mut world, |commands| {
        Wall::builder().left(&pf).spawn(commands)
    });
    let right = spawn_in_world(&mut world, |commands| {
        Wall::builder().right(&pf).spawn(commands)
    });
    let ceiling = spawn_in_world(&mut world, |commands| {
        Wall::builder().ceiling(&pf).spawn(commands)
    });
    let floor = spawn_in_world(&mut world, |commands| {
        Wall::builder().floor(&pf).spawn(commands)
    });

    for (name, entity) in [
        ("Left", left),
        ("Right", right),
        ("Ceiling", ceiling),
        ("Floor", floor),
    ] {
        let layers = world
            .get::<CollisionLayers>(entity)
            .unwrap_or_else(|| panic!("{name} should have CollisionLayers"));
        assert_eq!(
            layers.membership, WALL_LAYER,
            "{name} membership should be WALL_LAYER"
        );
        assert_eq!(layers.mask, BOLT_LAYER, "{name} mask should be BOLT_LAYER");
    }
}

// ── Behavior 28: build() always includes GameDrawLayer::Wall ──

#[test]
fn build_all_sides_have_game_draw_layer_wall() {
    let pf = default_playfield();
    let mut world = World::new();

    let left = spawn_in_world(&mut world, |commands| {
        Wall::builder().left(&pf).spawn(commands)
    });
    let right = spawn_in_world(&mut world, |commands| {
        Wall::builder().right(&pf).spawn(commands)
    });
    let ceiling = spawn_in_world(&mut world, |commands| {
        Wall::builder().ceiling(&pf).spawn(commands)
    });
    let floor = spawn_in_world(&mut world, |commands| {
        Wall::builder().floor(&pf).spawn(commands)
    });

    for (name, entity) in [
        ("Left", left),
        ("Right", right),
        ("Ceiling", ceiling),
        ("Floor", floor),
    ] {
        assert!(
            matches!(
                world.get::<GameDrawLayer>(entity),
                Some(GameDrawLayer::Wall)
            ),
            "{name} should have GameDrawLayer::Wall"
        );
    }
}

#[test]
fn build_visible_and_invisible_both_have_game_draw_layer() {
    let pf = default_playfield();
    let mut world = World::new();

    // Invisible wall
    let invisible = spawn_in_world(&mut world, |commands| {
        Wall::builder().left(&pf).spawn(commands)
    });
    assert!(
        matches!(
            world.get::<GameDrawLayer>(invisible),
            Some(GameDrawLayer::Wall)
        ),
        "invisible wall should have GameDrawLayer::Wall"
    );
}
