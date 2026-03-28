use bevy::prelude::*;

use crate::shared::playing_state::PlayingState;

/// Marker for phantom bolt entities.
#[derive(Component)]
pub struct PhantomBoltMarker;

/// Remaining lifespan of a phantom bolt in seconds.
#[derive(Component)]
pub struct PhantomTimer(pub f32);

/// Entity that spawned this phantom bolt.
#[derive(Component)]
pub struct PhantomOwner(pub Entity);

pub(crate) fn fire(entity: Entity, duration: f32, max_active: u32, world: &mut World) {
    // Enforce max active phantoms for this owner — despawn oldest if at cap.
    let mut owned: Vec<Entity> = Vec::new();
    {
        let mut query = world.query::<(Entity, &PhantomOwner)>();
        for (phantom_entity, owner) in query.iter(world) {
            if owner.0 == entity {
                owned.push(phantom_entity);
            }
        }
    }

    while owned.len() >= max_active as usize {
        if let Some(oldest) = owned.first().copied() {
            world.despawn(oldest);
            owned.remove(0);
        }
    }

    let position = world
        .get::<Transform>(entity)
        .map_or(Vec3::ZERO, |t| t.translation);

    world.spawn((
        PhantomBoltMarker,
        PhantomTimer(duration),
        PhantomOwner(entity),
        Transform::from_translation(position),
    ));
}

pub(crate) fn reverse(_entity: Entity, _world: &mut World) {
    // No-op — phantoms have a lifespan timer and self-despawn via tick_phantom.
}

/// Decrement phantom bolt timers and despawn expired phantoms.
fn tick_phantom(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut PhantomTimer), With<PhantomBoltMarker>>,
) {
    let dt = time.delta_secs();
    for (entity, mut timer) in &mut query {
        timer.0 -= dt;
        if timer.0 <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        tick_phantom.run_if(in_state(PlayingState::Active)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── fire tests ──────────────────────────────────────────────────

    #[test]
    fn fire_spawns_phantom_with_marker_timer_and_owner() {
        let mut world = World::new();
        let entity = world.spawn(Transform::from_xyz(30.0, 40.0, 0.0)).id();

        fire(entity, 5.0, 3, &mut world);

        let mut query =
            world.query::<(&PhantomBoltMarker, &PhantomTimer, &PhantomOwner, &Transform)>();
        let results: Vec<_> = query.iter(&world).collect();
        assert_eq!(results.len(), 1, "expected exactly one phantom");

        let (_marker, timer, owner, transform) = results[0];
        assert!(
            (timer.0 - 5.0).abs() < f32::EPSILON,
            "expected timer 5.0, got {}",
            timer.0
        );
        assert_eq!(owner.0, entity);
        assert!(
            (transform.translation.x - 30.0).abs() < f32::EPSILON,
            "expected x 30.0, got {}",
            transform.translation.x
        );
        assert!(
            (transform.translation.y - 40.0).abs() < f32::EPSILON,
            "expected y 40.0, got {}",
            transform.translation.y
        );
    }

    #[test]
    fn fire_enforces_max_active_cap() {
        let mut world = World::new();
        let entity = world.spawn(Transform::from_xyz(0.0, 0.0, 0.0)).id();

        // Spawn 4 phantoms with max_active=2
        fire(entity, 5.0, 2, &mut world);
        fire(entity, 5.0, 2, &mut world);
        fire(entity, 5.0, 2, &mut world);
        fire(entity, 5.0, 2, &mut world);

        let mut query = world.query::<&PhantomBoltMarker>();
        let count = query.iter(&world).count();
        assert_eq!(
            count, 2,
            "should enforce max of 2 active phantoms, got {count}"
        );
    }

    #[test]
    fn reverse_is_noop_phantoms_self_despawn() {
        let mut world = World::new();
        let owner = world.spawn(Transform::from_xyz(0.0, 0.0, 0.0)).id();

        fire(owner, 5.0, 10, &mut world);
        fire(owner, 5.0, 10, &mut world);

        reverse(owner, &mut world);

        // Phantoms should still exist — they self-despawn via tick_phantom
        let mut query = world.query::<&PhantomOwner>();
        let remaining = query.iter(&world).count();
        assert_eq!(
            remaining, 2,
            "reverse is no-op, phantoms persist until timer expires"
        );
    }

    // ── system tests ────────────────────────────────────────────────

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<crate::shared::game_state::GameState>();
        app.add_sub_state::<PlayingState>();
        app.add_systems(Update, tick_phantom);
        app
    }

    fn enter_playing(app: &mut App) {
        app.world_mut()
            .resource_mut::<NextState<crate::shared::game_state::GameState>>()
            .set(crate::shared::game_state::GameState::Playing);
        app.update();
    }

    #[test]
    fn tick_phantom_despawns_expired_phantoms() {
        let mut app = test_app();
        enter_playing(&mut app);

        let phantom = app
            .world_mut()
            .spawn((
                PhantomBoltMarker,
                PhantomTimer(0.0),
                PhantomOwner(Entity::PLACEHOLDER),
                Transform::from_xyz(0.0, 0.0, 0.0),
            ))
            .id();

        app.update();

        assert!(
            app.world().get_entity(phantom).is_err(),
            "expired phantom should be despawned"
        );
    }
}
