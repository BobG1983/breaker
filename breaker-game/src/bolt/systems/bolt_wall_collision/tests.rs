use bevy::prelude::*;
use rantzsoft_physics2d::{
    aabb::Aabb2D, collision_layers::CollisionLayers, plugin::RantzPhysics2dPlugin,
};
use rantzsoft_spatial2d::components::{GlobalPosition2D, Position2D, Spatial2D, Velocity2D};

use super::*;
use crate::{
    bolt::{
        components::{Bolt, BoltBaseSpeed, BoltRadius, PiercingRemaining},
        messages::BoltImpactWall,
        resources::BoltConfig,
    },
    effect::EffectivePiercing,
    shared::{BOLT_LAYER, GameDrawLayer, WALL_LAYER},
    wall::components::Wall,
};

// ── Helpers ──────────────────────────────────────────────────────

fn test_app() -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
        .add_plugins(RantzPhysics2dPlugin)
        .add_message::<BoltImpactWall>()
        .insert_resource(WallHitMessages::default())
        .add_systems(
            FixedUpdate,
            bolt_wall_collision
                .after(rantzsoft_physics2d::plugin::PhysicsSystems::MaintainQuadtree),
        )
        .add_systems(FixedUpdate, collect_wall_hits.after(bolt_wall_collision));
    app
}

fn bolt_param_bundle() -> (BoltBaseSpeed, BoltRadius) {
    let bc = BoltConfig::default();
    (BoltBaseSpeed(bc.base_speed), BoltRadius(bc.radius))
}

/// Accumulates one fixed timestep of overstep, then runs one update.
fn tick(app: &mut App) {
    let timestep = app.world().resource::<Time<Fixed>>().timestep();
    app.world_mut()
        .resource_mut::<Time<Fixed>>()
        .accumulate_overstep(timestep);
    app.update();
}

/// Spawns a bolt at the given position with the given velocity.
fn spawn_bolt(app: &mut App, x: f32, y: f32, vx: f32, vy: f32) -> Entity {
    let bc = BoltConfig::default();
    let pos = Vec2::new(x, y);
    app.world_mut()
        .spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(vx, vy)),
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
            Aabb2D::new(Vec2::ZERO, Vec2::splat(bc.radius)),
            CollisionLayers::new(BOLT_LAYER, WALL_LAYER),
            GameDrawLayer::Bolt,
        ))
        .id()
}

/// Spawns a bolt with `EffectivePiercing` and `PiercingRemaining` components.
fn spawn_piercing_bolt(
    app: &mut App,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    effective_piercing: u32,
    piercing_remaining: u32,
) -> Entity {
    let bc = BoltConfig::default();
    let pos = Vec2::new(x, y);
    app.world_mut()
        .spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(vx, vy)),
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
            Aabb2D::new(Vec2::ZERO, Vec2::splat(bc.radius)),
            CollisionLayers::new(BOLT_LAYER, WALL_LAYER),
            GameDrawLayer::Bolt,
            EffectivePiercing(effective_piercing),
            PiercingRemaining(piercing_remaining),
        ))
        .id()
}

/// Spawns a wall entity at the given position with the given half-extents.
fn spawn_wall(app: &mut App, x: f32, y: f32, half_width: f32, half_height: f32) -> Entity {
    let pos = Vec2::new(x, y);
    app.world_mut()
        .spawn((
            Wall,
            Aabb2D::new(Vec2::ZERO, Vec2::new(half_width, half_height)),
            CollisionLayers::new(WALL_LAYER, BOLT_LAYER),
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
            GameDrawLayer::Wall,
        ))
        .id()
}

/// Collects `BoltImpactWall` messages into a resource for test assertions.
#[derive(Resource, Default)]
struct WallHitMessages(Vec<BoltImpactWall>);

fn collect_wall_hits(
    mut reader: MessageReader<BoltImpactWall>,
    mut msgs: ResMut<WallHitMessages>,
) {
    for msg in reader.read() {
        msgs.0.push(msg.clone());
    }
}

// ── Behavior 2: bolt_wall_collision detects wall overlap and reflects velocity ──

