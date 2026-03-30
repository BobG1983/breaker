use bevy::prelude::*;
use rantzsoft_spatial2d::prelude::*;

use crate::{
    bolt::components::Bolt,
    shared::{CleanupOnNodeExit, playing_state::PlayingState},
};

/// Marker for gravity well entities.
#[derive(Component)]
pub(crate) struct GravityWellMarker;

/// Configuration and runtime state for a gravity well.
#[derive(Component)]
pub(crate) struct GravityWellConfig {
    /// Pull strength applied to bolts within radius.
    pub strength: f32,
    /// Attraction radius in world units.
    pub radius: f32,
    /// Remaining duration in seconds.
    pub remaining: f32,
    /// Entity that spawned this well.
    pub owner: Entity,
}

pub(crate) fn fire(
    entity: Entity,
    strength: f32,
    duration: f32,
    radius: f32,
    max: u32,
    _source_chip: &str,
    world: &mut World,
) {
    if max == 0 {
        return;
    }

    let position = super::entity_position(world, entity);

    // Enforce max active wells for this owner — despawn oldest if at cap.
    let mut owned: Vec<Entity> = Vec::new();
    {
        let mut query = world.query::<(Entity, &GravityWellConfig)>();
        for (well_entity, config) in query.iter(world) {
            if config.owner == entity {
                owned.push(well_entity);
            }
        }
    }

    // Despawn order is arbitrary (ECS query iteration is not guaranteed FIFO).
    while owned.len() >= max as usize {
        if let Some(oldest) = owned.first().copied() {
            world.despawn(oldest);
            owned.remove(0);
        }
    }

    world.spawn((
        GravityWellMarker,
        GravityWellConfig {
            strength,
            radius,
            remaining: duration,
            owner: entity,
        },
        Position2D(position),
        CleanupOnNodeExit,
    ));
}

/// No-op — gravity wells self-despawn via their duration timer.
pub(crate) const fn reverse(_entity: Entity, _source_chip: &str, _world: &mut World) {}

