use bevy::prelude::*;
use rantzsoft_physics2d::{
    aabb::Aabb2D, collision_layers::CollisionLayers, plugin::RantzPhysics2dPlugin,
};
use rantzsoft_spatial2d::components::{GlobalPosition2D, Position2D, Spatial2D, Velocity2D};

use super::*;
use crate::{
    bolt::{
        components::{Bolt, ImpactSide, LastImpact, PiercingRemaining},
        messages::BoltImpactWall,
        resources::BoltConfig,
    },
    effect::effects::piercing::ActivePiercings,
    shared::{BOLT_LAYER, GameDrawLayer, WALL_LAYER},
    wall::components::Wall,
};

// ── Helpers ──────────────────────────────────────────────

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
    Bolt::builder()
        .at_position(Vec2::new(x, y))
        .config(&BoltConfig::default())
        .with_velocity(Velocity2D(Vec2::new(vx, vy)))
        .primary()
        .spawn(app.world_mut())
}

/// Spawns a bolt with `ActivePiercings` and `PiercingRemaining` components.
fn spawn_piercing_bolt(
    app: &mut App,
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    active_piercings: Vec<u32>,
    piercing_remaining: u32,
) -> Entity {
    let entity = Bolt::builder()
        .at_position(Vec2::new(x, y))
        .config(&BoltConfig::default())
        .with_velocity(Velocity2D(Vec2::new(vx, vy)))
        .primary()
        .spawn(app.world_mut());
    app.world_mut().entity_mut(entity).insert((
        ActivePiercings(active_piercings),
        PiercingRemaining(piercing_remaining),
    ));
    entity
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

fn collect_wall_hits(mut reader: MessageReader<BoltImpactWall>, mut msgs: ResMut<WallHitMessages>) {
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

// ── Behavior 9: bolt_wall_collision resets PiercingRemaining to ActivePiercings.total() on wall overlap ──

#[test]
fn bolt_overlapping_wall_resets_piercing_remaining() {
    // Given: Bolt overlapping a wall, with ActivePiercings(vec![3]) and PiercingRemaining(1)
    // When: bolt_wall_collision detects wall overlap
    // Then: PiercingRemaining resets to 3 (matching ActivePiercings.total())
    let mut app = test_app();

    // Wall at x=-5 with half_width=5, bolt at x=-2 with radius 8 => overlap
    spawn_wall(&mut app, -5.0, 200.0, 5.0, 400.0);
    let bolt_entity = spawn_piercing_bolt(
        &mut app,
        -2.0,
        200.0, // position: inside wall's expanded AABB
        -400.0,
        0.0,     // velocity: moving left
        vec![3], // ActivePiercings(vec![3])
        1,       // PiercingRemaining(1) — partially spent
    );

    tick(&mut app);

    let pr = app
        .world()
        .get::<PiercingRemaining>(bolt_entity)
        .expect("PiercingRemaining should still be present on bolt");
    assert_eq!(
        pr.0, 3,
        "PiercingRemaining should reset to ActivePiercings.total() (3) on wall overlap, got {}",
        pr.0
    );
}

/// Edge case: `PiercingRemaining` without `ActivePiercings` stays unchanged.
#[test]
fn bolt_with_piercing_remaining_but_no_active_piercings_unchanged_on_wall_hit() {
    let mut app = test_app();

    // Wall at x=-5 with half_width=5, bolt at x=-2 with radius 8 => overlap
    spawn_wall(&mut app, -5.0, 200.0, 5.0, 400.0);

    // Spawn bolt with PiercingRemaining but NO ActivePiercings
    let bolt_entity = spawn_bolt(&mut app, -2.0, 200.0, -400.0, 0.0);
    app.world_mut()
        .entity_mut(bolt_entity)
        .insert(PiercingRemaining(1));

    tick(&mut app);

    let pr = app
        .world()
        .get::<PiercingRemaining>(bolt_entity)
        .expect("PiercingRemaining should still be present on bolt");
    assert_eq!(
        pr.0, 1,
        "PiercingRemaining without ActivePiercings should stay at 1 on wall overlap, got {}",
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

// ── Behavior 7: left wall rebound stamps LastImpact with ImpactSide::Left ──

#[test]
fn left_wall_rebound_stamps_last_impact_with_left_side() {
    // Given: Bolt at (-2.0, 200.0) with velocity (-400.0, 0.0) and radius 8.0,
    //        left wall at (-5.0, 200.0) with half_extents (5.0, 400.0).
    //        Bolt center is inside the wall's expanded AABB.
    // When: bolt_wall_collision runs for one fixed tick
    // Then: Bolt has LastImpact with side: ImpactSide::Left
    let mut app = test_app();

    spawn_wall(&mut app, -5.0, 200.0, 5.0, 400.0);
    let bolt_entity = spawn_bolt(&mut app, -2.0, 200.0, -400.0, 0.0);

    tick(&mut app);

    let last_impact = app
        .world()
        .get::<LastImpact>(bolt_entity)
        .expect("bolt should have LastImpact after left wall rebound");
    assert_eq!(
        last_impact.side,
        ImpactSide::Left,
        "left wall rebound should stamp ImpactSide::Left, got {:?}",
        last_impact.side
    );
}

#[test]
fn left_wall_tangent_bolt_does_not_stamp_last_impact() {
    // Edge case: Bolt tangent to wall boundary (exactly on expanded edge).
    // No collision, no LastImpact.
    let mut app = test_app();

    // Wall spans x=[-10, 0]. Expanded by radius 8: x=[-18, 8].
    // Bolt center at x=8 is exactly on the edge — not inside.
    spawn_wall(&mut app, -5.0, 200.0, 5.0, 400.0);
    let bolt_entity = spawn_bolt(&mut app, 8.0, 200.0, -400.0, 0.0);

    tick(&mut app);

    let last_impact = app.world().get::<LastImpact>(bolt_entity);
    assert!(
        last_impact.is_none(),
        "bolt tangent to wall should NOT get LastImpact, got {last_impact:?}"
    );
}

// ── Behavior 8: right wall rebound stamps LastImpact with ImpactSide::Right ──

#[test]
fn right_wall_rebound_stamps_last_impact_with_right_side() {
    // Given: Bolt at (2.0, 200.0) with velocity (400.0, 0.0) and radius 8.0,
    //        right wall at (5.0, 200.0) with half_extents (5.0, 400.0).
    //        Bolt center is inside the wall's expanded AABB.
    // When: bolt_wall_collision runs for one fixed tick
    // Then: Bolt has LastImpact with side: ImpactSide::Right
    let mut app = test_app();

    spawn_wall(&mut app, 5.0, 200.0, 5.0, 400.0);
    let bolt_entity = spawn_bolt(&mut app, 2.0, 200.0, 400.0, 0.0);

    tick(&mut app);

    let last_impact = app
        .world()
        .get::<LastImpact>(bolt_entity)
        .expect("bolt should have LastImpact after right wall rebound");
    assert_eq!(
        last_impact.side,
        ImpactSide::Right,
        "right wall rebound should stamp ImpactSide::Right, got {:?}",
        last_impact.side
    );
}

// ── Behavior 9: ceiling wall rebound stamps LastImpact with ImpactSide::Top ──

#[test]
fn ceiling_wall_rebound_stamps_last_impact_with_top_side() {
    // Given: Bolt at (100.0, 298.0) with velocity (100.0, 300.0) and radius 8.0,
    //        ceiling wall at (0.0, 305.0) with half_extents (400.0, 5.0).
    //        Wall spans y=[300, 310]. Expanded by radius 8: y=[292, 318].
    //        Bolt at y=298 is inside.
    // When: bolt_wall_collision runs for one fixed tick
    // Then: Bolt has LastImpact with side: ImpactSide::Top
    let mut app = test_app();

    spawn_wall(&mut app, 0.0, 305.0, 400.0, 5.0);
    let bolt_entity = spawn_bolt(&mut app, 100.0, 298.0, 100.0, 300.0);

    tick(&mut app);

    let last_impact = app
        .world()
        .get::<LastImpact>(bolt_entity)
        .expect("bolt should have LastImpact after ceiling wall rebound");
    assert_eq!(
        last_impact.side,
        ImpactSide::Top,
        "ceiling wall rebound should stamp ImpactSide::Top, got {:?}",
        last_impact.side
    );
}

#[test]
fn ceiling_wall_diagonal_bolt_side_determined_by_nearest_face() {
    // Edge case: Bolt moving diagonally (100.0, 300.0) — side is determined by
    // nearest face, not velocity direction.
    let mut app = test_app();

    spawn_wall(&mut app, 0.0, 305.0, 400.0, 5.0);
    let bolt_entity = spawn_bolt(&mut app, 100.0, 298.0, 100.0, 300.0);

    tick(&mut app);

    let last_impact = app
        .world()
        .get::<LastImpact>(bolt_entity)
        .expect("bolt should have LastImpact after ceiling wall rebound");
    assert_eq!(
        last_impact.side,
        ImpactSide::Top,
        "ceiling wall side should be ImpactSide::Top regardless of diagonal velocity, got {:?}",
        last_impact.side
    );
}

// ── Behavior 13: each new rebound overwrites the previous LastImpact ──

#[test]
fn wall_rebound_overwrites_previous_last_impact() {
    // Given: Bolt with pre-existing LastImpact { position: (50.0, 300.0), side: Top }.
    //        Bolt at (-2.0, 200.0) with velocity (-400.0, 0.0), overlapping left wall.
    // When: bolt_wall_collision runs for one fixed tick
    // Then: LastImpact is overwritten with new wall collision data.
    //       side is ImpactSide::Left, old values are gone.
    let mut app = test_app();

    spawn_wall(&mut app, -5.0, 200.0, 5.0, 400.0);
    let bolt_entity = spawn_bolt(&mut app, -2.0, 200.0, -400.0, 0.0);

    // Insert a pre-existing LastImpact
    app.world_mut().entity_mut(bolt_entity).insert(LastImpact {
        position: Vec2::new(50.0, 300.0),
        side: ImpactSide::Top,
    });

    tick(&mut app);

    let last_impact = app
        .world()
        .get::<LastImpact>(bolt_entity)
        .expect("bolt should have LastImpact after wall rebound");
    assert_eq!(
        last_impact.side,
        ImpactSide::Left,
        "wall rebound should overwrite side from Top to Left, got {:?}",
        last_impact.side
    );
    assert!(
        (last_impact.position.y - 200.0).abs() < 10.0,
        "wall rebound should overwrite position from (50, 300) — new position.y should be near 200.0, got {:?}",
        last_impact.position
    );
    assert!(
        (last_impact.position.x - 50.0).abs() > 0.01,
        "wall rebound should overwrite position.x from old value 50.0, got {}",
        last_impact.position.x
    );
}

// ── Behavior 14: bolt with no collision does not gain LastImpact ──

#[test]
fn bolt_with_no_collision_does_not_gain_last_impact() {
    // Given: Bolt at (200.0, 200.0) with velocity (-300.0, 100.0), no walls nearby.
    //        No pre-existing LastImpact on the bolt.
    // When: bolt_wall_collision runs for one fixed tick
    // Then: Bolt entity does NOT have a LastImpact component
    let mut app = test_app();

    // Walls far away
    spawn_wall(&mut app, -300.0, 0.0, 5.0, 400.0);
    spawn_wall(&mut app, 300.0, 0.0, 5.0, 400.0);

    let bolt_entity = spawn_bolt(&mut app, 200.0, 200.0, -300.0, 100.0);

    tick(&mut app);

    let last_impact = app.world().get::<LastImpact>(bolt_entity);
    assert!(
        last_impact.is_none(),
        "bolt with no wall collision should NOT get LastImpact, got {last_impact:?}"
    );
}

#[test]
fn bolt_with_zero_velocity_does_not_gain_last_impact() {
    // Edge case: Bolt with zero velocity — no collision possible, no LastImpact.
    let mut app = test_app();

    // Wall nearby but bolt has zero velocity
    spawn_wall(&mut app, -5.0, 200.0, 5.0, 400.0);
    let bolt_entity = spawn_bolt(&mut app, 200.0, 200.0, 0.0, 0.0);

    tick(&mut app);

    let last_impact = app.world().get::<LastImpact>(bolt_entity);
    assert!(
        last_impact.is_none(),
        "bolt with zero velocity should NOT get LastImpact, got {last_impact:?}"
    );
}

// ── Behavior 5: bolt_wall_collision resets PiercingRemaining from ActivePiercings.total() ──

/// Given: Bolt with `ActivePiercings(vec![2, 1])`, `PiercingRemaining(0)`, NO `EffectivePiercing`.
/// When: bolt hits wall.
/// Then: `PiercingRemaining` = 3 (2 + 1).
///
/// Fails at RED because production reads `EffectivePiercing` (absent -> no reset).
#[test]
fn bolt_wall_collision_resets_piercing_remaining_from_active_piercings_total() {
    let mut app = test_app();

    // Wall at x=-5 with half_width=5, bolt at x=-2 with radius 8 => overlap
    spawn_wall(&mut app, -5.0, 200.0, 5.0, 400.0);
    let bolt_entity = spawn_piercing_bolt(
        &mut app,
        -2.0,
        200.0, // position: inside wall's expanded AABB
        -400.0,
        0.0,        // velocity: moving left
        vec![2, 1], // ActivePiercings(vec![2, 1]) -> total = 3
        0,          // PiercingRemaining(0)
    );

    tick(&mut app);

    let pr = app
        .world()
        .get::<PiercingRemaining>(bolt_entity)
        .expect("PiercingRemaining should still be present on bolt");
    assert_eq!(
        pr.0, 3,
        "PiercingRemaining should reset to ActivePiercings.total() (2 + 1 = 3) on wall overlap, got {}",
        pr.0
    );
}
