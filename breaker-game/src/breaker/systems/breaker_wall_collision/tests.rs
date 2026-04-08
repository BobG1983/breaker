use bevy::{ecs::world::CommandQueue, prelude::*};
use rantzsoft_physics2d::{
    aabb::Aabb2D, collision_layers::CollisionLayers, plugin::RantzPhysics2dPlugin,
};
use rantzsoft_spatial2d::components::{GlobalPosition2D, Position2D, Spatial2D};

use super::system::*;
use crate::{
    breaker::{
        components::{BaseHeight, BaseWidth, Breaker},
        messages::BreakerImpactWall,
    },
    shared::{BREAKER_LAYER, GameDrawLayer, NodeScalingFactor, PlayfieldConfig, WALL_LAYER},
    walls::components::Wall,
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

// ── Helpers ──────────────────────────────────────────────────────

#[derive(Resource, Default)]
struct BreakerWallHitMessages(Vec<BreakerImpactWall>);

fn collect_breaker_wall_hits(
    mut reader: MessageReader<BreakerImpactWall>,
    mut msgs: ResMut<BreakerWallHitMessages>,
) {
    for msg in reader.read() {
        msgs.0.push(msg.clone());
    }
}

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(RantzPhysics2dPlugin)
        .add_message::<BreakerImpactWall>()
        .insert_resource(BreakerWallHitMessages::default())
        .add_systems(
            FixedUpdate,
            breaker_wall_collision
                .after(rantzsoft_physics2d::plugin::PhysicsSystems::MaintainQuadtree),
        )
        .add_systems(
            FixedUpdate,
            collect_breaker_wall_hits.after(breaker_wall_collision),
        );
    app
}

/// Accumulates one fixed timestep then runs one update.
fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

fn spawn_breaker(app: &mut App, pos: Vec2) -> Entity {
    app.world_mut()
        .spawn((
            Breaker,
            BaseWidth(120.0),
            BaseHeight(20.0),
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
            Aabb2D::new(Vec2::ZERO, Vec2::new(60.0, 10.0)),
            CollisionLayers::new(BREAKER_LAYER, WALL_LAYER),
            GameDrawLayer::Breaker,
        ))
        .id()
}

fn spawn_left_wall(app: &mut App) -> Entity {
    let pf = PlayfieldConfig::default();
    let entity = spawn_in_world(app.world_mut(), |commands| {
        Wall::builder().left(&pf).spawn(commands)
    });
    let pos = app.world().get::<Position2D>(entity).unwrap().0;
    app.world_mut()
        .entity_mut(entity)
        .insert(GlobalPosition2D(pos));
    entity
}

fn spawn_right_wall(app: &mut App) -> Entity {
    let pf = PlayfieldConfig::default();
    let entity = spawn_in_world(app.world_mut(), |commands| {
        Wall::builder().right(&pf).spawn(commands)
    });
    let pos = app.world().get::<Position2D>(entity).unwrap().0;
    app.world_mut()
        .entity_mut(entity)
        .insert(GlobalPosition2D(pos));
    entity
}

fn spawn_ceiling_wall(app: &mut App) -> Entity {
    let pf = PlayfieldConfig::default();
    let entity = spawn_in_world(app.world_mut(), |commands| {
        Wall::builder().ceiling(&pf).spawn(commands)
    });
    let pos = app.world().get::<Position2D>(entity).unwrap().0;
    app.world_mut()
        .entity_mut(entity)
        .insert(GlobalPosition2D(pos));
    entity
}

// ── B3: Breaker overlapping wall emits BreakerImpactWall ────────

#[test]
fn breaker_overlapping_wall_emits_impact_message() {
    // B3: Breaker at (-430,-250) half_w=60 half_h=10,
    // left wall at (-490,0) half_extents (90,300).
    // dx=|-430-(-490)|=60 < 60+90=150, dy=|-250-0|=250 < 10+300=310 => overlap => 1 message.
    let mut app = test_app();

    let breaker_entity = spawn_breaker(&mut app, Vec2::new(-430.0, -250.0));
    let wall_entity = spawn_left_wall(&mut app);

    tick(&mut app);

    let msgs = app.world().resource::<BreakerWallHitMessages>();
    assert_eq!(
        msgs.0.len(),
        1,
        "breaker overlapping wall should emit exactly 1 BreakerImpactWall, got {}",
        msgs.0.len()
    );
    assert_eq!(
        msgs.0[0].breaker, breaker_entity,
        "BreakerImpactWall.breaker should match the breaker entity"
    );
    assert_eq!(
        msgs.0[0].wall, wall_entity,
        "BreakerImpactWall.wall should match the wall entity"
    );
}