/// Decrement well timers and despawn expired wells.
fn tick_gravity_well(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut GravityWellConfig), With<GravityWellMarker>>,
) {
    let dt = time.delta_secs();
    for (entity, mut config) in &mut query {
        config.remaining -= dt;
        if config.remaining <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Pull bolts toward active gravity wells.
fn apply_gravity_pull(
    time: Res<Time>,
    wells: Query<(&Position2D, &GravityWellConfig), With<GravityWellMarker>>,
    mut bolts: Query<(&Position2D, &mut Velocity2D), With<Bolt>>,
) {
    let dt = time.delta_secs();
    for (well_position, config) in &wells {
        let well_pos = well_position.0;
        for (bolt_position, mut velocity) in &mut bolts {
            let bolt_pos = bolt_position.0;
            let delta = well_pos - bolt_pos;
            let distance = delta.length();
            if distance > 0.0 && distance <= config.radius {
                let direction = delta / distance;
                let pull = config.strength * dt;
                velocity.x = direction.x.mul_add(pull, velocity.x);
                velocity.y = direction.y.mul_add(pull, velocity.y);
            }
        }
    }
}

pub(crate) fn register(app: &mut App) {
    use crate::bolt::BoltSystems;

    app.add_systems(
        FixedUpdate,
        (
            tick_gravity_well,
            apply_gravity_pull.before(BoltSystems::PrepareVelocity),
        )
            .run_if(in_state(PlayingState::Active)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── fire tests ──────────────────────────────────────────────────

    #[test]
    fn fire_with_max_zero_returns_immediately() {
        let mut world = World::new();
        let entity = world.spawn(Position2D(Vec2::new(50.0, 75.0))).id();

        fire(entity, 100.0, 5.0, 80.0, 0, "", &mut world);

        let mut query = world.query::<&GravityWellConfig>();
        let count = query.iter(&world).count();
        assert_eq!(count, 0, "no well entities should be spawned when max is 0");
    }

    #[test]
    fn fire_spawns_well_entity_with_marker_and_config() {
        let mut world = World::new();
        let entity = world.spawn(Position2D(Vec2::new(50.0, 75.0))).id();

        fire(entity, 100.0, 5.0, 80.0, 3, "", &mut world);

        let mut query = world.query::<(&GravityWellMarker, &GravityWellConfig, &Position2D)>();
        let results: Vec<_> = query.iter(&world).collect();
        assert_eq!(results.len(), 1, "expected exactly one gravity well");

        let (_marker, config, position) = results[0];
        assert!(
            (config.strength - 100.0).abs() < f32::EPSILON,
            "expected strength 100.0, got {}",
            config.strength
        );
        assert!(
            (config.radius - 80.0).abs() < f32::EPSILON,
            "expected radius 80.0, got {}",
            config.radius
        );
        assert!(
            (config.remaining - 5.0).abs() < f32::EPSILON,
            "expected remaining 5.0, got {}",
            config.remaining
        );
        assert_eq!(config.owner, entity);
        assert!(
            (position.0.x - 50.0).abs() < f32::EPSILON,
            "expected x 50.0, got {}",
            position.0.x
        );
        assert!(
            (position.0.y - 75.0).abs() < f32::EPSILON,
            "expected y 75.0, got {}",
            position.0.y
        );
    }

    #[test]
    fn fire_enforces_max_cap_despawns_oldest() {
        let mut world = World::new();
        let entity = world.spawn(Position2D(Vec2::ZERO)).id();

        // Spawn 3 wells with max=2
        fire(entity, 100.0, 5.0, 80.0, 2, "", &mut world);
        fire(entity, 100.0, 5.0, 80.0, 2, "", &mut world);
        fire(entity, 100.0, 5.0, 80.0, 2, "", &mut world);

        let mut query = world.query::<&GravityWellConfig>();
        let count = query.iter(&world).count();
        assert_eq!(count, 2, "should enforce max of 2 wells, got {count}");
    }

    #[test]
    fn reverse_is_noop() {
        let mut world = World::new();
        let owner = world.spawn(Position2D(Vec2::ZERO)).id();

        fire(owner, 100.0, 5.0, 80.0, 10, "", &mut world);
        reverse(owner, "", &mut world);

        // Wells should still exist — reverse is a no-op
        let mut query = world.query::<&GravityWellConfig>();
        let count = query.iter(&world).count();
        assert_eq!(count, 1, "reverse should not despawn wells (no-op)");
    }

    // ── system tests ────────────────────────────────────────────────

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<crate::shared::game_state::GameState>();
        app.add_sub_state::<PlayingState>();
        app.add_systems(Update, tick_gravity_well);
        app.add_systems(Update, apply_gravity_pull);
        app
    }

    fn enter_playing(app: &mut App) {
        app.world_mut()
            .resource_mut::<NextState<crate::shared::game_state::GameState>>()
            .set(crate::shared::game_state::GameState::Playing);
        app.update();
    }

    #[test]
    fn tick_gravity_well_despawns_expired_wells() {
        let mut app = test_app();
        enter_playing(&mut app);

        let well = app
            .world_mut()
            .spawn((
                GravityWellMarker,
                GravityWellConfig {
                    strength: 100.0,
                    radius: 80.0,
                    remaining: 0.0,
                    owner: Entity::PLACEHOLDER,
                },
                Position2D(Vec2::ZERO),
            ))
            .id();

        app.update();

        assert!(
            app.world().get_entity(well).is_err(),
            "expired gravity well should be despawned"
        );
    }

    #[test]
    fn apply_gravity_pull_steers_bolt_toward_well_within_radius() {
        let mut app = test_app();
        enter_playing(&mut app);

        // Gravity well at origin with large radius
        app.world_mut().spawn((
            GravityWellMarker,
            GravityWellConfig {
                strength: 500.0,
                radius: 200.0,
                remaining: 10.0,
                owner: Entity::PLACEHOLDER,
            },
            Position2D(Vec2::ZERO),
        ));

        // Bolt at (100, 0) with zero velocity — should be pulled toward (0,0)
        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::ZERO),
                Position2D(Vec2::new(100.0, 0.0)),
            ))
            .id();

        app.update();

        let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
        // Bolt should have been pulled in the -x direction (toward the well)
        assert!(
            velocity.x < 0.0,
            "bolt velocity x should be negative (pulled toward well), got {}",
            velocity.x
        );
    }

    // ── fire() reads Position2D, not Transform ────────────────────

    #[test]
    fn fire_reads_position2d_not_transform_for_well_spawn_position() {
        let mut world = World::new();
        // Position2D and Transform are deliberately different to catch the wrong read.
        let entity = world
            .spawn((
                Position2D(Vec2::new(100.0, 200.0)),
                Transform::from_xyz(999.0, 999.0, 0.0),
            ))
            .id();

        fire(entity, 100.0, 5.0, 80.0, 3, "", &mut world);

        let mut query = world.query::<(&GravityWellMarker, &Position2D)>();
        let results: Vec<_> = query.iter(&world).collect();
        assert_eq!(results.len(), 1, "expected exactly one gravity well");

        let (_marker, pos) = results[0];
        assert_eq!(
            pos.0,
            Vec2::new(100.0, 200.0),
            "well should spawn at Position2D (100, 200), not Transform (999, 999)"
        );
    }

    #[test]
    fn fire_reads_position2d_zero_not_transform_for_well_spawn_position() {
        let mut world = World::new();
        // Edge case: Position2D at origin, Transform at a non-zero position.
        let entity = world
            .spawn((Position2D(Vec2::ZERO), Transform::from_xyz(50.0, 50.0, 0.0)))
            .id();

        fire(entity, 100.0, 5.0, 80.0, 3, "", &mut world);

        let mut query = world.query::<(&GravityWellMarker, &Position2D)>();
        let results: Vec<_> = query.iter(&world).collect();
        assert_eq!(results.len(), 1, "expected exactly one gravity well");

        let (_marker, pos) = results[0];
        assert_eq!(
            pos.0,
            Vec2::ZERO,
            "well should spawn at Position2D (0, 0), not Transform (50, 50)"
        );
    }

    #[test]
    fn fire_falls_back_to_zero_when_entity_has_no_position2d() {
        let mut world = World::new();
        // Entity has only Transform, no Position2D. fire() should fall back to Vec2::ZERO.
        let entity = world.spawn(Transform::from_xyz(50.0, 75.0, 0.0)).id();

        fire(entity, 100.0, 5.0, 80.0, 3, "", &mut world);

        let mut query = world.query::<(&GravityWellMarker, &Position2D)>();
        let results: Vec<_> = query.iter(&world).collect();
        assert_eq!(results.len(), 1, "expected exactly one gravity well");

        let (_marker, pos) = results[0];
        assert_eq!(
            pos.0,
            Vec2::ZERO,
            "well should default to Position2D(Vec2::ZERO) when owner has no Position2D"
        );
    }

    #[test]
    fn fire_falls_back_to_zero_when_entity_is_empty() {
        let mut world = World::new();
        // Entity has neither Position2D nor Transform.
        let entity = world.spawn_empty().id();

        fire(entity, 100.0, 5.0, 80.0, 3, "", &mut world);

        let mut query = world.query::<(&GravityWellMarker, &Position2D)>();
        let results: Vec<_> = query.iter(&world).collect();
        assert_eq!(results.len(), 1, "expected exactly one gravity well");

        let (_marker, pos) = results[0];
        assert_eq!(
            pos.0,
            Vec2::ZERO,
            "well should default to Position2D(Vec2::ZERO) when owner is empty"
        );
    }

    // ── Spawned well entity has CleanupOnNodeExit ───────────────

    #[test]
    fn fire_spawns_well_with_cleanup_on_node_exit() {
        use crate::shared::CleanupOnNodeExit;

        let mut world = World::new();
        let entity = world.spawn(Position2D(Vec2::new(50.0, 75.0))).id();

        fire(entity, 100.0, 5.0, 80.0, 3, "", &mut world);

        let mut query = world.query_filtered::<Entity, With<GravityWellMarker>>();
        let well = query.iter(&world).next().expect("well should exist");

        assert!(
            world.get::<CleanupOnNodeExit>(well).is_some(),
            "spawned gravity well should have CleanupOnNodeExit"
        );
    }

    #[test]
    fn fire_multiple_wells_all_have_cleanup_on_node_exit() {
        use crate::shared::CleanupOnNodeExit;

        let mut world = World::new();
        let entity = world.spawn(Position2D(Vec2::new(10.0, 20.0))).id();

        // Spawn 3 wells with max=5 so none get despawned.
        fire(entity, 100.0, 5.0, 80.0, 5, "", &mut world);
        fire(entity, 200.0, 3.0, 60.0, 5, "", &mut world);
        fire(entity, 300.0, 4.0, 70.0, 5, "", &mut world);

        let mut query = world.query_filtered::<Entity, With<GravityWellMarker>>();
        let wells: Vec<Entity> = query.iter(&world).collect();
        assert_eq!(wells.len(), 3, "expected 3 gravity wells");

        for well in &wells {
            assert!(
                world.get::<CleanupOnNodeExit>(*well).is_some(),
                "ALL spawned gravity wells should have CleanupOnNodeExit"
            );
        }
    }

    // ── apply_gravity_pull uses Position2D ──────────────────────

    #[test]
    fn apply_gravity_pull_uses_position2d_for_bolt_distance() {
        let mut app = test_app();
        enter_playing(&mut app);

        // Gravity well at Position2D origin with deliberately different Transform.
        app.world_mut().spawn((
            GravityWellMarker,
            GravityWellConfig {
                strength: 500.0,
                radius: 200.0,
                remaining: 10.0,
                owner: Entity::PLACEHOLDER,
            },
            Position2D(Vec2::new(0.0, 0.0)),
            Transform::from_xyz(999.0, 999.0, 0.0),
        ));

        // Bolt at Position2D (100, 0) with deliberately different Transform.
        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::ZERO),
                Position2D(Vec2::new(100.0, 0.0)),
                Transform::from_xyz(999.0, 999.0, 0.0),
            ))
            .id();

        app.update();

        let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
        // Bolt should be pulled toward (0,0) via Position2D, not toward Transform positions.
        // Direction from bolt (100,0) to well (0,0) = (-1, 0), so velocity.x should be negative.
        assert!(
            velocity.x < 0.0,
            "bolt should be pulled in -x direction toward well at Position2D (0,0), got velocity.x = {}",
            velocity.x
        );
    }

    #[test]
    fn apply_gravity_pull_uses_position2d_for_well_position_not_transform() {
        let mut app = test_app();
        enter_playing(&mut app);

        // Well at Position2D (0,0) but Transform at (500, 500). If the system reads
        // Transform, the bolt at (100, 0) would be pulled toward (500, 500) instead of (0,0).
        app.world_mut().spawn((
            GravityWellMarker,
            GravityWellConfig {
                strength: 500.0,
                radius: 200.0,
                remaining: 10.0,
                owner: Entity::PLACEHOLDER,
            },
            Position2D(Vec2::new(0.0, 0.0)),
            Transform::from_xyz(500.0, 500.0, 0.0),
        ));

        // Bolt at Position2D (100, 0).
        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::ZERO),
                Position2D(Vec2::new(100.0, 0.0)),
                Transform::from_xyz(100.0, 0.0, 0.0),
            ))
            .id();

        app.update();

        let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
        // If using Position2D: delta = (0,0) - (100,0) = (-100, 0) → bolt pulled in -x.
        assert!(
            velocity.x < 0.0,
            "bolt should be pulled toward well at Position2D (0,0), not Transform (500,500). Got velocity.x = {}",
            velocity.x
        );
    }

    #[test]
    fn apply_gravity_pull_uses_position2d_radius_check_not_transform() {
        let mut app = test_app();
        enter_playing(&mut app);

        // Well at Position2D (200, 0) with small radius 50. Transform at (30, 0).
        // By Position2D, bolt at (0,0) is 200 units away (outside radius 50).
        // By Transform, bolt at (0,0) would appear 30 units away (inside radius 50).
        app.world_mut().spawn((
            GravityWellMarker,
            GravityWellConfig {
                strength: 500.0,
                radius: 50.0,
                remaining: 10.0,
                owner: Entity::PLACEHOLDER,
            },
            Position2D(Vec2::new(200.0, 0.0)),
            Transform::from_xyz(30.0, 0.0, 0.0),
        ));

        // Bolt at Position2D origin.
        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::ZERO),
                Position2D(Vec2::ZERO),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ))
            .id();

        app.update();

        let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
        // Bolt is outside radius by Position2D distance (200 > 50), should NOT be pulled.
        assert!(
            velocity.x.abs() < f32::EPSILON && velocity.y.abs() < f32::EPSILON,
            "bolt should NOT be pulled when outside radius by Position2D distance. Got velocity = ({}, {})",
            velocity.x,
            velocity.y
        );
    }

    #[test]
    fn apply_gravity_pull_does_not_pull_bolt_outside_radius_by_position2d() {
        let mut app = test_app();
        enter_playing(&mut app);

        // Well at Position2D origin, radius 50. Transform at (0,0) matches.
        // Bolt at Position2D (100, 0) = distance 100 > radius 50.
        // Transform at (10, 0) would be within radius if the system reads Transform.
        app.world_mut().spawn((
            GravityWellMarker,
            GravityWellConfig {
                strength: 500.0,
                radius: 50.0,
                remaining: 10.0,
                owner: Entity::PLACEHOLDER,
            },
            Position2D(Vec2::ZERO),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::ZERO),
                Position2D(Vec2::new(100.0, 0.0)),
                Transform::from_xyz(10.0, 0.0, 0.0),
            ))
            .id();

        app.update();

        let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
        // By Position2D: distance 100 > radius 50 → no pull.
        // By Transform: distance 10 < radius 50 → would incorrectly pull.
        assert!(
            velocity.x.abs() < f32::EPSILON && velocity.y.abs() < f32::EPSILON,
            "bolt at Position2D distance 100 should NOT be pulled (radius 50). Got velocity = ({}, {})",
            velocity.x,
            velocity.y
        );
    }

    #[test]
    fn apply_gravity_pull_pulls_bolt_at_exact_radius_boundary() {
        let mut app = test_app();
        enter_playing(&mut app);

        // Well at Position2D origin, radius 50.
        // Bolt at Position2D (50, 0) = distance exactly 50 (inside).
        // Transform at (999, 0) — if system reads Transform, distance 999 > 50, would NOT pull.
        app.world_mut().spawn((
            GravityWellMarker,
            GravityWellConfig {
                strength: 500.0,
                radius: 50.0,
                remaining: 10.0,
                owner: Entity::PLACEHOLDER,
            },
            Position2D(Vec2::ZERO),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::ZERO),
                Position2D(Vec2::new(50.0, 0.0)),
                Transform::from_xyz(999.0, 0.0, 0.0),
            ))
            .id();

        app.update();

        let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
        // By Position2D: distance 50.0 <= radius 50.0 → should pull in -x direction.
        // By Transform: distance 999.0 > radius 50.0 → would not pull.
        assert!(
            velocity.x < 0.0,
            "bolt at exact radius boundary by Position2D (50.0) should be pulled toward well. Got velocity.x = {}",
            velocity.x
        );
    }

    #[test]
    fn apply_gravity_pull_no_pull_when_bolt_at_same_position2d_as_well() {
        let mut app = test_app();
        enter_playing(&mut app);

        // Well and bolt both at Position2D origin — distance = 0, no pull.
        // Transform values differ to ensure the system reads Position2D.
        app.world_mut().spawn((
            GravityWellMarker,
            GravityWellConfig {
                strength: 500.0,
                radius: 200.0,
                remaining: 10.0,
                owner: Entity::PLACEHOLDER,
            },
            Position2D(Vec2::ZERO),
            Transform::from_xyz(100.0, 0.0, 0.0),
        ));

        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::ZERO),
                Position2D(Vec2::ZERO),
                Transform::from_xyz(50.0, 0.0, 0.0),
            ))
            .id();

        app.update();

        let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
        // By Position2D: distance = 0, guard prevents pull.
        // By Transform: distance = 50, would incorrectly pull.
        assert!(
            velocity.x.abs() < f32::EPSILON && velocity.y.abs() < f32::EPSILON,
            "bolt at same Position2D as well (distance 0) should not be pulled. Got velocity = ({}, {})",
            velocity.x,
            velocity.y
        );
    }

    #[test]
    fn apply_gravity_pull_skips_well_without_position2d() {
        let mut app = test_app();
        enter_playing(&mut app);

        // Well with NO Position2D — only Transform. The query should not match this well.
        app.world_mut().spawn((
            GravityWellMarker,
            GravityWellConfig {
                strength: 500.0,
                radius: 200.0,
                remaining: 10.0,
                owner: Entity::PLACEHOLDER,
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::ZERO),
                Position2D(Vec2::new(10.0, 0.0)),
                Transform::from_xyz(10.0, 0.0, 0.0),
            ))
            .id();

        app.update();

        let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
        // Well lacks Position2D, so it should not appear in the query.
        assert!(
            velocity.x.abs() < f32::EPSILON && velocity.y.abs() < f32::EPSILON,
            "well without Position2D should not affect bolt. Got velocity = ({}, {})",
            velocity.x,
            velocity.y
        );
    }

    #[test]
    fn apply_gravity_pull_only_well_with_position2d_affects_bolt() {
        let mut app = test_app();
        enter_playing(&mut app);

        // Well A: has Position2D at (0, 0), radius 200 — should affect bolt.
        app.world_mut().spawn((
            GravityWellMarker,
            GravityWellConfig {
                strength: 500.0,
                radius: 200.0,
                remaining: 10.0,
                owner: Entity::PLACEHOLDER,
            },
            Position2D(Vec2::ZERO),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        // Well B: NO Position2D, only Transform — should be skipped.
        app.world_mut().spawn((
            GravityWellMarker,
            GravityWellConfig {
                strength: 500.0,
                radius: 200.0,
                remaining: 10.0,
                owner: Entity::PLACEHOLDER,
            },
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        // Bolt at Position2D (100, 0) — within radius of well A.
        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::ZERO),
                Position2D(Vec2::new(100.0, 0.0)),
                Transform::from_xyz(100.0, 0.0, 0.0),
            ))
            .id();

        app.update();

        let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
        // Only well A (with Position2D) should pull the bolt.
        // The bolt should be pulled in -x direction toward well A at (0,0).
        assert!(
            velocity.x < 0.0,
            "bolt should be pulled by well with Position2D. Got velocity.x = {}",
            velocity.x
        );
    }

    // ── Regression: apply_gravity_pull must run before speed clamp ───

    /// Regression: `apply_gravity_pull` adds velocity after speed clamp, allowing
    /// bolt speed to exceed `BoltMaxSpeed`.
    ///
    /// Given: Bolt at max speed (600.0) heading upward, gravity well at (0, 200)
    ///        pulling bolt upward (same direction), strength 5000.0.
    /// When: Both `apply_gravity_pull` and `prepare_bolt_velocity` run in the same
    ///        `FixedUpdate` tick.
    /// Then: Final bolt speed is at most `BoltMaxSpeed` (600.0).
    ///
    /// This test FAILS if `apply_gravity_pull` runs after the speed clamp (the bug).
    /// The fix: add `.before(BoltSystems::PrepareVelocity)` to `register()` so
    /// the speed clamp always catches velocity added by gravity pull.
    ///
    /// Scheduling note: uses production `register()` for gravity well systems.
    /// `prepare_bolt_velocity` is registered FIRST so that without an explicit
    /// `.before()` constraint, Bevy's topological sort may place gravity pull
    /// after the clamp, reproducing the bug.
    #[test]
    fn apply_gravity_pull_is_ordered_before_prepare_velocity() {
        use crate::{
            bolt::{
                BoltSystems,
                components::{Bolt, BoltMaxSpeed, BoltMinSpeed},
                systems::prepare_bolt_velocity,
            },
            breaker::components::{Breaker, MinAngleFromHorizontal},
        };

        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<crate::shared::game_state::GameState>();
        app.add_sub_state::<PlayingState>();

        // Register bolt speed clamping FIRST.
        app.add_systems(
            FixedUpdate,
            prepare_bolt_velocity
                .in_set(BoltSystems::PrepareVelocity)
                .run_if(in_state(PlayingState::Active)),
        );

        // Register gravity well systems via production register() SECOND.
        // Without .before(BoltSystems::PrepareVelocity), apply_gravity_pull
        // may run after the speed clamp.
        register(&mut app);

        // Enter Playing state
        app.world_mut()
            .resource_mut::<NextState<crate::shared::game_state::GameState>>()
            .set(crate::shared::game_state::GameState::Playing);
        app.update();

        // Spawn breaker with MinAngleFromHorizontal (required by prepare_bolt_velocity)
        app.world_mut()
            .spawn((Breaker, MinAngleFromHorizontal(15.0_f32.to_radians())));

        let max_speed = 600.0_f32;

        // Gravity well above bolt, pulling bolt upward (same direction as velocity).
        // Very high strength to ensure measurable pull in one tick.
        app.world_mut().spawn((
            GravityWellMarker,
            GravityWellConfig {
                strength: 5000.0,
                radius: 500.0,
                remaining: 10.0,
                owner: Entity::PLACEHOLDER,
            },
            Position2D(Vec2::new(0.0, 200.0)),
        ));

        // Bolt already at max speed heading upward, positioned at (0,0)
        // within the gravity well radius.
        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::new(0.0, max_speed)),
                BoltMinSpeed(200.0),
                BoltMaxSpeed(max_speed),
                Position2D(Vec2::ZERO),
            ))
            .id();

        // Tick one fixed update
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();

        let velocity = app.world().get::<Velocity2D>(bolt).unwrap();
        let final_speed = velocity.speed();
        assert!(
            final_speed <= max_speed + 1.0,
            "bolt speed ({final_speed:.1}) should not exceed BoltMaxSpeed ({max_speed:.1}) \
             after gravity pull + speed clamp — apply_gravity_pull must be ordered \
             before BoltSystems::PrepareVelocity"
        );
    }
}