#[test]
fn bolt_overlapping_left_wall_emits_impact_and_reflects_velocity() {
    // Spec behavior 2:
    // Given: Bolt at (-2.0, 200.0) with velocity (-400.0, 0.0) and radius 8.0,
    //        left wall at (-5.0, 200.0) with half_extents (5.0, 400.0).
    //        Bolt center is inside the wall's expanded AABB (expanded by bolt radius).
    // When: bolt_wall_collision runs
    // Then: BoltImpactWall emitted, velocity.x becomes positive (reflected off left wall)
    let mut app = test_app();

    // Wall at x=-5 with half_width=5 means wall spans x=[-10, 0].
    // Bolt at x=-2 with radius 8 means expanded AABB spans x=[-18, 8].
    // Bolt center at -2 is inside [-18, 8] => overlap.
    let wall_entity = spawn_wall(&mut app, -5.0, 200.0, 5.0, 400.0);
    let bolt_entity = spawn_bolt(&mut app, -2.0, 200.0, -400.0, 0.0);

    tick(&mut app);

    let msgs = app.world().resource::<WallHitMessages>();
    assert_eq!(
        msgs.0.len(),
        1,
        "bolt overlapping wall should emit exactly one BoltImpactWall, got {}",
        msgs.0.len()
    );
    assert_eq!(
        msgs.0[0].bolt, bolt_entity,
        "BoltImpactWall.bolt should match the overlapping bolt entity"
    );
    assert_eq!(
        msgs.0[0].wall, wall_entity,
        "BoltImpactWall.wall should match the overlapped wall entity"
    );

    let vel = app
        .world()
        .get::<Velocity2D>(bolt_entity)
        .expect("bolt should still exist");
    assert!(
        vel.0.x > 0.0,
        "bolt velocity.x should be reflected positive off left wall, got vx={}",
        vel.0.x
    );
}

// ── Behavior 9: bolt_wall_collision resets PiercingRemaining to EffectivePiercing on wall overlap ──

#[test]
fn bolt_overlapping_wall_resets_piercing_remaining() {
    // Spec behavior 9:
    // Given: Bolt overlapping a wall, with EffectivePiercing(3) and PiercingRemaining(1)
    // When: bolt_wall_collision detects wall overlap
    // Then: PiercingRemaining resets to 3 (matching EffectivePiercing.0)
    let mut app = test_app();

    // Wall at x=-5 with half_width=5, bolt at x=-2 with radius 8 => overlap
    spawn_wall(&mut app, -5.0, 200.0, 5.0, 400.0);
    let bolt_entity = spawn_piercing_bolt(
        &mut app, -2.0, 200.0, // position: inside wall's expanded AABB
        -400.0, 0.0, // velocity: moving left
        3,   // EffectivePiercing(3)
        1,   // PiercingRemaining(1) — partially spent
    );

    tick(&mut app);

    let pr = app
        .world()
        .get::<PiercingRemaining>(bolt_entity)
        .expect("PiercingRemaining should still be present on bolt");
    assert_eq!(
        pr.0, 3,
        "PiercingRemaining should reset to EffectivePiercing.0 (3) on wall overlap, got {}",
        pr.0
    );
}

/// Spec behavior 9 edge case: `PiercingRemaining` without `EffectivePiercing` stays unchanged.
#[test]
fn bolt_with_piercing_remaining_but_no_effective_piercing_unchanged_on_wall_hit() {
    let mut app = test_app();

    // Wall at x=-5 with half_width=5, bolt at x=-2 with radius 8 => overlap
    spawn_wall(&mut app, -5.0, 200.0, 5.0, 400.0);

    // Spawn bolt with PiercingRemaining but NO EffectivePiercing
    let bc = BoltConfig::default();
    let pos = Vec2::new(-2.0, 200.0);
    let bolt_entity = app
        .world_mut()
        .spawn((
            Bolt,
            bolt_param_bundle(),
            Velocity2D(Vec2::new(-400.0, 0.0)),
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
            Aabb2D::new(Vec2::ZERO, Vec2::splat(bc.radius)),
            CollisionLayers::new(BOLT_LAYER, WALL_LAYER),
            GameDrawLayer::Bolt,
            PiercingRemaining(1),
            // No EffectivePiercing
        ))
        .id();

    tick(&mut app);

    let pr = app
        .world()
        .get::<PiercingRemaining>(bolt_entity)
        .expect("PiercingRemaining should still be present on bolt");
    assert_eq!(
        pr.0, 1,
        "PiercingRemaining without EffectivePiercing should stay at 1 on wall overlap, got {}",
        pr.0
    );
}

// ── Behavior 4: bolt_wall_collision is no-op when bolt is not near any wall ──

