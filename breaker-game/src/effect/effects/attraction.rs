use bevy::prelude::*;
use rantzsoft_physics2d::{
    aabb::Aabb2D, collision_layers::CollisionLayers, resources::CollisionQuadtree,
};
use rantzsoft_spatial2d::components::{GlobalPosition2D, Velocity2D};

use crate::{
    bolt::messages::{BoltImpactBreaker, BoltImpactCell, BoltImpactWall},
    effect::core::AttractionType,
    shared::{BREAKER_LAYER, CELL_LAYER, WALL_LAYER},
};

/// Search radius for quadtree AABB query. Large enough to cover the entire
/// playfield so the nearest entity is always found regardless of distance.
const ATTRACTION_SEARCH_RADIUS: f32 = 500.0;

/// An individual attraction entry tracking type, force, and active state.
#[derive(Clone, Debug, PartialEq)]
pub struct AttractionEntry {
    /// Which entity type to attract toward.
    pub attraction_type: AttractionType,
    /// Attraction strength.
    pub force: f32,
    /// Whether this attraction is currently active (deactivates on hit).
    pub active: bool,
}

/// Component holding all active attractions on an entity.
#[derive(Component, Debug, Default, Clone)]
pub struct ActiveAttractions(pub Vec<AttractionEntry>);

/// Adds an attraction entry to the entity.
///
/// Inserts `ActiveAttractions` if not already present.
pub(crate) fn fire(entity: Entity, attraction_type: AttractionType, force: f32, world: &mut World) {
    let entry = AttractionEntry {
        attraction_type,
        force,
        active: true,
    };

    if let Some(mut attractions) = world.get_mut::<ActiveAttractions>(entity) {
        attractions.0.push(entry);
    } else if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
        entity_mut.insert(ActiveAttractions(vec![entry]));
    }
}

/// Removes a matching attraction entry from the entity.
pub(crate) fn reverse(
    entity: Entity,
    attraction_type: AttractionType,
    force: f32,
    world: &mut World,
) {
    if let Some(mut attractions) = world.get_mut::<ActiveAttractions>(entity)
        && let Some(idx) = attractions.0.iter().position(|e| {
            e.attraction_type == attraction_type && (e.force - force).abs() < f32::EPSILON
        })
    {
        attractions.0.remove(idx);
    }
}

/// Steers entities with [`ActiveAttractions`] toward the nearest target of each
/// attracted type using the [`CollisionQuadtree`].
fn apply_attraction(
    time: Res<Time<Fixed>>,
    quadtree: Res<CollisionQuadtree>,
    positions: Query<&GlobalPosition2D>,
    mut attracted: Query<(
        Entity,
        &GlobalPosition2D,
        &mut Velocity2D,
        &ActiveAttractions,
    )>,
) {
    let dt = time.timestep().as_secs_f32();

    for (_entity, global_pos, mut velocity, attractions) in &mut attracted {
        let entity_pos = global_pos.0;

        if !attractions.0.iter().any(|e| e.active) {
            continue;
        }

        // Track the nearest candidate across ALL active attraction types.
        let mut nearest_dist = f32::MAX;
        let mut nearest_pos = Vec2::ZERO;
        let mut nearest_force = 0.0_f32;

        for entry in attractions.0.iter().filter(|e| e.active) {
            let layer = match entry.attraction_type {
                AttractionType::Cell => CELL_LAYER,
                AttractionType::Wall => WALL_LAYER,
                AttractionType::Breaker => BREAKER_LAYER,
            };
            let search_region = Aabb2D::new(entity_pos, Vec2::splat(ATTRACTION_SEARCH_RADIUS));
            let layers = CollisionLayers::new(0, layer);
            let candidates = quadtree
                .quadtree
                .query_aabb_filtered(&search_region, layers);

            for candidate in candidates {
                if let Ok(candidate_pos) = positions.get(candidate) {
                    let dist = entity_pos.distance(candidate_pos.0);
                    if dist < nearest_dist {
                        nearest_dist = dist;
                        nearest_pos = candidate_pos.0;
                        nearest_force = entry.force;
                    }
                }
            }
        }

        if nearest_dist < f32::MAX {
            let direction = (nearest_pos - entity_pos).normalize_or_zero();
            velocity.x += direction.x * nearest_force * dt;
            velocity.y += direction.y * nearest_force * dt;
        }
    }
}