#[test]
fn breaker_with_entity_scale_overlaps_wall() {
    // B3 edge case: Breaker at (-460,-250) with NodeScalingFactor(2.0) => half_w=120 half_h=20.
    // Left wall at (-490,0) half_x=90. dx=|-460-(-490)|=30 < 120+90=210 (overlap).
    // Verify 1 message.
    let mut app = test_app();

    let breaker_entity = spawn_breaker(&mut app, Vec2::new(-460.0, -250.0));
    app.world_mut()
        .entity_mut(breaker_entity)
        .insert(NodeScalingFactor(2.0));

    let wall_entity = spawn_left_wall(&mut app);

    tick(&mut app);

    let msgs = app.world().resource::<BreakerWallHitMessages>();
    assert_eq!(
        msgs.0.len(),
        1,
        "scaled breaker should overlap wall, got {} messages",
        msgs.0.len()
    );
    assert_eq!(msgs.0[0].breaker, breaker_entity);
    assert_eq!(msgs.0[0].wall, wall_entity);
}

// ── B4: No message when breaker and wall do not overlap ─────────

#[test]
fn breaker_far_from_walls_emits_no_message() {
    // B4: Breaker at (0,-250) half_w=60, three walls far away.
    // Left wall at (-490,0): dx=490 >= 60+90=150 => no overlap.
    // Right wall at (490,0): dx=490 >= 60+90=150 => no overlap.
    // Ceiling at (0,390): dy=640 >= 10+90=100 => no overlap.
    let mut app = test_app();

    spawn_breaker(&mut app, Vec2::new(0.0, -250.0));
    spawn_left_wall(&mut app);
    spawn_right_wall(&mut app);
    spawn_ceiling_wall(&mut app);

    tick(&mut app);

    let msgs = app.world().resource::<BreakerWallHitMessages>();
    assert!(
        msgs.0.is_empty(),
        "breaker far from all walls should emit 0 BreakerImpactWall, got {}",
        msgs.0.len()
    );
}

#[test]
fn breaker_tangent_to_wall_emits_no_message() {
    // B4 edge case (tangent): Breaker at (-340,-250) half_w=60,
    // left wall at (-490,0) half_x=90. dx=|-340-(-490)|=150, threshold=60+90=150.
    // Strict inequality: 150 < 150 is false => no message.
    let mut app = test_app();

    spawn_breaker(&mut app, Vec2::new(-340.0, -250.0));
    spawn_left_wall(&mut app);

    tick(&mut app);

    let msgs = app.world().resource::<BreakerWallHitMessages>();
    assert!(
        msgs.0.is_empty(),
        "breaker tangent to wall (dx == threshold) should emit 0 messages, got {}",
        msgs.0.len()
    );
}

#[test]
fn breaker_with_entity_scale_shrink_tangent_to_wall_emits_no_message() {
    // B4 edge case (scale shrink): Breaker at (-370,-250) with NodeScalingFactor(0.5)
    // => half_w=30 half_h=5. Left wall at (-490,0) half_x=90.
    // dx=|-370-(-490)|=120, threshold=30+90=120.
    // Strict inequality: 120 < 120 is false => no message.
    let mut app = test_app();

    let breaker_entity = spawn_breaker(&mut app, Vec2::new(-370.0, -250.0));
    app.world_mut()
        .entity_mut(breaker_entity)
        .insert(NodeScalingFactor(0.5));

    spawn_left_wall(&mut app);

    tick(&mut app);

    let msgs = app.world().resource::<BreakerWallHitMessages>();
    assert!(
        msgs.0.is_empty(),
        "scaled-down breaker tangent to wall should emit 0 messages, got {}",
        msgs.0.len()
    );
}

// ── Bug Fix Regression: ActiveSizeBoosts in breaker AABB ────────
//
// The bug: breaker_wall_collision computes half_w = BaseWidth.half_width() * NodeScalingFactor
// but ignores ActiveSizeBoosts. When size boosts are active, the collision AABB is too small.

#[test]
fn breaker_wall_collision_uses_active_size_boosts_in_size() {
    // Behavior 25: Breaker at (-400,-250) with ActiveSizeBoosts([2.0]).
    // Correct: half_w = 60.0 * 2.0 = 120.0. Left wall at (-490,0) half_extents (90,300).
    // dx = |-400 - (-490)| = 90.0. threshold = 120.0 + 90.0 = 210.0. 90.0 < 210.0 => overlap.
    let mut app = test_app();

    let breaker_entity = spawn_breaker(&mut app, Vec2::new(-400.0, -250.0));
    app.world_mut().entity_mut(breaker_entity).insert(
        crate::effect::effects::size_boost::ActiveSizeBoosts(vec![2.0]),
    );

    let wall_entity = spawn_left_wall(&mut app);

    tick(&mut app);

    let msgs = app.world().resource::<BreakerWallHitMessages>();
    assert_eq!(
        msgs.0.len(),
        1,
        "breaker with ActiveSizeBoosts([2.0]) should overlap wall at dx=90 (threshold=210), got {} messages",
        msgs.0.len(),
    );
    assert_eq!(msgs.0[0].breaker, breaker_entity);
    assert_eq!(msgs.0[0].wall, wall_entity);
}

