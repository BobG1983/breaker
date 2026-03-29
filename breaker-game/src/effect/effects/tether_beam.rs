//! Two free-moving bolts connected by a crackling neon beam that damages intersected cells.

use std::collections::HashSet;

use bevy::prelude::*;
use rand::Rng;
use rantzsoft_physics2d::{
    aabb::Aabb2D, ccd::ray_vs_aabb, collision_layers::CollisionLayers, plugin::PhysicsSystems,
    resources::CollisionQuadtree,
};
use rantzsoft_spatial2d::components::{
    GlobalPosition2D, Position2D, PreviousPosition, PreviousScale, Scale2D, Velocity2D,
};

use crate::{
    bolt::{
        BASE_BOLT_DAMAGE,
        components::{Bolt, BoltBaseSpeed, BoltMaxSpeed, BoltMinSpeed, BoltRadius, ExtraBolt},
        resources::BoltConfig,
    },
    cells::{components::Cell, messages::DamageCell},
    shared::{
        BOLT_LAYER, BREAKER_LAYER, CELL_LAYER, CleanupOnNodeExit, GameDrawLayer, WALL_LAYER,
        playing_state::PlayingState, rng::GameRng,
    },
};

/// Marker on a tether bolt entity, pointing to its beam entity.
#[derive(Component)]
pub struct TetherBoltMarker(pub Entity);

/// The beam entity linking two tether bolts.
#[derive(Component)]
pub struct TetherBeamComponent {
    /// First tether bolt entity.
    pub bolt_a: Entity,
    /// Second tether bolt entity.
    pub bolt_b: Entity,
    /// Damage multiplier applied to `BASE_BOLT_DAMAGE`.
    pub damage_mult: f32,
}

/// Spawns two tethered bolts with a damaging beam between them.
///
/// Evolution of `ChainBolt`. The beam is a line segment between the two bolt
/// positions — cells intersecting the beam take damage each tick.
pub(crate) fn fire(entity: Entity, damage_mult: f32, world: &mut World) {
    let config = world.resource::<BoltConfig>();
    let radius = config.radius;
    let base_speed = config.base_speed;
    let min_speed = config.min_speed;
    let max_speed = config.max_speed;

    let spawn_pos = world.get::<Position2D>(entity).map_or(Vec2::ZERO, |p| p.0);

    // Generate two independent random angles for the two bolts
    let (angle_a, angle_b) = {
        let mut rng = world.resource_mut::<GameRng>();
        (
            rng.0.random_range(0.0..std::f32::consts::TAU),
            rng.0.random_range(0.0..std::f32::consts::TAU),
        )
    };

    let velocity_a = Vec2::new(angle_a.cos(), angle_a.sin()) * base_speed;
    let velocity_b = Vec2::new(angle_b.cos(), angle_b.sin()) * base_speed;

    // Spawn bolt A
    let bolt_a = world
        .spawn((
            (
                Bolt,
                ExtraBolt,
                Position2D(spawn_pos),
                PreviousPosition(spawn_pos),
                Scale2D {
                    x: radius,
                    y: radius,
                },
                PreviousScale {
                    x: radius,
                    y: radius,
                },
                Aabb2D::new(Vec2::ZERO, Vec2::new(radius, radius)),
            ),
            (
                CollisionLayers::new(BOLT_LAYER, CELL_LAYER | WALL_LAYER | BREAKER_LAYER),
                Velocity2D(velocity_a),
                BoltBaseSpeed(base_speed),
                BoltMinSpeed(min_speed),
                BoltMaxSpeed(max_speed),
                BoltRadius(radius),
                CleanupOnNodeExit,
                GameDrawLayer::Bolt,
            ),
        ))
        .id();

    // Spawn bolt B
    let bolt_b = world
        .spawn((
            (
                Bolt,
                ExtraBolt,
                Position2D(spawn_pos),
                PreviousPosition(spawn_pos),
                Scale2D {
                    x: radius,
                    y: radius,
                },
                PreviousScale {
                    x: radius,
                    y: radius,
                },
                Aabb2D::new(Vec2::ZERO, Vec2::new(radius, radius)),
            ),
            (
                CollisionLayers::new(BOLT_LAYER, CELL_LAYER | WALL_LAYER | BREAKER_LAYER),
                Velocity2D(velocity_b),
                BoltBaseSpeed(base_speed),
                BoltMinSpeed(min_speed),
                BoltMaxSpeed(max_speed),
                BoltRadius(radius),
                CleanupOnNodeExit,
                GameDrawLayer::Bolt,
            ),
        ))
        .id();

    // Spawn the beam entity linking both bolts
    let beam = world
        .spawn((
            TetherBeamComponent {
                bolt_a,
                bolt_b,
                damage_mult,
            },
            CleanupOnNodeExit,
        ))
        .id();

    // Add TetherBoltMarker to each bolt, pointing to the beam
    world.entity_mut(bolt_a).insert(TetherBoltMarker(beam));
    world.entity_mut(bolt_b).insert(TetherBoltMarker(beam));
}