/// Reads bolt impact messages and deactivates attraction entries on hit with an
/// attracted type, reactivates all deactivated entries on hit with a
/// non-attracted type.
fn manage_attraction_types(
    mut impact_cell: MessageReader<BoltImpactCell>,
    mut impact_wall: MessageReader<BoltImpactWall>,
    mut impact_breaker: MessageReader<BoltImpactBreaker>,
    mut attracted: Query<&mut ActiveAttractions>,
) {
    // Process cell impacts.
    for msg in impact_cell.read() {
        if let Ok(mut attractions) = attracted.get_mut(msg.bolt) {
            process_impact(&mut attractions, AttractionType::Cell);
        }
    }

    // Process wall impacts.
    for msg in impact_wall.read() {
        if let Ok(mut attractions) = attracted.get_mut(msg.bolt) {
            process_impact(&mut attractions, AttractionType::Wall);
        }
    }

    // Process breaker impacts.
    for msg in impact_breaker.read() {
        if let Ok(mut attractions) = attracted.get_mut(msg.bolt) {
            process_impact(&mut attractions, AttractionType::Breaker);
        }
    }
}

/// Handles an impact for a specific attraction type.
///
/// If the entity has an entry for the impact type, deactivate matching entries.
/// If it does NOT have an entry for the impact type, reactivate all entries.
fn process_impact(attractions: &mut ActiveAttractions, impact_type: AttractionType) {
    let has_type = attractions
        .0
        .iter()
        .any(|e| e.attraction_type == impact_type);

    if has_type {
        // Deactivate all entries of the matching type.
        for entry in &mut attractions.0 {
            if entry.attraction_type == impact_type {
                entry.active = false;
            }
        }
    } else {
        // Reactivate all deactivated entries.
        for entry in &mut attractions.0 {
            entry.active = true;
        }
    }
}

/// Registers attraction systems in `FixedUpdate`.
pub(crate) fn register(app: &mut App) {
    use crate::shared::PlayingState;
    app.add_systems(
        FixedUpdate,
        (
            apply_attraction.after(rantzsoft_physics2d::plugin::PhysicsSystems::MaintainQuadtree),
            manage_attraction_types,
        )
            .run_if(in_state(PlayingState::Active)),
    );
}

#[cfg(test)]
mod tests {
    use rantzsoft_physics2d::{
        aabb::Aabb2D, collision_layers::CollisionLayers, resources::CollisionQuadtree,
    };
    use rantzsoft_spatial2d::components::{GlobalPosition2D, Velocity2D};

    use super::*;
    use crate::{
        bolt::messages::{BoltImpactBreaker, BoltImpactCell, BoltImpactWall},
        shared::{BREAKER_LAYER, CELL_LAYER, WALL_LAYER},
    };

    // ── fire/reverse unit tests (existing) ──────────────────────────

    #[test]
    fn fire_inserts_active_attractions_on_fresh_entity() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        fire(entity, AttractionType::Cell, 10.0, &mut world);