#[test]
fn breaker_wall_collision_identity_boost_still_overlaps() {
    // Behavior 25 edge case: ActiveSizeBoosts([1.0]) => half_w = 60.0 * 1.0 = 60.0.
    // Breaker at (-425,-250). Left wall at (-490,0) half_x=90.
    // dx = |-425 - (-490)| = 65.0 < 60.0 + 90.0 = 150.0 => overlap.
    let mut app = test_app();

    let breaker_entity = spawn_breaker(&mut app, Vec2::new(-425.0, -250.0));
    app.world_mut().entity_mut(breaker_entity).insert(
        crate::effect::effects::size_boost::ActiveSizeBoosts(vec![1.0]),
    );

    let wall_entity = spawn_left_wall(&mut app);

    tick(&mut app);

    let msgs = app.world().resource::<BreakerWallHitMessages>();
    assert_eq!(
        msgs.0.len(),
        1,
        "identity boost should still overlap at dx=65 (threshold=150), got {} messages",
        msgs.0.len(),
    );
    assert_eq!(msgs.0[0].breaker, breaker_entity);
    assert_eq!(msgs.0[0].wall, wall_entity);
}

#[test]
fn breaker_wall_collision_boosted_does_not_overlap_beyond_range() {
    // Behavior 26: ActiveSizeBoosts([1.5]) => half_w = 60.0 * 1.5 = 90.0.
    // Breaker at (-300,-250). Left wall at (-490,0) half_x=90.
    // dx = |-300 - (-490)| = 190.0 > 90.0 + 90.0 = 180.0 => no overlap.
    let mut app = test_app();

    let breaker_entity = spawn_breaker(&mut app, Vec2::new(-300.0, -250.0));
    app.world_mut().entity_mut(breaker_entity).insert(
        crate::effect::effects::size_boost::ActiveSizeBoosts(vec![1.5]),
    );

    spawn_left_wall(&mut app);

    tick(&mut app);

    let msgs = app.world().resource::<BreakerWallHitMessages>();
    assert!(
        msgs.0.is_empty(),
        "boosted breaker should NOT overlap wall at dx=190 (threshold=180), got {} messages",
        msgs.0.len(),
    );
}

#[test]
fn breaker_wall_collision_boosted_tangent_emits_no_message() {
    // Behavior 26 edge case: ActiveSizeBoosts([1.5]) => half_w = 90.0.
    // Breaker at (-310,-250). Left wall at (-490,0) half_x=90.
    // dx = |-310 - (-490)| = 180.0, threshold = 90.0 + 90.0 = 180.0.
    // Strict inequality: 180.0 < 180.0 is false => 0 messages.
    let mut app = test_app();

    let breaker_entity = spawn_breaker(&mut app, Vec2::new(-310.0, -250.0));
    app.world_mut().entity_mut(breaker_entity).insert(
        crate::effect::effects::size_boost::ActiveSizeBoosts(vec![1.5]),
    );

    spawn_left_wall(&mut app);

    tick(&mut app);

    let msgs = app.world().resource::<BreakerWallHitMessages>();
    assert!(
        msgs.0.is_empty(),
        "boosted breaker tangent to wall (dx == threshold=180) should emit 0 messages, got {}",
        msgs.0.len(),
    );
}

#[test]
fn breaker_wall_collision_boost_and_node_scale_combined() {
    // Behavior 27: ActiveSizeBoosts([1.5]), NodeScalingFactor(2.0).
    // half_w = 60.0 * 1.5 * 2.0 = 180.0. Breaker at (-350,-250). Left wall at (-490,0) half_x=90.
    // dx = |-350 - (-490)| = 140.0. threshold = 180.0 + 90.0 = 270.0. 140.0 < 270.0 => overlap.
    let mut app = test_app();

    let breaker_entity = spawn_breaker(&mut app, Vec2::new(-350.0, -250.0));
    app.world_mut().entity_mut(breaker_entity).insert((
        crate::effect::effects::size_boost::ActiveSizeBoosts(vec![1.5]),
        NodeScalingFactor(2.0),
    ));

    let wall_entity = spawn_left_wall(&mut app);

    tick(&mut app);

    let msgs = app.world().resource::<BreakerWallHitMessages>();
    assert_eq!(
        msgs.0.len(),
        1,
        "breaker with boost+scale should overlap wall at dx=140 (threshold=270), got {} messages",
        msgs.0.len(),
    );

    assert_eq!(msgs.0[0].breaker, breaker_entity);
    assert_eq!(msgs.0[0].wall, wall_entity);
}