/// No-op — tether bolts have their own lifecycle.
pub(crate) fn reverse(_entity: Entity, _damage_mult: f32, _world: &mut World) {}

/// Tick system: damages cells whose AABB intersects each tether beam segment.
///
/// For each beam, looks up the positions of `bolt_a` and `bolt_b`. If either bolt
/// is missing, despawns the beam. Otherwise, computes broadphase via quadtree
/// AABB query and narrowphase via ray-vs-AABB intersection, sending `DamageCell`
/// for each cell hit by the beam segment.
///
/// The beam has an effective half-width equal to the bolt radius (from
/// `BoltConfig`), so cells whose AABBs are within the bolt radius of the beam
/// line segment are considered intersecting.
pub fn tick_tether_beam(
    mut commands: Commands,
    beams: Query<(Entity, &TetherBeamComponent)>,
    bolt_positions: Query<&Position2D, With<Bolt>>,
    quadtree: Res<CollisionQuadtree>,
    cell_aabbs: Query<(&Aabb2D, &GlobalPosition2D), With<Cell>>,
    bolt_config: Option<Res<BoltConfig>>,
    mut damage_writer: MessageWriter<DamageCell>,
) {
    let query_layers = CollisionLayers::new(0, CELL_LAYER);
    let beam_half_width = bolt_config
        .as_ref()
        .map_or(BoltConfig::default().radius, |c| c.radius);

    for (beam_entity, component) in &beams {
        // Look up both bolt positions; despawn beam if either is missing
        let pos_a = if let Ok(p) = bolt_positions.get(component.bolt_a) {
            p.0
        } else {
            commands.entity(beam_entity).despawn();
            continue;
        };
        let pos_b = if let Ok(p) = bolt_positions.get(component.bolt_b) {
            p.0
        } else {
            commands.entity(beam_entity).despawn();
            continue;
        };

        // Broadphase: compute beam bounding box expanded by beam half-width and
        // query quadtree. The expansion ensures cells near (but not exactly on)
        // the beam line are included as candidates.
        let beam_aabb =
            Aabb2D::from_min_max(pos_a.min(pos_b), pos_a.max(pos_b)).expand_by(beam_half_width);
        let candidates = quadtree
            .quadtree
            .query_aabb_filtered(&beam_aabb, query_layers);

        // Narrowphase: test line-segment vs cell AABB intersection
        let beam_vec = pos_b - pos_a;
        let max_dist = beam_vec.length();
        let direction = beam_vec.normalize_or_zero();
        let damage = BASE_BOLT_DAMAGE * component.damage_mult;

        let mut damaged_this_tick: HashSet<Entity> = HashSet::new();

        for cell in candidates {
            if damaged_this_tick.contains(&cell) {
                continue;
            }

            let Ok((local_aabb, global_pos)) = cell_aabbs.get(cell) else {
                continue;
            };

            // Compute world-space AABB for the cell, expanded by the beam
            // half-width (Minkowski sum) so a point-ray test is equivalent to a
            // thick-beam-vs-AABB test.
            let world_aabb = Aabb2D::new(global_pos.0 + local_aabb.center, local_aabb.half_extents)
                .expand_by(beam_half_width);

            // Check ray intersection OR origin-inside-AABB
            let ray_hit = ray_vs_aabb(pos_a, direction, max_dist, &world_aabb);
            let origin_inside = world_aabb.contains_point(pos_a);

            if ray_hit.is_some() || origin_inside {
                damaged_this_tick.insert(cell);
                damage_writer.write(DamageCell {
                    cell,
                    damage,
                    source_chip: None,
                });
            }
        }
    }
}