        let attractions = world.get::<ActiveAttractions>(entity).unwrap();
        assert_eq!(attractions.0.len(), 1);
        assert_eq!(attractions.0[0].attraction_type, AttractionType::Cell);
        assert!((attractions.0[0].force - 10.0).abs() < f32::EPSILON);
        assert!(attractions.0[0].active);
    }

    #[test]
    fn fire_appends_entry_to_existing_active_attractions() {
        let mut world = World::new();
        let entity = world
            .spawn(ActiveAttractions(vec![AttractionEntry {
                attraction_type: AttractionType::Wall,
                force: 5.0,
                active: true,
            }]))
            .id();

        fire(entity, AttractionType::Breaker, 15.0, &mut world);

        let attractions = world.get::<ActiveAttractions>(entity).unwrap();
        assert_eq!(
            attractions.0.len(),
            2,
            "should have two entries after appending"
        );
        assert_eq!(attractions.0[1].attraction_type, AttractionType::Breaker);
        assert!((attractions.0[1].force - 15.0).abs() < f32::EPSILON);
    }

    #[test]
    fn reverse_removes_matching_entry() {
        let mut world = World::new();
        let entity = world
            .spawn(ActiveAttractions(vec![
                AttractionEntry {
                    attraction_type: AttractionType::Cell,
                    force: 10.0,
                    active: true,
                },
                AttractionEntry {
                    attraction_type: AttractionType::Wall,
                    force: 5.0,
                    active: true,
                },
            ]))
            .id();

        reverse(entity, AttractionType::Cell, 10.0, &mut world);

        let attractions = world.get::<ActiveAttractions>(entity).unwrap();
        assert_eq!(attractions.0.len(), 1, "matching entry should be removed");
        assert_eq!(attractions.0[0].attraction_type, AttractionType::Wall);
    }

    #[test]
    fn reverse_with_no_match_is_noop() {
        let mut world = World::new();
        let entity = world
            .spawn(ActiveAttractions(vec![AttractionEntry {
                attraction_type: AttractionType::Cell,
                force: 10.0,
                active: true,
            }]))
            .id();

        // Different type — no match.
        reverse(entity, AttractionType::Breaker, 10.0, &mut world);

        let attractions = world.get::<ActiveAttractions>(entity).unwrap();
        assert_eq!(
            attractions.0.len(),
            1,
            "no entry should be removed when no match"
        );
    }

    // ── apply_attraction system tests ───────────────────────────────

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<crate::shared::game_state::GameState>();
        app.add_sub_state::<crate::shared::PlayingState>();
        app.insert_resource(CollisionQuadtree::default());
        app.add_systems(Update, apply_attraction);
        app
    }

    fn test_app_with_manage() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<BoltImpactCell>();
        app.add_message::<BoltImpactWall>();
        app.add_message::<BoltImpactBreaker>();
        app.add_systems(
            FixedUpdate,
            (
                enqueue_messages.before(manage_attraction_types),
                manage_attraction_types,
            ),
        );
        app
    }

    fn enter_playing(app: &mut App) {
        app.world_mut()
            .resource_mut::<NextState<crate::shared::game_state::GameState>>()
            .set(crate::shared::game_state::GameState::Playing);
        app.update();
    }

    /// Populate the quadtree with entities that have positions and collision layers.
    fn populate_quadtree(app: &mut App, entries: &[(Entity, Vec2, CollisionLayers)]) {
        let mut quadtree = app.world_mut().resource_mut::<CollisionQuadtree>();
        for &(entity, pos, layers) in entries {
            quadtree
                .quadtree
                .insert(entity, Aabb2D::new(pos, Vec2::new(8.0, 8.0)), layers);
        }
    }

    #[test]
    fn apply_attraction_steers_toward_nearest_cell_target() {
        let mut app = test_app();
        enter_playing(&mut app);

        // Entity A at origin with Cell attraction
        let entity_a = app
            .world_mut()
            .spawn((
                GlobalPosition2D(Vec2::new(0.0, 0.0)),
                Velocity2D(Vec2::ZERO),
                ActiveAttractions(vec![AttractionEntry {
                    attraction_type: AttractionType::Cell,
                    force: 500.0,
                    active: true,
                }]),
            ))
            .id();

        // Cell target at (100, 0)
        let cell = app
            .world_mut()
            .spawn(GlobalPosition2D(Vec2::new(100.0, 0.0)))
            .id();

        populate_quadtree(
            &mut app,
            &[(
                cell,
                Vec2::new(100.0, 0.0),
                CollisionLayers::new(CELL_LAYER, 0),
            )],
        );

        app.update();

        let velocity = app.world().get::<Velocity2D>(entity_a).unwrap();
        assert!(
            velocity.x > 0.0,
            "entity should be steered toward cell at +x, got velocity.x = {}",
            velocity.x
        );
    }

    #[test]
    fn apply_attraction_zero_distance_no_steering() {
        let mut app = test_app();
        enter_playing(&mut app);

        // Entity A at same position as cell
        let entity_a = app
            .world_mut()
            .spawn((
                GlobalPosition2D(Vec2::new(100.0, 0.0)),
                Velocity2D(Vec2::ZERO),
                ActiveAttractions(vec![AttractionEntry {
                    attraction_type: AttractionType::Cell,
                    force: 500.0,
                    active: true,
                }]),
            ))
            .id();

        let cell = app
            .world_mut()
            .spawn(GlobalPosition2D(Vec2::new(100.0, 0.0)))
            .id();

        populate_quadtree(
            &mut app,
            &[(
                cell,
                Vec2::new(100.0, 0.0),
                CollisionLayers::new(CELL_LAYER, 0),
            )],
        );

        app.update();

        let velocity = app.world().get::<Velocity2D>(entity_a).unwrap();
        assert_eq!(
            velocity.0,
            Vec2::ZERO,
            "zero distance should produce no steering, got {:?}",
            velocity.0
        );
    }

    #[test]
    fn apply_attraction_inactive_entry_produces_no_steering() {
        let mut app = test_app();
        enter_playing(&mut app);

        let entity_a = app
            .world_mut()
            .spawn((
                GlobalPosition2D(Vec2::new(0.0, 0.0)),
                Velocity2D(Vec2::ZERO),
                ActiveAttractions(vec![AttractionEntry {
                    attraction_type: AttractionType::Cell,
                    force: 500.0,
                    active: false,
                }]),
            ))
            .id();

        let cell = app
            .world_mut()
            .spawn(GlobalPosition2D(Vec2::new(100.0, 0.0)))
            .id();

        populate_quadtree(
            &mut app,
            &[(
                cell,
                Vec2::new(100.0, 0.0),
                CollisionLayers::new(CELL_LAYER, 0),
            )],
        );

        app.update();

        let velocity = app.world().get::<Velocity2D>(entity_a).unwrap();
        assert_eq!(
            velocity.0,
            Vec2::ZERO,
            "inactive attraction should produce no steering, got {:?}",
            velocity.0
        );
    }

    #[test]
    fn apply_attraction_mixed_active_inactive_only_active_steers() {
        let mut app = test_app();
        enter_playing(&mut app);

        let entity_a = app
            .world_mut()
            .spawn((
                GlobalPosition2D(Vec2::new(0.0, 0.0)),
                Velocity2D(Vec2::ZERO),
                ActiveAttractions(vec![
                    AttractionEntry {
                        attraction_type: AttractionType::Cell,
                        force: 500.0,
                        active: false,
                    },
                    AttractionEntry {
                        attraction_type: AttractionType::Wall,
                        force: 300.0,
                        active: true,
                    },
                ]),
            ))
            .id();

        // Wall target at (0, 100)
        let wall = app
            .world_mut()
            .spawn(GlobalPosition2D(Vec2::new(0.0, 100.0)))
            .id();

        populate_quadtree(
            &mut app,
            &[(
                wall,
                Vec2::new(0.0, 100.0),
                CollisionLayers::new(WALL_LAYER, 0),
            )],
        );

        app.update();

        let velocity = app.world().get::<Velocity2D>(entity_a).unwrap();
        assert!(
            velocity.y > 0.0,
            "only active Wall attraction should steer toward +y, got velocity.y = {}",
            velocity.y
        );
    }

    #[test]
    fn apply_attraction_multiple_types_nearest_target_wins() {
        let mut app = test_app();
        enter_playing(&mut app);

        let entity_a = app
            .world_mut()
            .spawn((
                GlobalPosition2D(Vec2::new(0.0, 0.0)),
                Velocity2D(Vec2::ZERO),
                ActiveAttractions(vec![
                    AttractionEntry {
                        attraction_type: AttractionType::Cell,
                        force: 500.0,
                        active: true,
                    },
                    AttractionEntry {
                        attraction_type: AttractionType::Wall,
                        force: 500.0,
                        active: true,
                    },
                ]),
            ))
            .id();

        // Cell at (200, 0) — farther
        let cell = app
            .world_mut()
            .spawn(GlobalPosition2D(Vec2::new(200.0, 0.0)))
            .id();

        // Wall at (50, 0) — closer
        let wall = app
            .world_mut()
            .spawn(GlobalPosition2D(Vec2::new(50.0, 0.0)))
            .id();

        populate_quadtree(
            &mut app,
            &[
                (
                    cell,
                    Vec2::new(200.0, 0.0),
                    CollisionLayers::new(CELL_LAYER, 0),
                ),
                (
                    wall,
                    Vec2::new(50.0, 0.0),
                    CollisionLayers::new(WALL_LAYER, 0),
                ),
            ],
        );

        app.update();

        let velocity = app.world().get::<Velocity2D>(entity_a).unwrap();
        // Steered toward wall at (50, 0) — velocity x should be positive
        assert!(
            velocity.x > 0.0,
            "should steer toward nearest target (wall at 50,0), got velocity.x = {}",
            velocity.x
        );
    }

    #[test]
    fn apply_attraction_only_queries_matching_layer() {
        let mut app = test_app();
        enter_playing(&mut app);

        let entity_a = app
            .world_mut()
            .spawn((
                GlobalPosition2D(Vec2::new(0.0, 0.0)),
                Velocity2D(Vec2::ZERO),
                ActiveAttractions(vec![AttractionEntry {
                    attraction_type: AttractionType::Cell,
                    force: 500.0,
                    active: true,
                }]),
            ))
            .id();

        // Cell at (100, 0) with CELL_LAYER
        let cell = app
            .world_mut()
            .spawn(GlobalPosition2D(Vec2::new(100.0, 0.0)))
            .id();

        // Wall at (0, 100) with WALL_LAYER — should NOT be a target for Cell attraction
        let wall = app
            .world_mut()
            .spawn(GlobalPosition2D(Vec2::new(0.0, 100.0)))
            .id();

        // Breaker at (-100, 0) with BREAKER_LAYER — should NOT be a target
        let breaker = app
            .world_mut()
            .spawn(GlobalPosition2D(Vec2::new(-100.0, 0.0)))
            .id();

        populate_quadtree(
            &mut app,
            &[
                (
                    cell,
                    Vec2::new(100.0, 0.0),
                    CollisionLayers::new(CELL_LAYER, 0),
                ),
                (
                    wall,
                    Vec2::new(0.0, 100.0),
                    CollisionLayers::new(WALL_LAYER, 0),
                ),
                (
                    breaker,
                    Vec2::new(-100.0, 0.0),
                    CollisionLayers::new(BREAKER_LAYER, 0),
                ),
            ],
        );

        app.update();

        let velocity = app.world().get::<Velocity2D>(entity_a).unwrap();
        // Should steer only toward the cell at (100, 0)
        assert!(
            velocity.x > 0.0,
            "Cell attraction should steer toward cell at (100,0), got velocity.x = {}",
            velocity.x
        );
        // Y should be zero or negligible (only steered in +x direction toward cell)
        assert!(
            velocity.y.abs() < 0.01,
            "Cell attraction should not steer toward wall or breaker, got velocity.y = {}",
            velocity.y
        );
    }

    #[test]
    fn apply_attraction_no_targets_velocity_unchanged() {
        let mut app = test_app();
        enter_playing(&mut app);

        let entity_a = app
            .world_mut()
            .spawn((
                GlobalPosition2D(Vec2::new(0.0, 0.0)),
                Velocity2D(Vec2::new(100.0, 200.0)),
                ActiveAttractions(vec![AttractionEntry {
                    attraction_type: AttractionType::Cell,
                    force: 500.0,
                    active: true,
                }]),
            ))
            .id();

        // Empty quadtree — no targets
        app.update();

        let velocity = app.world().get::<Velocity2D>(entity_a).unwrap();
        assert!(
            (velocity.x - 100.0).abs() < f32::EPSILON,
            "velocity.x should be unchanged (100.0), got {}",
            velocity.x
        );
        assert!(
            (velocity.y - 200.0).abs() < f32::EPSILON,
            "velocity.y should be unchanged (200.0), got {}",
            velocity.y
        );
    }

    #[test]
    fn apply_attraction_entity_without_attractions_unaffected() {
        let mut app = test_app();
        enter_playing(&mut app);

        // Entity without ActiveAttractions
        let entity_a = app
            .world_mut()
            .spawn((
                GlobalPosition2D(Vec2::new(0.0, 0.0)),
                Velocity2D(Vec2::new(100.0, 200.0)),
            ))
            .id();

        let cell = app
            .world_mut()
            .spawn(GlobalPosition2D(Vec2::new(100.0, 0.0)))
            .id();

        populate_quadtree(
            &mut app,
            &[(
                cell,
                Vec2::new(100.0, 0.0),
                CollisionLayers::new(CELL_LAYER, 0),
            )],
        );

        app.update();

        let velocity = app.world().get::<Velocity2D>(entity_a).unwrap();
        assert!(
            (velocity.x - 100.0).abs() < f32::EPSILON,
            "entity without ActiveAttractions should not be steered, got velocity.x = {}",
            velocity.x
        );
    }

    #[test]
    fn apply_attraction_empty_attractions_no_steering() {
        let mut app = test_app();
        enter_playing(&mut app);

        let entity_a = app
            .world_mut()
            .spawn((
                GlobalPosition2D(Vec2::new(0.0, 0.0)),
                Velocity2D(Vec2::new(100.0, 200.0)),
                ActiveAttractions(vec![]),
            ))
            .id();

        let cell = app
            .world_mut()
            .spawn(GlobalPosition2D(Vec2::new(100.0, 0.0)))
            .id();

        populate_quadtree(
            &mut app,
            &[(
                cell,
                Vec2::new(100.0, 0.0),
                CollisionLayers::new(CELL_LAYER, 0),
            )],
        );

        app.update();

        let velocity = app.world().get::<Velocity2D>(entity_a).unwrap();
        assert!(
            (velocity.x - 100.0).abs() < f32::EPSILON,
            "empty ActiveAttractions should produce no steering, got velocity.x = {}",
            velocity.x
        );
    }

    #[test]
    fn apply_attraction_force_scales_with_dt() {
        let mut app = test_app();
        enter_playing(&mut app);

        let entity_a = app
            .world_mut()
            .spawn((
                GlobalPosition2D(Vec2::new(0.0, 0.0)),
                Velocity2D(Vec2::ZERO),
                ActiveAttractions(vec![AttractionEntry {
                    attraction_type: AttractionType::Cell,
                    force: 1000.0,
                    active: true,
                }]),
            ))
            .id();

        let cell = app
            .world_mut()
            .spawn(GlobalPosition2D(Vec2::new(100.0, 0.0)))
            .id();

        populate_quadtree(
            &mut app,
            &[(
                cell,
                Vec2::new(100.0, 0.0),
                CollisionLayers::new(CELL_LAYER, 0),
            )],
        );

        app.update();

        let velocity = app.world().get::<Velocity2D>(entity_a).unwrap();
        // Direction toward (100, 0) from (0, 0) is (1.0, 0.0).
        // With force=1000 and default dt=~0.015625 (1/64), velocity.x should be
        // approximately 1000 * 0.015625 = 15.625.
        // We use a generous tolerance because the exact dt may vary with MinimalPlugins.
        assert!(
            velocity.x > 0.0,
            "velocity.x should be positive after attraction force * dt, got {}",
            velocity.x
        );
        // Verify the velocity is proportional to force (not just direction)
        assert!(
            velocity.x > 1.0,
            "with force=1000 and any reasonable dt, velocity.x should be > 1.0, got {}",
            velocity.x
        );
    }

    // ── manage_attraction_types system tests ────────────────────────

    /// Resource holding test impact messages to enqueue before the system runs.
    #[derive(Resource, Default)]
    struct TestImpactMessages {
        cell: Vec<BoltImpactCell>,
        wall: Vec<BoltImpactWall>,
        breaker: Vec<BoltImpactBreaker>,
    }

    fn enqueue_messages(
        msgs: Res<TestImpactMessages>,
        mut cell_writer: MessageWriter<BoltImpactCell>,
        mut wall_writer: MessageWriter<BoltImpactWall>,
        mut breaker_writer: MessageWriter<BoltImpactBreaker>,
    ) {
        for msg in &msgs.cell {
            cell_writer.write(msg.clone());
        }
        for msg in &msgs.wall {
            wall_writer.write(msg.clone());
        }
        for msg in &msgs.breaker {
            breaker_writer.write(msg.clone());
        }
    }

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    #[test]
    fn manage_attraction_cell_impact_deactivates_cell_entry() {
        let mut app = test_app_with_manage();

        let bolt = app
            .world_mut()
            .spawn(ActiveAttractions(vec![
                AttractionEntry {
                    attraction_type: AttractionType::Cell,
                    force: 500.0,
                    active: true,
                },
                AttractionEntry {
                    attraction_type: AttractionType::Wall,
                    force: 300.0,
                    active: true,
                },
            ]))
            .id();

        app.insert_resource(TestImpactMessages {
            cell: vec![BoltImpactCell {
                bolt,
                cell: Entity::PLACEHOLDER,
            }],
            ..default()
        });

        tick(&mut app);

        let attractions = app.world().get::<ActiveAttractions>(bolt).unwrap();
        // Cell entry should be deactivated
        let cell_entry = attractions
            .0
            .iter()
            .find(|e| e.attraction_type == AttractionType::Cell)
            .expect("Cell entry should still exist");
        assert!(
            !cell_entry.active,
            "Cell attraction should be deactivated after BoltImpactCell"
        );
        // Wall entry should remain active
        let wall_entry = attractions
            .0
            .iter()
            .find(|e| e.attraction_type == AttractionType::Wall)
            .expect("Wall entry should still exist");
        assert!(
            wall_entry.active,
            "Wall attraction should remain active after BoltImpactCell"
        );
    }

    #[test]
    fn manage_attraction_non_attracted_type_impact_reactivates_all() {
        let mut app = test_app_with_manage();

        let bolt = app
            .world_mut()
            .spawn(ActiveAttractions(vec![AttractionEntry {
                attraction_type: AttractionType::Cell,
                force: 500.0,
                active: false,
            }]))
            .id();

        // Wall impact — bolt has no Wall entry, so this is a non-attracted type
        app.insert_resource(TestImpactMessages {
            wall: vec![BoltImpactWall {
                bolt,
                wall: Entity::PLACEHOLDER,
            }],
            ..default()
        });

        tick(&mut app);

        let attractions = app.world().get::<ActiveAttractions>(bolt).unwrap();
        assert!(
            attractions.0[0].active,
            "Cell entry should be reactivated after non-attracted-type impact (wall)"
        );
    }

    #[test]
    fn manage_attraction_wall_impact_deactivates_wall_entry() {
        let mut app = test_app_with_manage();

        let bolt = app
            .world_mut()
            .spawn(ActiveAttractions(vec![AttractionEntry {
                attraction_type: AttractionType::Wall,
                force: 300.0,
                active: true,
            }]))
            .id();

        app.insert_resource(TestImpactMessages {
            wall: vec![BoltImpactWall {
                bolt,
                wall: Entity::PLACEHOLDER,
            }],
            ..default()
        });

        tick(&mut app);

        let attractions = app.world().get::<ActiveAttractions>(bolt).unwrap();
        assert!(
            !attractions.0[0].active,
            "Wall attraction should be deactivated after BoltImpactWall"
        );
    }

    #[test]
    fn manage_attraction_breaker_impact_deactivates_breaker_entry() {
        let mut app = test_app_with_manage();

        let bolt = app
            .world_mut()
            .spawn(ActiveAttractions(vec![AttractionEntry {
                attraction_type: AttractionType::Breaker,
                force: 200.0,
                active: true,
            }]))
            .id();

        app.insert_resource(TestImpactMessages {
            breaker: vec![BoltImpactBreaker {
                bolt,
                breaker: Entity::PLACEHOLDER,
            }],
            ..default()
        });

        tick(&mut app);

        let attractions = app.world().get::<ActiveAttractions>(bolt).unwrap();
        assert!(
            !attractions.0[0].active,
            "Breaker attraction should be deactivated after BoltImpactBreaker"
        );
    }

    #[test]
    fn manage_attraction_impact_for_different_bolt_is_ignored() {
        let mut app = test_app_with_manage();

        let bolt_a = app
            .world_mut()
            .spawn(ActiveAttractions(vec![AttractionEntry {
                attraction_type: AttractionType::Cell,
                force: 500.0,
                active: true,
            }]))
            .id();

        let bolt_b = app.world_mut().spawn_empty().id();

        // Impact message is for bolt_b, not bolt_a
        app.insert_resource(TestImpactMessages {
            cell: vec![BoltImpactCell {
                bolt: bolt_b,
                cell: Entity::PLACEHOLDER,
            }],
            ..default()
        });

        tick(&mut app);

        let attractions = app.world().get::<ActiveAttractions>(bolt_a).unwrap();
        assert!(
            attractions.0[0].active,
            "bolt_a's attractions should be unchanged when impact was for bolt_b"
        );
    }

    #[test]
    fn manage_attraction_attracted_type_already_inactive_no_reactivation() {
        let mut app = test_app_with_manage();

        let bolt = app
            .world_mut()
            .spawn(ActiveAttractions(vec![
                AttractionEntry {
                    attraction_type: AttractionType::Cell,
                    force: 500.0,
                    active: false,
                },
                AttractionEntry {
                    attraction_type: AttractionType::Wall,
                    force: 300.0,
                    active: false,
                },
            ]))
            .id();

        // Cell impact — Cell IS an attracted type, so Wall should NOT be reactivated
        app.insert_resource(TestImpactMessages {
            cell: vec![BoltImpactCell {
                bolt,
                cell: Entity::PLACEHOLDER,
            }],
            ..default()
        });

        tick(&mut app);

        let attractions = app.world().get::<ActiveAttractions>(bolt).unwrap();
        let wall_entry = attractions
            .0
            .iter()
            .find(|e| e.attraction_type == AttractionType::Wall)
            .expect("Wall entry should still exist");
        assert!(
            !wall_entry.active,
            "Wall entry should NOT be reactivated when impact is with an attracted type (Cell)"
        );
    }

    #[test]
    fn manage_attraction_multiple_cell_entries_all_deactivated() {
        let mut app = test_app_with_manage();

        let bolt = app
            .world_mut()
            .spawn(ActiveAttractions(vec![
                AttractionEntry {
                    attraction_type: AttractionType::Cell,
                    force: 500.0,
                    active: true,
                },
                AttractionEntry {
                    attraction_type: AttractionType::Cell,
                    force: 300.0,
                    active: true,
                },
            ]))
            .id();

        app.insert_resource(TestImpactMessages {
            cell: vec![BoltImpactCell {
                bolt,
                cell: Entity::PLACEHOLDER,
            }],
            ..default()
        });

        tick(&mut app);

        let attractions = app.world().get::<ActiveAttractions>(bolt).unwrap();
        for entry in &attractions.0 {
            assert!(
                !entry.active,
                "all Cell entries should be deactivated, but force={} is still active",
                entry.force
            );
        }
    }

    #[test]
    fn manage_attraction_all_already_active_reactivation_is_noop() {
        let mut app = test_app_with_manage();

        let bolt = app
            .world_mut()
            .spawn(ActiveAttractions(vec![AttractionEntry {
                attraction_type: AttractionType::Cell,
                force: 500.0,
                active: true,
            }]))
            .id();

        // Wall impact — bolt has no Wall entry, so this would trigger reactivation
        // but everything is already active, so it's a no-op
        app.insert_resource(TestImpactMessages {
            wall: vec![BoltImpactWall {
                bolt,
                wall: Entity::PLACEHOLDER,
            }],
            ..default()
        });

        tick(&mut app);

        let attractions = app.world().get::<ActiveAttractions>(bolt).unwrap();
        assert!(
            attractions.0[0].active,
            "already active entry should remain active after reactivation no-op"
        );
    }
}
