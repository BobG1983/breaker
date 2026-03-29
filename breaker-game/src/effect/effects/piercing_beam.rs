//! Fast-expanding beam rectangle in the bolt's velocity direction.

use bevy::prelude::*;
use rantzsoft_physics2d::{
    aabb::Aabb2D, collision_layers::CollisionLayers, plugin::PhysicsSystems,
    resources::CollisionQuadtree,
};
use rantzsoft_spatial2d::components::{GlobalPosition2D, Position2D, Velocity2D};

use crate::{
    bolt::BASE_BOLT_DAMAGE,
    cells::messages::DamageCell,
    shared::{CELL_LAYER, CleanupOnNodeExit, PlayfieldConfig},
};

/// Deferred request for piercing beam instant damage.
///
/// Spawned by `fire()` with pre-computed beam geometry,
/// consumed (and despawned) by `process_piercing_beam` in the same or next tick.
#[derive(Component)]
pub struct PiercingBeamRequest {
    /// Beam origin (entity position).
    pub origin: Vec2,
    /// Normalized beam direction.
    pub direction: Vec2,
    /// Beam length (distance to playfield boundary).
    pub length: f32,
    /// Beam half-width.
    pub half_width: f32,
    /// Pre-calculated damage per cell.
    pub damage: f32,
}

pub(crate) fn fire(entity: Entity, damage_mult: f32, width: f32, world: &mut World) {
    let pos = world
        .get::<Position2D>(entity)
        .map(|p| p.0)
        .or_else(|| {
            world
                .get::<Transform>(entity)
                .map(|t| t.translation.truncate())
        })
        .unwrap_or(Vec2::ZERO);

    let velocity = world.get::<Velocity2D>(entity).map_or(Vec2::ZERO, |v| v.0);

    let mut dir = velocity.normalize_or_zero();
    if dir == Vec2::ZERO {
        dir = Vec2::Y;
    }
    let playfield = world.resource::<PlayfieldConfig>();

    // Compute distance to each playfield boundary along the beam direction.
    let mut min_t = f32::MAX;
    if dir.x > f32::EPSILON {
        min_t = min_t.min((playfield.right() - pos.x) / dir.x);
    } else if dir.x < -f32::EPSILON {
        min_t = min_t.min((playfield.left() - pos.x) / dir.x);
    }
    if dir.y > f32::EPSILON {
        min_t = min_t.min((playfield.top() - pos.y) / dir.y);
    } else if dir.y < -f32::EPSILON {
        min_t = min_t.min((playfield.bottom() - pos.y) / dir.y);
    }
    let beam_length = min_t.max(0.0);

    world.spawn((
        PiercingBeamRequest {
            origin: pos,
            direction: dir,
            length: beam_length,
            half_width: width / 2.0,
            damage: BASE_BOLT_DAMAGE * damage_mult,
        },
        CleanupOnNodeExit,
    ));
}

pub(crate) fn reverse(_entity: Entity, world: &mut World) {
    let _ = world;
}