/// Registers systems for `TetherBeam` effect.
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        tick_tether_beam
            .after(PhysicsSystems::MaintainQuadtree)
            .run_if(in_state(PlayingState::Active)),
    );
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use rantzsoft_physics2d::{
        aabb::Aabb2D, collision_layers::CollisionLayers, plugin::RantzPhysics2dPlugin,
    };
    use rantzsoft_spatial2d::components::{
        GlobalPosition2D, Position2D, Scale2D, Spatial2D, Velocity2D,
    };

    use super::*;
    use crate::{
        bolt::{
            BASE_BOLT_DAMAGE,
            components::{Bolt, BoltBaseSpeed, BoltMaxSpeed, BoltMinSpeed, BoltRadius, ExtraBolt},
            resources::BoltConfig,
        },
        cells::{components::Cell, messages::DamageCell},
        shared::{
            BOLT_LAYER, BREAKER_LAYER, CELL_LAYER, CleanupOnNodeExit, CleanupOnRunEnd,
            GameDrawLayer, WALL_LAYER, rng::GameRng,
        },
    };

    fn world_with_bolt_config() -> World {
        let mut world = World::new();
        world.insert_resource(BoltConfig::default());
        world.insert_resource(GameRng::default());
        world
    }

    // ── fire() tests ──────────────────────────────────────────────

    #[test]
    fn fire_spawns_two_tether_bolts_with_full_physics_components() {
        let mut world = world_with_bolt_config();
        let entity = world.spawn(Position2D(Vec2::new(100.0, 200.0))).id();

        fire(entity, 1.5, &mut world);

        let mut query = world.query_filtered::<Entity, With<TetherBoltMarker>>();
        let bolts: Vec<Entity> = query.iter(&world).collect();
        assert_eq!(
            bolts.len(),
            2,
            "fire should spawn exactly 2 tether bolts, got {}",
            bolts.len()
        );

        for bolt in &bolts {
            // Bolt marker
            assert!(
                world.get::<Bolt>(*bolt).is_some(),
                "tether bolt should have Bolt"
            );

            // ExtraBolt
            assert!(
                world.get::<ExtraBolt>(*bolt).is_some(),
                "tether bolt should have ExtraBolt"
            );

            // Position2D from owner
            let pos = world
                .get::<Position2D>(*bolt)
                .expect("tether bolt should have Position2D");
            assert_eq!(pos.0, Vec2::new(100.0, 200.0));

            // Velocity2D — magnitude at base_speed
            let vel = world
                .get::<Velocity2D>(*bolt)
                .expect("tether bolt should have Velocity2D");
            assert!(
                (vel.0.length() - 400.0).abs() < 1.0,
                "tether bolt velocity magnitude should be base_speed (400.0), got {}",
                vel.0.length()
            );

            // Scale2D
            let scale = world
                .get::<Scale2D>(*bolt)
                .expect("tether bolt should have Scale2D");
            assert!((scale.x - 8.0).abs() < f32::EPSILON);
            assert!((scale.y - 8.0).abs() < f32::EPSILON);

            // Aabb2D
            let aabb = world
                .get::<Aabb2D>(*bolt)
                .expect("tether bolt should have Aabb2D");
            assert_eq!(aabb.center, Vec2::ZERO);
            assert_eq!(aabb.half_extents, Vec2::new(8.0, 8.0));

            // CollisionLayers
            let layers = world
                .get::<CollisionLayers>(*bolt)
                .expect("tether bolt should have CollisionLayers");
            assert_eq!(layers.membership, BOLT_LAYER);
            assert_eq!(layers.mask, CELL_LAYER | WALL_LAYER | BREAKER_LAYER);

            // Speed components
            assert!((world.get::<BoltBaseSpeed>(*bolt).unwrap().0 - 400.0).abs() < f32::EPSILON);
            assert!((world.get::<BoltMinSpeed>(*bolt).unwrap().0 - 200.0).abs() < f32::EPSILON);
            assert!((world.get::<BoltMaxSpeed>(*bolt).unwrap().0 - 800.0).abs() < f32::EPSILON);
            assert!((world.get::<BoltRadius>(*bolt).unwrap().0 - 8.0).abs() < f32::EPSILON);

            // CleanupOnNodeExit
            assert!(world.get::<CleanupOnNodeExit>(*bolt).is_some());

            // GameDrawLayer::Bolt
            assert!(world.get::<GameDrawLayer>(*bolt).is_some());
        }
    }

    #[test]
    fn fire_spawns_tether_bolt_marker_storing_beam_entity() {
        let mut world = world_with_bolt_config();
        let entity = world.spawn(Position2D(Vec2::ZERO)).id();

        fire(entity, 1.5, &mut world);

        // Find beam entity
        let mut beam_query = world.query_filtered::<Entity, With<TetherBeamComponent>>();
        let beam_entities: Vec<Entity> = beam_query.iter(&world).collect();
        assert_eq!(beam_entities.len(), 1, "should spawn exactly 1 beam entity");
        let beam_entity = beam_entities[0];

        // Both tether bolts should store this beam entity
        let mut bolt_query = world.query::<&TetherBoltMarker>();
        for marker in bolt_query.iter(&world) {
            assert_eq!(
                marker.0, beam_entity,
                "TetherBoltMarker should store the beam entity"
            );
        }
    }

    #[test]
    fn fire_spawns_two_bolts_with_different_velocity_directions() {
        let mut world = world_with_bolt_config();
        let entity = world.spawn(Position2D(Vec2::ZERO)).id();

        fire(entity, 1.5, &mut world);

        let mut query = world.query::<(&TetherBoltMarker, &Velocity2D)>();
        let velocities: Vec<Vec2> = query.iter(&world).map(|(_, v)| v.0).collect();
        assert_eq!(velocities.len(), 2);

        for vel in &velocities {
            assert!(
                (vel.length() - 400.0).abs() < 1.0,
                "each tether bolt velocity should be ~400.0, got {}",
                vel.length()
            );
        }

        // Probabilistically different directions (each gets independent random angle)
        let dir_a = velocities[0].normalize();
        let dir_b = velocities[1].normalize();
        // With independent random angles, they should differ
        assert!(
            (dir_a - dir_b).length() > 0.001,
            "two tether bolts should have different velocity directions"
        );
    }

    #[test]
    fn fire_does_not_spawn_distance_constraint() {
        let mut world = world_with_bolt_config();
        let entity = world.spawn(Position2D(Vec2::ZERO)).id();

        fire(entity, 1.5, &mut world);

        // Gate: fire() must actually spawn tether bolts for this negative assertion to be meaningful
        let mut bolt_query = world.query_filtered::<Entity, With<TetherBoltMarker>>();
        let tether_bolt_count = bolt_query.iter(&world).count();
        assert!(
            tether_bolt_count >= 1,
            "gate: fire() must spawn tether bolts for DistanceConstraint check to be meaningful, got {tether_bolt_count}"
        );

        // No DistanceConstraint should exist — unlike ChainBolt
        let mut query = world.query::<&rantzsoft_physics2d::constraint::DistanceConstraint>();
        let count = query.iter(&world).count();
        assert_eq!(
            count, 0,
            "TetherBeam should NOT spawn DistanceConstraint, got {count}"
        );
    }

    #[test]
    fn fire_spawns_tether_beam_component_linking_both_bolts() {
        let mut world = world_with_bolt_config();
        let entity = world.spawn(Position2D(Vec2::new(50.0, 50.0))).id();

        fire(entity, 1.5, &mut world);

        let mut beam_query = world.query::<&TetherBeamComponent>();
        let beams: Vec<&TetherBeamComponent> = beam_query.iter(&world).collect();
        assert_eq!(beams.len(), 1, "should spawn exactly 1 TetherBeamComponent");

        let beam = beams[0];
        assert!(
            (beam.damage_mult - 1.5).abs() < f32::EPSILON,
            "damage_mult should be 1.5, got {}",
            beam.damage_mult
        );

        // Copy beam fields into owned locals so the immutable borrow on world is dropped
        let beam_bolt_a = beam.bolt_a;
        let beam_bolt_b = beam.bolt_b;
        drop(beams);

        // bolt_a and bolt_b should reference the tether bolt entities
        let mut bolt_query = world.query_filtered::<Entity, With<TetherBoltMarker>>();
        let bolt_entities: HashSet<Entity> = bolt_query.iter(&world).collect();
        assert!(
            bolt_entities.contains(&beam_bolt_a),
            "beam.bolt_a should reference a tether bolt entity"
        );
        assert!(
            bolt_entities.contains(&beam_bolt_b),
            "beam.bolt_b should reference a tether bolt entity"
        );
        assert_ne!(
            beam_bolt_a, beam_bolt_b,
            "bolt_a and bolt_b should be different entities"
        );
    }

    #[test]
    fn fire_with_zero_damage_mult_spawns_beam() {
        let mut world = world_with_bolt_config();
        let entity = world.spawn(Position2D(Vec2::ZERO)).id();

        fire(entity, 0.0, &mut world);

        let mut beam_query = world.query::<&TetherBeamComponent>();
        let beam = beam_query.iter(&world).next().expect("beam should exist");
        assert!(
            (beam.damage_mult - 0.0).abs() < f32::EPSILON,
            "damage_mult=0.0 should be stored, got {}",
            beam.damage_mult
        );
    }

    #[test]
    fn fire_spawns_bolts_with_extra_bolt_and_cleanup_on_node_exit() {
        let mut world = world_with_bolt_config();
        let entity = world.spawn(Position2D(Vec2::ZERO)).id();

        fire(entity, 1.5, &mut world);

        let mut query = world.query_filtered::<Entity, With<TetherBoltMarker>>();
        for bolt in query.iter(&world) {
            assert!(
                world.get::<ExtraBolt>(bolt).is_some(),
                "tether bolt should have ExtraBolt"
            );
            assert!(
                world.get::<CleanupOnNodeExit>(bolt).is_some(),
                "tether bolt should have CleanupOnNodeExit"
            );
            assert!(
                world.get::<CleanupOnRunEnd>(bolt).is_none(),
                "tether bolt should NOT have CleanupOnRunEnd"
            );
        }
    }

    #[test]
    fn fire_reads_position_from_position2d_not_transform() {
        let mut world = world_with_bolt_config();
        let entity = world
            .spawn((
                Position2D(Vec2::new(30.0, 40.0)),
                Transform::from_xyz(999.0, 999.0, 0.0),
            ))
            .id();

        fire(entity, 1.5, &mut world);

        let mut query = world.query::<(&TetherBoltMarker, &Position2D)>();
        for (_marker, pos) in query.iter(&world) {
            assert_eq!(
                pos.0,
                Vec2::new(30.0, 40.0),
                "tether bolt should use Position2D (30, 40), not Transform (999, 999)"
            );
        }
    }

    #[test]
    fn fire_spawns_bolts_at_zero_when_owner_has_no_position2d() {
        let mut world = world_with_bolt_config();
        let entity = world.spawn_empty().id();

        fire(entity, 1.5, &mut world);

        // Gate: fire() must actually spawn tether bolts for position check to be meaningful
        let mut count_query = world.query_filtered::<Entity, With<TetherBoltMarker>>();
        let tether_bolt_count = count_query.iter(&world).count();
        assert!(
            tether_bolt_count >= 2,
            "expected tether bolts to be spawned, got {tether_bolt_count}"
        );

        let mut query = world.query::<(&TetherBoltMarker, &Position2D)>();
        for (_marker, pos) in query.iter(&world) {
            assert_eq!(
                pos.0,
                Vec2::ZERO,
                "tether bolt should default to Vec2::ZERO when owner has no Position2D"
            );
        }
    }

    // ── reverse() — no-op ──────────────────────────────────────────

    #[test]
    fn reverse_does_not_despawn_tether_entities() {
        let mut world = world_with_bolt_config();
        let entity = world.spawn(Position2D(Vec2::ZERO)).id();

        fire(entity, 1.5, &mut world);

        let mut bolt_query = world.query_filtered::<Entity, With<TetherBoltMarker>>();
        let bolt_count_before = bolt_query.iter(&world).count();
        let mut beam_query = world.query_filtered::<Entity, With<TetherBeamComponent>>();
        let beam_count_before = beam_query.iter(&world).count();

        reverse(entity, 1.5, &mut world);

        let bolt_count_after = bolt_query.iter(&world).count();
        let beam_count_after = beam_query.iter(&world).count();
        assert_eq!(
            bolt_count_before, bolt_count_after,
            "reverse should not despawn tether bolts"
        );
        assert_eq!(
            beam_count_before, beam_count_after,
            "reverse should not despawn beam"
        );
    }

    #[test]
    fn reverse_with_no_tether_entities_does_not_panic() {
        let mut world = world_with_bolt_config();
        let entity = world.spawn(Position2D(Vec2::ZERO)).id();

        // Should not panic
        reverse(entity, 1.5, &mut world);
    }

    // ── tick_tether_beam system tests ───────────────────────────────

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

    fn damage_test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(RantzPhysics2dPlugin);
        app.add_message::<DamageCell>();
        app.insert_resource(DamageCellCollector::default());
        app.add_systems(Update, tick_tether_beam);
        app.add_systems(Update, collect_damage_cells.after(tick_tether_beam));
        app
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
        spawn_test_cell_with_extents(app, x, y, Vec2::new(10.0, 10.0))
    }

    fn spawn_test_cell_with_extents(app: &mut App, x: f32, y: f32, half_extents: Vec2) -> Entity {
        let pos = Vec2::new(x, y);
        app.world_mut()
            .spawn((
                Cell,
                Aabb2D::new(Vec2::ZERO, half_extents),
                CollisionLayers::new(CELL_LAYER, 0),
                Position2D(pos),
                GlobalPosition2D(pos),
                Spatial2D,
            ))
            .id()
    }

    /// Spawn a tether beam with two bolt entities at given positions.
    fn spawn_tether_beam(
        app: &mut App,
        pos_a: Vec2,
        pos_b: Vec2,
        damage_mult: f32,
    ) -> (Entity, Entity, Entity) {
        let bolt_a = app
            .world_mut()
            .spawn((Bolt, Position2D(pos_a), GlobalPosition2D(pos_a), Spatial2D))
            .id();
        let bolt_b = app
            .world_mut()
            .spawn((Bolt, Position2D(pos_b), GlobalPosition2D(pos_b), Spatial2D))
            .id();
        let beam = app
            .world_mut()
            .spawn((
                TetherBeamComponent {
                    bolt_a,
                    bolt_b,
                    damage_mult,
                },
                CleanupOnNodeExit,
            ))
            .id();
        // Add TetherBoltMarker to each bolt
        app.world_mut()
            .entity_mut(bolt_a)
            .insert(TetherBoltMarker(beam));
        app.world_mut()
            .entity_mut(bolt_b)
            .insert(TetherBoltMarker(beam));
        (bolt_a, bolt_b, beam)
    }

    #[test]
    fn tick_tether_beam_damages_cell_intersecting_beam_segment() {
        let mut app = damage_test_app();

        let (_bolt_a, _bolt_b, _beam) =
            spawn_tether_beam(&mut app, Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), 2.0);
        let cell = spawn_test_cell(&mut app, 50.0, 0.0);

        tick(&mut app);

        let collector = app.world().resource::<DamageCellCollector>();
        assert_eq!(
            collector.0.len(),
            1,
            "expected 1 DamageCell message, got {}",
            collector.0.len()
        );
        assert_eq!(collector.0[0].cell, cell);
        let expected_damage = BASE_BOLT_DAMAGE * 2.0;
        assert!(
            (collector.0[0].damage - expected_damage).abs() < f32::EPSILON,
            "expected damage {expected_damage}, got {}",
            collector.0[0].damage
        );
    }

    #[test]
    fn tick_tether_beam_does_not_damage_cell_not_intersecting() {
        let mut app = damage_test_app();

        // Beam along y=0 from (0,0) to (100,0)
        spawn_tether_beam(&mut app, Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), 1.0);
        // Cell at (50, 50) with small AABB — does NOT intersect the beam segment at y=0
        spawn_test_cell_with_extents(&mut app, 50.0, 50.0, Vec2::new(5.0, 5.0));

        tick(&mut app);

        let collector = app.world().resource::<DamageCellCollector>();
        assert!(
            collector.0.is_empty(),
            "cell at (50, 50) should not be hit by horizontal beam at y=0"
        );
    }

    #[test]
    fn tick_tether_beam_uses_line_segment_vs_aabb_not_circle() {
        let mut app = damage_test_app();

        // Beam from (0,0) to (100,0) along y=0 with damage_mult=1.0
        spawn_tether_beam(&mut app, Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), 1.0);

        // Cell at (50, 30) with half_extents (5,5) — AABB spans y=[25,35], does NOT intersect y=0
        spawn_test_cell_with_extents(&mut app, 50.0, 30.0, Vec2::new(5.0, 5.0));

        tick(&mut app);

        let collector = app.world().resource::<DamageCellCollector>();
        assert!(
            collector.0.is_empty(),
            "cell near beam but AABB not intersecting should receive no damage"
        );
    }

    #[test]
    fn tick_tether_beam_cell_aabb_barely_intersects_beam() {
        let mut app = damage_test_app();

        // Beam from (0,0) to (100,0) along y=0
        spawn_tether_beam(&mut app, Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), 1.0);

        // Cell at (50, 6) with half_extents (5,5) — AABB spans y=[1,11], DOES intersect y=0
        let cell = spawn_test_cell_with_extents(&mut app, 50.0, 6.0, Vec2::new(5.0, 5.0));

        tick(&mut app);

        let collector = app.world().resource::<DamageCellCollector>();
        assert_eq!(
            collector.0.len(),
            1,
            "cell at (50, 6) with half_extents (5,5) should intersect beam at y=0"
        );
        assert_eq!(collector.0[0].cell, cell);
    }

    #[test]
    fn tick_tether_beam_damages_multiple_cells_along_beam() {
        let mut app = damage_test_app();

        spawn_tether_beam(&mut app, Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), 1.0);

        let cell_a = spawn_test_cell(&mut app, 20.0, 0.0);
        let cell_b = spawn_test_cell(&mut app, 60.0, 0.0);
        let cell_c = spawn_test_cell(&mut app, 90.0, 0.0);

        tick(&mut app);

        let collector = app.world().resource::<DamageCellCollector>();
        assert_eq!(
            collector.0.len(),
            3,
            "expected 3 DamageCell messages, got {}",
            collector.0.len()
        );

        let damaged_cells: HashSet<Entity> = collector.0.iter().map(|m| m.cell).collect();
        assert!(damaged_cells.contains(&cell_a), "cell A should be damaged");
        assert!(damaged_cells.contains(&cell_b), "cell B should be damaged");
        assert!(damaged_cells.contains(&cell_c), "cell C should be damaged");

        for msg in &collector.0 {
            assert!(
                (msg.damage - BASE_BOLT_DAMAGE).abs() < f32::EPSILON,
                "each cell damage should be BASE_BOLT_DAMAGE * 1.0 = 10.0, got {}",
                msg.damage
            );
        }
    }

    #[test]
    fn tick_tether_beam_dedup_damages_cell_at_most_once_per_tick() {
        let mut app = damage_test_app();

        spawn_tether_beam(&mut app, Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), 1.0);
        let cell = spawn_test_cell(&mut app, 50.0, 0.0);

        tick(&mut app);

        let collector = app.world().resource::<DamageCellCollector>();
        assert_eq!(
            collector.0.len(),
            1,
            "cell should be damaged exactly once per tick (dedup), got {}",
            collector.0.len()
        );
        assert_eq!(collector.0[0].cell, cell);
    }

    #[test]
    fn tick_tether_beam_skips_entities_outside_cell_layer() {
        let mut app = damage_test_app();

        spawn_tether_beam(&mut app, Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), 1.0);

        // Spawn a bolt-layer entity at (50, 0) — should NOT be damaged
        let pos = Vec2::new(50.0, 0.0);
        app.world_mut().spawn((
            Aabb2D::new(Vec2::ZERO, Vec2::new(10.0, 10.0)),
            CollisionLayers::new(BOLT_LAYER, 0),
            Position2D(pos),
            GlobalPosition2D(pos),
            Spatial2D,
        ));

        tick(&mut app);

        let collector = app.world().resource::<DamageCellCollector>();
        assert!(
            collector.0.is_empty(),
            "non-CELL_LAYER entities should not receive DamageCell"
        );
    }

    #[test]
    fn tick_tether_beam_damages_entity_with_cell_layer_in_combined_layers() {
        let mut app = damage_test_app();

        spawn_tether_beam(&mut app, Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), 1.0);

        // Entity with CELL_LAYER | BOLT_LAYER — IS on CELL_LAYER, so should be damaged
        let pos = Vec2::new(50.0, 0.0);
        let cell = app
            .world_mut()
            .spawn((
                Cell,
                Aabb2D::new(Vec2::ZERO, Vec2::new(10.0, 10.0)),
                CollisionLayers::new(CELL_LAYER | BOLT_LAYER, 0),
                Position2D(pos),
                GlobalPosition2D(pos),
                Spatial2D,
            ))
            .id();

        tick(&mut app);

        let collector = app.world().resource::<DamageCellCollector>();
        assert_eq!(
            collector.0.len(),
            1,
            "entity with CELL_LAYER in combined mask should be damaged"
        );
        assert_eq!(collector.0[0].cell, cell);
    }

    #[test]
    fn tick_tether_beam_damages_every_tick_no_cooldown() {
        let mut app = damage_test_app();

        spawn_tether_beam(&mut app, Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), 1.0);
        spawn_test_cell(&mut app, 50.0, 0.0);

        // First tick
        tick(&mut app);

        // Second tick
        tick(&mut app);

        let collector = app.world().resource::<DamageCellCollector>();
        assert_eq!(
            collector.0.len(),
            2,
            "beam should damage cell on each tick (no cooldown), got {} messages",
            collector.0.len()
        );
    }

    #[test]
    fn tick_tether_beam_zero_length_segment_damages_cell_containing_point() {
        let mut app = damage_test_app();

        // Both bolts at same position — zero-length beam
        spawn_tether_beam(&mut app, Vec2::new(50.0, 50.0), Vec2::new(50.0, 50.0), 1.0);

        // Cell at (50, 50) with AABB half_extents (10, 10) — contains the point
        let cell = spawn_test_cell(&mut app, 50.0, 50.0);

        tick(&mut app);

        let collector = app.world().resource::<DamageCellCollector>();
        assert_eq!(
            collector.0.len(),
            1,
            "zero-length beam at cell position should damage the cell"
        );
        assert_eq!(collector.0[0].cell, cell);
    }

    #[test]
    fn tick_tether_beam_zero_length_segment_does_not_damage_distant_cell() {
        let mut app = damage_test_app();

        // Both bolts at same position (50, 50) — zero-length beam
        spawn_tether_beam(&mut app, Vec2::new(50.0, 50.0), Vec2::new(50.0, 50.0), 1.0);

        // Cell at (100, 100) with small AABB (5, 5) — does not contain point (50, 50)
        spawn_test_cell_with_extents(&mut app, 100.0, 100.0, Vec2::new(5.0, 5.0));

        tick(&mut app);

        let collector = app.world().resource::<DamageCellCollector>();
        assert!(
            collector.0.is_empty(),
            "zero-length beam at (50,50) should not damage cell at (100,100)"
        );
    }

    #[test]
    fn tick_tether_beam_despawns_beam_when_bolt_a_despawned() {
        let mut app = damage_test_app();

        let (bolt_a, _bolt_b, beam) =
            spawn_tether_beam(&mut app, Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), 1.0);

        // Despawn bolt_a
        app.world_mut().despawn(bolt_a);

        tick(&mut app);

        assert!(
            app.world().get_entity(beam).is_err(),
            "beam entity should be despawned when bolt_a is gone"
        );
    }

    #[test]
    fn tick_tether_beam_despawns_beam_when_bolt_b_despawned() {
        let mut app = damage_test_app();

        let (_bolt_a, bolt_b, beam) =
            spawn_tether_beam(&mut app, Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), 1.0);

        // Despawn bolt_b
        app.world_mut().despawn(bolt_b);

        tick(&mut app);

        assert!(
            app.world().get_entity(beam).is_err(),
            "beam entity should be despawned when bolt_b is gone"
        );
    }

    #[test]
    fn tick_tether_beam_despawns_beam_when_both_bolts_despawned() {
        let mut app = damage_test_app();

        let (bolt_a, bolt_b, beam) =
            spawn_tether_beam(&mut app, Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), 1.0);

        // Despawn both
        app.world_mut().despawn(bolt_a);
        app.world_mut().despawn(bolt_b);

        tick(&mut app);

        assert!(
            app.world().get_entity(beam).is_err(),
            "beam entity should be despawned when both bolts are gone"
        );
    }

    #[test]
    fn tick_tether_beam_bolt_a_survives_beam_cleanup() {
        let mut app = damage_test_app();

        let (bolt_a, bolt_b, _beam) =
            spawn_tether_beam(&mut app, Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), 1.0);

        // Despawn bolt_b, keep bolt_a alive
        app.world_mut().despawn(bolt_b);

        tick(&mut app);

        assert!(
            app.world().get_entity(bolt_a).is_ok(),
            "bolt_a should still exist after beam cleanup"
        );
    }

    #[test]
    fn multiple_tether_beams_operate_independently() {
        let mut app = damage_test_app();

        // Beam 1: (0, 0) to (100, 0), damage_mult=1.0
        spawn_tether_beam(&mut app, Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), 1.0);

        // Beam 2: (200, 0) to (300, 0), damage_mult=2.0
        spawn_tether_beam(&mut app, Vec2::new(200.0, 0.0), Vec2::new(300.0, 0.0), 2.0);

        // Cell 1 near beam 1
        let cell1 = spawn_test_cell(&mut app, 50.0, 0.0);
        // Cell 2 near beam 2
        let cell2 = spawn_test_cell(&mut app, 250.0, 0.0);

        tick(&mut app);

        let collector = app.world().resource::<DamageCellCollector>();
        assert_eq!(
            collector.0.len(),
            2,
            "expected 2 DamageCell messages (one per beam), got {}",
            collector.0.len()
        );

        // Find damage for each cell
        let cell1_damage = collector.0.iter().find(|m| m.cell == cell1);
        let cell2_damage = collector.0.iter().find(|m| m.cell == cell2);

        assert!(cell1_damage.is_some(), "cell1 should be damaged by beam 1");
        assert!(cell2_damage.is_some(), "cell2 should be damaged by beam 2");

        assert!(
            (cell1_damage.unwrap().damage - BASE_BOLT_DAMAGE * 1.0).abs() < f32::EPSILON,
            "cell1 damage should be BASE_BOLT_DAMAGE * 1.0 = 10.0"
        );
        assert!(
            (cell2_damage.unwrap().damage - BASE_BOLT_DAMAGE * 2.0).abs() < f32::EPSILON,
            "cell2 damage should be BASE_BOLT_DAMAGE * 2.0 = 20.0"
        );
    }

    #[test]
    fn cell_midway_between_two_beams_not_reached_by_either() {
        let mut app = damage_test_app();

        // Beam 1: (0, 0) to (100, 0)
        spawn_tether_beam(&mut app, Vec2::new(0.0, 0.0), Vec2::new(100.0, 0.0), 1.0);
        // Beam 2: (200, 0) to (300, 0)
        spawn_tether_beam(&mut app, Vec2::new(200.0, 0.0), Vec2::new(300.0, 0.0), 2.0);

        // Cell at (150, 0) — between the two beams, not reached by either
        spawn_test_cell(&mut app, 150.0, 0.0);

        tick(&mut app);

        let collector = app.world().resource::<DamageCellCollector>();
        assert!(
            collector.0.is_empty(),
            "cell at (150, 0) should not be reached by either beam"
        );
    }
}
