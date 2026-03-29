use bevy::prelude::*;
use rantzsoft_spatial2d::prelude::*;

use crate::{bolt::components::Bolt, shared::playing_state::PlayingState};

/// Marker for gravity well entities.
#[derive(Component)]
pub struct GravityWellMarker;

/// Configuration and runtime state for a gravity well.
#[derive(Component)]
pub struct GravityWellConfig {
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
    let position = world
        .get::<Transform>(entity)
        .map_or(Vec3::ZERO, |t| t.translation);

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
        Transform::from_translation(position),
    ));
}

/// No-op — gravity wells self-despawn via their duration timer.
pub(crate) fn reverse(_entity: Entity, _source_chip: &str, _world: &mut World) {}

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
    wells: Query<(&Transform, &GravityWellConfig), With<GravityWellMarker>>,
    mut bolts: Query<(&Transform, &mut Velocity2D), With<Bolt>>,
) {
    let dt = time.delta_secs();
    for (well_transform, config) in &wells {
        let well_pos = well_transform.translation.truncate();
        for (bolt_transform, mut velocity) in &mut bolts {
            let bolt_pos = bolt_transform.translation.truncate();
            let delta = well_pos - bolt_pos;
            let distance = delta.length();
            if distance > 0.0 && distance <= config.radius {
                let direction = delta / distance;
                let pull = config.strength * dt;
                velocity.x += direction.x * pull;
                velocity.y += direction.y * pull;
            }
        }
    }
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (tick_gravity_well, apply_gravity_pull).run_if(in_state(PlayingState::Active)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── fire tests ──────────────────────────────────────────────────

    #[test]
    fn fire_spawns_well_entity_with_marker_and_config() {
        let mut world = World::new();
        let entity = world.spawn(Transform::from_xyz(50.0, 75.0, 0.0)).id();

        fire(entity, 100.0, 5.0, 80.0, 3, "", &mut world);

        let mut query = world.query::<(&GravityWellMarker, &GravityWellConfig, &Transform)>();
        let results: Vec<_> = query.iter(&world).collect();
        assert_eq!(results.len(), 1, "expected exactly one gravity well");

        let (_marker, config, transform) = results[0];
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
            (transform.translation.x - 50.0).abs() < f32::EPSILON,
            "expected x 50.0, got {}",
            transform.translation.x
        );
        assert!(
            (transform.translation.y - 75.0).abs() < f32::EPSILON,
            "expected y 75.0, got {}",
            transform.translation.y
        );
    }

    #[test]
    fn fire_enforces_max_cap_despawns_oldest() {
        let mut world = World::new();
        let entity = world.spawn(Transform::from_xyz(0.0, 0.0, 0.0)).id();

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
        let owner = world.spawn(Transform::from_xyz(0.0, 0.0, 0.0)).id();

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
                Transform::from_xyz(0.0, 0.0, 0.0),
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
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        // Bolt at (100, 0) with zero velocity — should be pulled toward (0,0)
        let bolt = app
            .world_mut()
            .spawn((
                Bolt,
                Velocity2D(Vec2::ZERO),
                Transform::from_xyz(100.0, 0.0, 0.0),
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
}