/// Process all pending piercing beam requests: query quadtree, send damage, despawn request.
///
/// For each request, constructs the beam's bounding AABB, queries the quadtree
/// for candidate cells, performs narrow-phase filtering against the oriented beam
/// rectangle, sends [`DamageCell`] for each intersecting cell, then despawns the
/// request entity.
pub fn process_piercing_beam(
    mut commands: Commands,
    requests: Query<(Entity, &PiercingBeamRequest)>,
    quadtree: Res<CollisionQuadtree>,
    positions: Query<&GlobalPosition2D>,
    mut damage_writer: MessageWriter<DamageCell>,
) {
    let query_layers = CollisionLayers::new(0, CELL_LAYER);

    for (entity, request) in &requests {
        let dir = request.direction;
        let origin = request.origin;
        let length = request.length;
        let hw = request.half_width;

        // Compute the beam's oriented bounding box as an AABB for broad-phase.
        let end = origin + dir * length;
        let perp = Vec2::new(-dir.y, dir.x);

        let corners = [
            origin + perp * hw,
            origin - perp * hw,
            end + perp * hw,
            end - perp * hw,
        ];

        let min_x = corners.iter().map(|c| c.x).fold(f32::INFINITY, f32::min);
        let max_x = corners
            .iter()
            .map(|c| c.x)
            .fold(f32::NEG_INFINITY, f32::max);
        let min_y = corners.iter().map(|c| c.y).fold(f32::INFINITY, f32::min);
        let max_y = corners
            .iter()
            .map(|c| c.y)
            .fold(f32::NEG_INFINITY, f32::max);

        let bounding_aabb = Aabb2D::from_min_max(Vec2::new(min_x, min_y), Vec2::new(max_x, max_y));

        let candidates = quadtree
            .quadtree
            .query_aabb_filtered(&bounding_aabb, query_layers);

        // Narrow-phase: check each candidate against the oriented beam rectangle.
        for cell in candidates {
            let cell_pos = positions.get(cell).map_or(Vec2::ZERO, |p| p.0);

            let to_cell = cell_pos - origin;
            let along = to_cell.dot(dir);
            if along < 0.0 || along > length {
                continue;
            }
            let perp_dist = (to_cell - dir * along).length();
            if perp_dist > hw {
                continue;
            }

            damage_writer.write(DamageCell {
                cell,
                damage: request.damage,
                source_chip: None,
            });
        }

        commands.entity(entity).despawn();
    }
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        process_piercing_beam.after(PhysicsSystems::MaintainQuadtree),
    );
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use rantzsoft_physics2d::{
        aabb::Aabb2D, collision_layers::CollisionLayers, plugin::RantzPhysics2dPlugin,
    };
    use rantzsoft_spatial2d::components::{GlobalPosition2D, Position2D, Spatial2D, Velocity2D};

    use super::*;
    use crate::{
        bolt::BASE_BOLT_DAMAGE,
        cells::{components::Cell, messages::DamageCell},
        shared::{BOLT_LAYER, CELL_LAYER, CleanupOnNodeExit, PlayfieldConfig, WALL_LAYER},
    };

    // ── Test helpers ────────────────────────────────────────────────

    fn piercing_beam_fire_world() -> World {
        let mut world = World::new();
        world.insert_resource(PlayfieldConfig::default());
        world
    }

    /// Accumulates one fixed timestep then runs one update (ensures quadtree maintenance runs).
    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn spawn_test_cell(app: &mut App, x: f32, y: f32) -> Entity {
        let pos = Vec2::new(x, y);
        app.world_mut()
            .spawn((
                Cell,
                Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
                CollisionLayers::new(CELL_LAYER, 0),
                Position2D(pos),
                GlobalPosition2D(pos),
                Spatial2D,
            ))
            .id()
    }

    /// Collects [`DamageCell`] messages into a resource for test assertions.
    #[derive(Resource, Default)]
    struct DamageCellCollector(Vec<DamageCell>);

    fn collect_damage_cells(
        mut reader: MessageReader<DamageCell>,
        mut collector: ResMut<DamageCellCollector>,
    ) {
        for msg in reader.read() {
            collector.0.push(msg.clone());
        }
    }

    fn piercing_beam_damage_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzPhysics2dPlugin);
        app.add_message::<DamageCell>();
        app.insert_resource(DamageCellCollector::default());
        app.insert_resource(PlayfieldConfig::default());
        app.add_systems(Update, process_piercing_beam);
        app.add_systems(Update, collect_damage_cells.after(process_piercing_beam));
        app
    }

    // ── Behavior 16: fire() spawns PiercingBeamRequest with correct beam geometry ──

    #[test]
    fn fire_spawns_request_with_correct_upward_beam_geometry() {
        let mut world = piercing_beam_fire_world();

        let entity = world
            .spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                Velocity2D(Vec2::new(0.0, 400.0)),
            ))
            .id();

        fire(entity, 1.0, 20.0, &mut world);

        let mut query = world.query::<&PiercingBeamRequest>();
        let results: Vec<_> = query.iter(&world).collect();
        assert_eq!(results.len(), 1, "expected one PiercingBeamRequest entity");

        let request = results[0];
        assert!(
            (request.origin.x - 0.0).abs() < f32::EPSILON,
            "origin x should be 0.0, got {}",
            request.origin.x
        );
        assert!(
            (request.origin.y - 0.0).abs() < f32::EPSILON,
            "origin y should be 0.0, got {}",
            request.origin.y
        );
        assert!(
            (request.direction.x - 0.0).abs() < 0.01,
            "direction x should be 0.0, got {}",
            request.direction.x
        );
        assert!(
            (request.direction.y - 1.0).abs() < 0.01,
            "direction y should be 1.0, got {}",
            request.direction.y
        );
        // PlayfieldConfig default: top = 300.0. From (0,0) upward, length = 300.0
        assert!(
            (request.length - 300.0).abs() < 0.01,
            "length should be 300.0 (to top boundary), got {}",
            request.length
        );
        assert!(
            (request.half_width - 10.0).abs() < f32::EPSILON,
            "half_width should be 10.0 (width/2), got {}",
            request.half_width
        );
        let expected_damage = BASE_BOLT_DAMAGE * 1.0;
        assert!(
            (request.damage - expected_damage).abs() < f32::EPSILON,
            "damage should be {}, got {}",
            expected_damage,
            request.damage
        );
    }

    #[test]
    fn fire_entity_near_boundary_produces_short_beam() {
        let mut world = piercing_beam_fire_world();

        let entity = world
            .spawn((
                Transform::from_xyz(0.0, 290.0, 0.0),
                Velocity2D(Vec2::new(0.0, 400.0)),
            ))
            .id();

        fire(entity, 1.0, 20.0, &mut world);

        let mut query = world.query::<&PiercingBeamRequest>();
        let results: Vec<_> = query.iter(&world).collect();
        assert_eq!(results.len(), 1);

        let request = results[0];
        // top = 300.0, entity at y=290 -> beam length = 10.0
        assert!(
            (request.length - 10.0).abs() < 0.01,
            "beam near boundary should have short length, got {}",
            request.length
        );
    }

    // ── Behavior 17: fire() computes beam length in negative direction ──

    #[test]
    fn fire_computes_beam_length_to_bottom_boundary() {
        let mut world = piercing_beam_fire_world();

        let entity = world
            .spawn((
                Transform::from_xyz(0.0, 200.0, 0.0),
                Velocity2D(Vec2::new(0.0, -400.0)),
            ))
            .id();

        fire(entity, 1.0, 20.0, &mut world);

        let mut query = world.query::<&PiercingBeamRequest>();
        let results: Vec<_> = query.iter(&world).collect();
        assert_eq!(results.len(), 1);

        let request = results[0];
        assert!(
            (request.direction.x - 0.0).abs() < 0.01,
            "direction x should be 0.0"
        );
        assert!(
            (request.direction.y - (-1.0)).abs() < 0.01,
            "direction y should be -1.0, got {}",
            request.direction.y
        );
        // bottom = -300.0, entity at y=200 -> distance = 500.0
        assert!(
            (request.length - 500.0).abs() < 0.01,
            "beam length should be 500.0, got {}",
            request.length
        );
        assert!(
            (request.origin.y - 200.0).abs() < f32::EPSILON,
            "origin y should be 200.0"
        );
    }

    // ── Behavior 18: fire() handles diagonal velocity ──

    #[test]
    fn fire_handles_diagonal_velocity_direction() {
        let mut world = piercing_beam_fire_world();

        let entity = world
            .spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                Velocity2D(Vec2::new(300.0, 300.0)),
            ))
            .id();

        fire(entity, 1.0, 30.0, &mut world);

        let mut query = world.query::<&PiercingBeamRequest>();
        let results: Vec<_> = query.iter(&world).collect();
        assert_eq!(results.len(), 1);

        let request = results[0];
        // Normalized (300, 300) -> approximately (0.707, 0.707)
        let expected_dir = Vec2::new(300.0, 300.0).normalize();
        assert!(
            (request.direction.x - expected_dir.x).abs() < 0.01,
            "direction x should be ~0.707, got {}",
            request.direction.x
        );
        assert!(
            (request.direction.y - expected_dir.y).abs() < 0.01,
            "direction y should be ~0.707, got {}",
            request.direction.y
        );
        assert!(
            (request.half_width - 15.0).abs() < f32::EPSILON,
            "half_width should be 15.0, got {}",
            request.half_width
        );
        // Beam should extend to whichever boundary is hit first along diagonal
        // From (0,0) at 45 degrees: right=400 -> t_x = 400/0.707 ~ 565.7
        //                             top=300 -> t_y = 300/0.707 ~ 424.3
        // min(565.7, 424.3) ~ 424.26
        assert!(
            (request.length - 424.26).abs() < 1.0,
            "beam length should be ~424.26 (top boundary hit first at 45 degrees), got {}",
            request.length
        );
    }

    // ── Behavior 19: fire() applies damage_mult ──

    #[test]
    fn fire_applies_damage_mult_to_base_bolt_damage() {
        let mut world = piercing_beam_fire_world();

        let entity = world
            .spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                Velocity2D(Vec2::new(0.0, 400.0)),
            ))
            .id();

        fire(entity, 3.0, 20.0, &mut world);

        let mut query = world.query::<&PiercingBeamRequest>();
        let results: Vec<_> = query.iter(&world).collect();
        assert_eq!(results.len(), 1);

        let expected_damage = BASE_BOLT_DAMAGE * 3.0;
        assert!(
            (results[0].damage - expected_damage).abs() < f32::EPSILON,
            "damage should be {}, got {}",
            expected_damage,
            results[0].damage
        );
    }

    #[test]
    fn fire_with_zero_damage_mult_produces_zero_damage() {
        let mut world = piercing_beam_fire_world();

        let entity = world
            .spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                Velocity2D(Vec2::new(0.0, 400.0)),
            ))
            .id();

        fire(entity, 0.0, 20.0, &mut world);

        let mut query = world.query::<&PiercingBeamRequest>();
        let results: Vec<_> = query.iter(&world).collect();
        assert_eq!(results.len(), 1);

        assert!(
            (results[0].damage - 0.0).abs() < f32::EPSILON,
            "damage_mult=0.0 should produce damage 0.0, got {}",
            results[0].damage
        );
    }

    // ── Behavior 20: fire() with missing Velocity2D defaults to Vec2::Y ──

    #[test]
    fn fire_with_missing_velocity_defaults_direction_to_y() {
        let mut world = piercing_beam_fire_world();

        let entity = world.spawn(Transform::from_xyz(0.0, 0.0, 0.0)).id();

        fire(entity, 1.0, 20.0, &mut world);

        let mut query = world.query::<&PiercingBeamRequest>();
        let results: Vec<_> = query.iter(&world).collect();
        assert_eq!(
            results.len(),
            1,
            "request should be spawned even without Velocity2D"
        );

        let request = results[0];
        assert!(
            (request.direction.x - 0.0).abs() < 0.01,
            "missing velocity should default direction x to 0.0"
        );
        assert!(
            (request.direction.y - 1.0).abs() < 0.01,
            "missing velocity should default direction y to 1.0 (Vec2::Y)"
        );
        // Beam extends from (0,0) upward to top=300
        assert!(
            (request.length - 300.0).abs() < 0.01,
            "beam should extend to top boundary, got {}",
            request.length
        );
    }

    #[test]
    fn fire_with_no_transform_and_no_velocity_defaults_both() {
        let mut world = piercing_beam_fire_world();

        let entity = world.spawn_empty().id();

        fire(entity, 1.0, 20.0, &mut world);

        let mut query = world.query::<&PiercingBeamRequest>();
        let results: Vec<_> = query.iter(&world).collect();
        assert_eq!(results.len(), 1, "request should be spawned");

        let request = results[0];
        assert!(
            (request.origin.x).abs() < f32::EPSILON,
            "origin should default to 0.0 x"
        );
        assert!(
            (request.origin.y).abs() < f32::EPSILON,
            "origin should default to 0.0 y"
        );
        assert!(
            (request.direction.y - 1.0).abs() < 0.01,
            "direction should default to Vec2::Y"
        );
    }

    // ── Behavior 21: fire() with zero velocity defaults to Vec2::Y ──

    #[test]
    fn fire_with_zero_velocity_defaults_direction_to_y() {
        let mut world = piercing_beam_fire_world();

        let entity = world
            .spawn((Transform::from_xyz(0.0, 0.0, 0.0), Velocity2D(Vec2::ZERO)))
            .id();

        fire(entity, 1.0, 20.0, &mut world);

        let mut query = world.query::<&PiercingBeamRequest>();
        let results: Vec<_> = query.iter(&world).collect();
        assert_eq!(
            results.len(),
            1,
            "request should be spawned even with zero velocity"
        );

        let request = results[0];
        assert!(
            (request.direction.y - 1.0).abs() < 0.01,
            "zero velocity should default direction to Vec2::Y, got direction ({}, {})",
            request.direction.x,
            request.direction.y
        );
    }

    // ── Behavior 22: fire() with no Transform defaults origin to Vec2::ZERO ──

    #[test]
    fn fire_with_no_transform_defaults_origin_to_zero() {
        let mut world = piercing_beam_fire_world();

        let entity = world.spawn(Velocity2D(Vec2::new(0.0, 400.0))).id();

        fire(entity, 1.0, 20.0, &mut world);

        let mut query = world.query::<&PiercingBeamRequest>();
        let results: Vec<_> = query.iter(&world).collect();
        assert_eq!(results.len(), 1);

        let request = results[0];
        assert!(
            (request.origin.x).abs() < f32::EPSILON,
            "origin x should default to 0.0"
        );
        assert!(
            (request.origin.y).abs() < f32::EPSILON,
            "origin y should default to 0.0"
        );
        assert!(
            (request.direction.y - 1.0).abs() < 0.01,
            "direction should be Vec2::Y"
        );
        // From (0,0) upward to top=300
        assert!(
            (request.length - 300.0).abs() < 0.01,
            "length should be 300.0"
        );
    }

    // ── Behavior 16 extra: request entity has CleanupOnNodeExit ──

    #[test]
    fn fire_request_entity_has_cleanup_on_node_exit() {
        let mut world = piercing_beam_fire_world();

        let entity = world
            .spawn((
                Transform::from_xyz(0.0, 0.0, 0.0),
                Velocity2D(Vec2::new(0.0, 400.0)),
            ))
            .id();

        fire(entity, 1.0, 20.0, &mut world);

        let mut query = world.query_filtered::<Entity, With<PiercingBeamRequest>>();
        let request_entity = query.iter(&world).next().expect("request should exist");

        assert!(
            world.get::<CleanupOnNodeExit>(request_entity).is_some(),
            "PiercingBeamRequest entity should have CleanupOnNodeExit"
        );
    }

    // ── Behavior 23: reverse() is a no-op ──

    #[test]
    fn reverse_is_noop() {
        let mut world = World::new();
        let entity = world.spawn(Transform::from_xyz(10.0, 20.0, 0.0)).id();

        reverse(entity, &mut world);

        assert!(
            world.get_entity(entity).is_ok(),
            "entity should still exist after no-op reverse"
        );
    }

    #[test]
    fn reverse_on_empty_entity_is_noop() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        reverse(entity, &mut world);

        assert!(
            world.get_entity(entity).is_ok(),
            "empty entity should still exist after no-op reverse"
        );
    }

    // ── Behavior 24: process_piercing_beam damages all cells in beam path ──

    #[test]
    fn process_piercing_beam_damages_all_cells_in_beam_and_despawns() {
        let mut app = piercing_beam_damage_test_app();

        let cell_a = spawn_test_cell(&mut app, 0.0, 50.0);
        let cell_b = spawn_test_cell(&mut app, 0.0, 150.0);
        let cell_c = spawn_test_cell(&mut app, 0.0, 250.0);

        let request = app
            .world_mut()
            .spawn(PiercingBeamRequest {
                origin: Vec2::new(0.0, 0.0),
                direction: Vec2::new(0.0, 1.0),
                length: 300.0,
                half_width: 10.0,
                damage: 10.0,
            })
            .id();

        tick(&mut app);

        let collector = app.world().resource::<DamageCellCollector>();
        assert_eq!(
            collector.0.len(),
            3,
            "expected 3 DamageCell messages (one per cell), got {}",
            collector.0.len()
        );

        let damaged_cells: HashSet<Entity> = collector.0.iter().map(|m| m.cell).collect();
        assert!(damaged_cells.contains(&cell_a), "cell_a should be damaged");
        assert!(damaged_cells.contains(&cell_b), "cell_b should be damaged");
        assert!(damaged_cells.contains(&cell_c), "cell_c should be damaged");

        for msg in &collector.0 {
            assert!(
                (msg.damage - 10.0).abs() < f32::EPSILON,
                "each cell should receive damage 10.0"
            );
            assert!(msg.source_chip.is_none(), "source_chip should be None");
        }

        assert!(
            app.world().get_entity(request).is_err(),
            "PiercingBeamRequest entity should be despawned after processing"
        );
    }

    // ── Behavior 25: does not damage cells outside beam width ──

    #[test]
    fn process_piercing_beam_does_not_damage_cell_outside_beam_width() {
        let mut app = piercing_beam_damage_test_app();

        // Cell at (50, 100) — 50 units to the right of beam center
        // Beam half_width=10, so beam extends 10 units left/right
        spawn_test_cell(&mut app, 50.0, 100.0);

        let request = app
            .world_mut()
            .spawn(PiercingBeamRequest {
                origin: Vec2::new(0.0, 0.0),
                direction: Vec2::new(0.0, 1.0),
                length: 300.0,
                half_width: 10.0,
                damage: 10.0,
            })
            .id();

        tick(&mut app);

        let collector = app.world().resource::<DamageCellCollector>();
        assert!(
            collector.0.is_empty(),
            "cell at (50, 100) is outside beam width (10+5 < 50) — no damage"
        );

        assert!(
            app.world().get_entity(request).is_err(),
            "request should be despawned"
        );
    }

    // ── Behavior 26: only targets cells on CELL_LAYER ──

    #[test]
    fn process_piercing_beam_only_targets_cell_layer() {
        let mut app = piercing_beam_damage_test_app();

        // Cell on CELL_LAYER
        let cell = spawn_test_cell(&mut app, 0.0, 50.0);

        // Entity on WALL_LAYER
        let wall_pos = Vec2::new(0.0, 100.0);
        app.world_mut().spawn((
            Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
            CollisionLayers::new(WALL_LAYER, 0),
            Position2D(wall_pos),
            GlobalPosition2D(wall_pos),
            Spatial2D,
        ));

        // Entity on BOLT_LAYER
        let bolt_pos = Vec2::new(0.0, 150.0);
        app.world_mut().spawn((
            Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
            CollisionLayers::new(BOLT_LAYER, 0),
            Position2D(bolt_pos),
            GlobalPosition2D(bolt_pos),
            Spatial2D,
        ));

        app.world_mut().spawn(PiercingBeamRequest {
            origin: Vec2::new(0.0, 0.0),
            direction: Vec2::new(0.0, 1.0),
            length: 300.0,
            half_width: 10.0,
            damage: 10.0,
        });

        tick(&mut app);

        let collector = app.world().resource::<DamageCellCollector>();
        assert_eq!(
            collector.0.len(),
            1,
            "only CELL_LAYER entity should be damaged, got {}",
            collector.0.len()
        );
        assert_eq!(collector.0[0].cell, cell);
    }

    #[test]
    fn process_piercing_beam_targets_entity_with_combined_cell_layer() {
        let mut app = piercing_beam_damage_test_app();

        // Entity with CELL_LAYER | WALL_LAYER
        let pos = Vec2::new(0.0, 50.0);
        let combined = app
            .world_mut()
            .spawn((
                Cell,
                Aabb2D::new(Vec2::ZERO, Vec2::new(5.0, 5.0)),
                CollisionLayers::new(CELL_LAYER | WALL_LAYER, 0),
                Position2D(pos),
                GlobalPosition2D(pos),
                Spatial2D,
            ))
            .id();

        app.world_mut().spawn(PiercingBeamRequest {
            origin: Vec2::new(0.0, 0.0),
            direction: Vec2::new(0.0, 1.0),
            length: 300.0,
            half_width: 10.0,
            damage: 10.0,
        });

        tick(&mut app);

        let collector = app.world().resource::<DamageCellCollector>();
        assert_eq!(
            collector.0.len(),
            1,
            "entity with CELL_LAYER in combined mask should be damaged"
        );
        assert_eq!(collector.0[0].cell, combined);
    }

    // ── Behavior 27: no cells in beam path — despawns without damage ──

    #[test]
    fn process_piercing_beam_no_cells_despawns_without_damage() {
        let mut app = piercing_beam_damage_test_app();

        let request = app
            .world_mut()
            .spawn(PiercingBeamRequest {
                origin: Vec2::new(0.0, 0.0),
                direction: Vec2::new(0.0, 1.0),
                length: 300.0,
                half_width: 10.0,
                damage: 10.0,
            })
            .id();

        tick(&mut app);

        let collector = app.world().resource::<DamageCellCollector>();
        assert!(
            collector.0.is_empty(),
            "no cells — zero DamageCell messages"
        );

        assert!(
            app.world().get_entity(request).is_err(),
            "request should be despawned even with no cells"
        );
    }

    #[test]
    fn process_piercing_beam_cells_outside_beam_rectangle_no_damage() {
        let mut app = piercing_beam_damage_test_app();

        // Cell far to the right — outside beam
        spawn_test_cell(&mut app, 200.0, 100.0);

        let request = app
            .world_mut()
            .spawn(PiercingBeamRequest {
                origin: Vec2::new(0.0, 0.0),
                direction: Vec2::new(0.0, 1.0),
                length: 300.0,
                half_width: 10.0,
                damage: 10.0,
            })
            .id();

        tick(&mut app);

        let collector = app.world().resource::<DamageCellCollector>();
        assert!(
            collector.0.is_empty(),
            "cell outside beam rectangle should not be damaged"
        );

        assert!(
            app.world().get_entity(request).is_err(),
            "request should be despawned"
        );
    }

    // ── Behavior 28: each cell damaged at most once per beam ──

    #[test]
    fn process_piercing_beam_damages_each_cell_at_most_once() {
        let mut app = piercing_beam_damage_test_app();

        // One cell with large AABB that overlaps beam
        let pos = Vec2::new(0.0, 50.0);
        let cell = app
            .world_mut()
            .spawn((
                Cell,
                Aabb2D::new(Vec2::ZERO, Vec2::new(20.0, 20.0)),
                CollisionLayers::new(CELL_LAYER, 0),
                Position2D(pos),
                GlobalPosition2D(pos),
                Spatial2D,
            ))
            .id();

        app.world_mut().spawn(PiercingBeamRequest {
            origin: Vec2::new(0.0, 0.0),
            direction: Vec2::new(0.0, 1.0),
            length: 300.0,
            half_width: 20.0,
            damage: 10.0,
        });

        tick(&mut app);

        let collector = app.world().resource::<DamageCellCollector>();
        assert_eq!(
            collector.0.len(),
            1,
            "cell should be damaged exactly once, got {}",
            collector.0.len()
        );
        assert_eq!(collector.0[0].cell, cell);
    }

    // ── Behavior 29: Multiple requests processed independently ──

    #[test]
    fn multiple_piercing_beam_requests_processed_independently() {
        let mut app = piercing_beam_damage_test_app();

        // One cell in both beams' paths
        let cell = spawn_test_cell(&mut app, 0.0, 50.0);

        let req1 = app
            .world_mut()
            .spawn(PiercingBeamRequest {
                origin: Vec2::new(0.0, 0.0),
                direction: Vec2::new(0.0, 1.0),
                length: 300.0,
                half_width: 10.0,
                damage: 10.0,
            })
            .id();

        let req2 = app
            .world_mut()
            .spawn(PiercingBeamRequest {
                origin: Vec2::new(0.0, 0.0),
                direction: Vec2::new(0.0, 1.0),
                length: 300.0,
                half_width: 10.0,
                damage: 20.0,
            })
            .id();

        tick(&mut app);

        let collector = app.world().resource::<DamageCellCollector>();
        assert_eq!(
            collector.0.len(),
            2,
            "two beams hitting same cell should produce 2 DamageCell messages, got {}",
            collector.0.len()
        );

        for msg in &collector.0 {
            assert_eq!(msg.cell, cell, "both messages should target the same cell");
        }

        let mut damages: Vec<f32> = collector.0.iter().map(|m| m.damage).collect();
        damages.sort_by(|a, b| a.partial_cmp(b).unwrap());
        assert!(
            (damages[0] - 10.0).abs() < f32::EPSILON,
            "expected damage 10.0"
        );
        assert!(
            (damages[1] - 20.0).abs() < f32::EPSILON,
            "expected damage 20.0"
        );

        assert!(
            app.world().get_entity(req1).is_err(),
            "first request should be despawned"
        );
        assert!(
            app.world().get_entity(req2).is_err(),
            "second request should be despawned"
        );
    }

    // ── Behavior 30: diagonal beam ──

    #[test]
    fn process_piercing_beam_handles_diagonal_beam() {
        let mut app = piercing_beam_damage_test_app();

        // Cell on diagonal path at (100, 100)
        let cell_on_path = spawn_test_cell(&mut app, 100.0, 100.0);
        // Cell off diagonal path at (100, 0)
        let cell_off_path = spawn_test_cell(&mut app, 100.0, 0.0);

        let dir = Vec2::new(1.0, 1.0).normalize();
        app.world_mut().spawn(PiercingBeamRequest {
            origin: Vec2::new(0.0, 0.0),
            direction: dir,
            length: 400.0,
            half_width: 15.0,
            damage: 10.0,
        });

        tick(&mut app);

        let collector = app.world().resource::<DamageCellCollector>();
        // Cell at (100, 100) is on the 45-degree diagonal path and should be hit
        let cells_hit: HashSet<Entity> = collector.0.iter().map(|m| m.cell).collect();
        assert!(
            cells_hit.contains(&cell_on_path),
            "cell at (100, 100) should be hit by diagonal beam"
        );
        assert!(
            !cells_hit.contains(&cell_off_path),
            "cell off diagonal path should not be damaged"
        );
    }

    // ── Behavior 31: register() wires the process system ──

    #[test]
    fn register_wires_process_piercing_beam_system() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzPhysics2dPlugin);
        app.add_message::<DamageCell>();
        app.insert_resource(DamageCellCollector::default());
        app.insert_resource(PlayfieldConfig::default());
        app.add_systems(Update, collect_damage_cells);

        register(&mut app);

        // Spawn a request — if register() wires the system, it should be processed
        let request = app
            .world_mut()
            .spawn(PiercingBeamRequest {
                origin: Vec2::ZERO,
                direction: Vec2::Y,
                length: 300.0,
                half_width: 10.0,
                damage: 10.0,
            })
            .id();

        tick(&mut app);

        // The request should be despawned after processing
        assert!(
            app.world().get_entity(request).is_err(),
            "register() should wire process_piercing_beam — request should be despawned after tick"
        );
    }
}