#[test]
fn bolt_far_from_walls_emits_no_impact_and_preserves_state() {
    // Spec behavior 4:
    // Given: Bolt at (200.0, 200.0) center of playfield, no wall within bolt radius
    // When: bolt_wall_collision runs
    // Then: no BoltImpactWall emitted, position and velocity unchanged
    let mut app = test_app();

    // Walls at playfield edges, far from bolt
    spawn_wall(&mut app, -300.0, 0.0, 5.0, 400.0); // left wall
    spawn_wall(&mut app, 300.0, 0.0, 5.0, 400.0); // right wall
    spawn_wall(&mut app, 0.0, 300.0, 400.0, 5.0); // ceiling

    let bolt_entity = spawn_bolt(&mut app, 200.0, 200.0, -300.0, 100.0);

    let vel_before = app.world().get::<Velocity2D>(bolt_entity).unwrap().0;
    let pos_before = app.world().get::<Position2D>(bolt_entity).unwrap().0;

    tick(&mut app);

    let msgs = app.world().resource::<WallHitMessages>();
    assert!(
        msgs.0.is_empty(),
        "bolt far from walls should emit no BoltImpactWall, got {} messages",
        msgs.0.len()
    );

    let vel_after = app.world().get::<Velocity2D>(bolt_entity).unwrap().0;
    let pos_after = app.world().get::<Position2D>(bolt_entity).unwrap().0;
    assert_eq!(
        vel_before, vel_after,
        "velocity should be unchanged when no wall overlap: before={vel_before}, after={vel_after}"
    );
    assert_eq!(
        pos_before, pos_after,
        "position should be unchanged when no wall overlap: before={pos_before}, after={pos_after}"
    );
}

// ── Behavior 5 (simplified): bolt overlapping ceiling wall after cell bounce ──

#[test]
fn bolt_overlapping_ceiling_wall_reflects_velocity_downward() {
    // Simplified behavior 5: bolt ended up inside ceiling wall
    // (simulating post-cell-bounce overlap). bolt_wall_collision resolves.
    // Given: Bolt at (100.0, 298.0) with velocity (100.0, 300.0), radius 8.0.
    //        Ceiling wall at (0.0, 305.0) with half_extents (400.0, 5.0).
    //        Wall spans y=[300, 310]. Expanded by radius 8: y=[292, 318].
    //        Bolt center at y=298 is inside [292, 318] => overlap.
    // When: bolt_wall_collision runs
    // Then: BoltImpactWall emitted, velocity.y becomes negative (reflected off ceiling)
    let mut app = test_app();

    let wall_entity = spawn_wall(&mut app, 0.0, 305.0, 400.0, 5.0);
    let bolt_entity = spawn_bolt(&mut app, 100.0, 298.0, 100.0, 300.0);

    tick(&mut app);

    let msgs = app.world().resource::<WallHitMessages>();
    assert_eq!(
        msgs.0.len(),
        1,
        "bolt overlapping ceiling should emit exactly one BoltImpactWall, got {}",
        msgs.0.len()
    );
    assert_eq!(msgs.0[0].bolt, bolt_entity);
    assert_eq!(msgs.0[0].wall, wall_entity);

    let vel = app
        .world()
        .get::<Velocity2D>(bolt_entity)
        .expect("bolt should still exist");
    assert!(
        vel.0.y < 0.0,
        "bolt velocity.y should be reflected negative off ceiling, got vy={}",
        vel.0.y
    );
}

// ── Edge case: bolt at exact wall boundary (tangent) should not trigger ──

#[test]
fn bolt_tangent_to_wall_does_not_trigger_overlap() {
    // Edge case for behavior 2:
    // Bolt radius = 8.0. Wall half_width = 5.0 at x = -50.0.
    // Wall right edge at x = -45.0. Expanded by bolt radius: x = -37.0.
    // Bolt at x = -37.0 is exactly at the boundary — not inside.
    // No BoltImpactWall should be emitted.
    let mut app = test_app();

    // Wall spans x=[-55, -45]. Expanded by radius 8: x=[-63, -37].
    // Bolt center at x=-37 is exactly on the edge.
    spawn_wall(&mut app, -50.0, 200.0, 5.0, 400.0);
    let bolt_entity = spawn_bolt(&mut app, -37.0, 200.0, -400.0, 0.0);

    let vel_before = app.world().get::<Velocity2D>(bolt_entity).unwrap().0;

    tick(&mut app);

    let msgs = app.world().resource::<WallHitMessages>();
    assert!(
        msgs.0.is_empty(),
        "bolt tangent to wall boundary should not trigger overlap, got {} messages",
        msgs.0.len()
    );

    let vel_after = app.world().get::<Velocity2D>(bolt_entity).unwrap().0;
    assert_eq!(
        vel_before, vel_after,
        "velocity should be unchanged when bolt is tangent to wall"
    );
}
