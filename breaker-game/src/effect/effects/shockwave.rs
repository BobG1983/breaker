use bevy::prelude::*;

use crate::shared::playing_state::PlayingState;

/// Marks the entity that spawned this shockwave.
#[derive(Component)]
pub struct ShockwaveSource(pub Entity);

/// Current radius of the expanding shockwave.
#[derive(Component)]
pub struct ShockwaveRadius(pub f32);

/// Maximum radius the shockwave expands to before despawning.
#[derive(Component)]
pub struct ShockwaveMaxRadius(pub f32);

/// Expansion speed of the shockwave in world units per second.
#[derive(Component)]
pub struct ShockwaveSpeed(pub f32);

pub(crate) fn fire(
    entity: Entity,
    base_range: f32,
    range_per_level: f32,
    stacks: u32,
    speed: f32,
    world: &mut World,
) {
    let extra_stacks = u16::try_from(stacks.saturating_sub(1)).unwrap_or(u16::MAX);
    let effective_range = base_range + f32::from(extra_stacks) * range_per_level;

    let position = world
        .get::<Transform>(entity)
        .map_or(Vec3::ZERO, |t| t.translation);

    world.spawn((
        ShockwaveSource(entity),
        ShockwaveRadius(0.0),
        ShockwaveMaxRadius(effective_range),
        ShockwaveSpeed(speed),
        Transform::from_translation(position),
    ));
}

pub(crate) fn reverse(_entity: Entity, world: &mut World) {
    let _ = world;
}

/// Expand shockwave radius by speed * delta time each fixed tick.
fn tick_shockwave(time: Res<Time>, mut query: Query<(&mut ShockwaveRadius, &ShockwaveSpeed)>) {
    let dt = time.delta_secs();
    for (mut radius, speed) in &mut query {
        radius.0 += speed.0 * dt;
    }
}

/// Despawn shockwaves that have reached their maximum radius.
fn despawn_finished_shockwave(
    mut commands: Commands,
    query: Query<(Entity, &ShockwaveRadius, &ShockwaveMaxRadius)>,
) {
    for (entity, radius, max_radius) in &query {
        if radius.0 >= max_radius.0 {
            commands.entity(entity).despawn();
        }
    }
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        (tick_shockwave, despawn_finished_shockwave)
            .chain()
            .run_if(in_state(PlayingState::Active)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── fire tests ──────────────────────────────────────────────────

    #[test]
    fn fire_spawns_shockwave_entity_at_source_position() {
        let mut world = World::new();
        let entity = world.spawn(Transform::from_xyz(100.0, 200.0, 0.0)).id();

        fire(entity, 24.0, 8.0, 1, 50.0, &mut world);

        let mut query = world.query::<(
            &ShockwaveSource,
            &ShockwaveRadius,
            &ShockwaveMaxRadius,
            &ShockwaveSpeed,
            &Transform,
        )>();
        let results: Vec<_> = query.iter(&world).collect();
        assert_eq!(results.len(), 1, "expected exactly one shockwave entity");

        let (source, radius, max_radius, speed, transform) = results[0];
        assert_eq!(source.0, entity);
        assert!(
            (radius.0 - 0.0).abs() < f32::EPSILON,
            "expected radius 0.0, got {}",
            radius.0
        );
        // stacks=1 → effective = 24.0 + (1-1)*8.0 = 24.0
        assert!(
            (max_radius.0 - 24.0).abs() < f32::EPSILON,
            "expected max_radius 24.0, got {}",
            max_radius.0
        );
        assert!(
            (speed.0 - 50.0).abs() < f32::EPSILON,
            "expected speed 50.0, got {}",
            speed.0
        );
        assert!(
            (transform.translation.x - 100.0).abs() < f32::EPSILON,
            "expected x 100.0, got {}",
            transform.translation.x
        );
        assert!(
            (transform.translation.y - 200.0).abs() < f32::EPSILON,
            "expected y 200.0, got {}",
            transform.translation.y
        );
    }

    #[test]
    fn fire_effective_range_scales_with_stacks() {
        let mut world = World::new();
        let entity = world.spawn(Transform::from_xyz(0.0, 0.0, 0.0)).id();

        // stacks=3, base=24, per_level=8 → effective = 24 + (3-1)*8 = 40
        fire(entity, 24.0, 8.0, 3, 50.0, &mut world);

        let mut query = world.query::<&ShockwaveMaxRadius>();
        let max_radius = query.iter(&world).next().unwrap();
        assert!(
            (max_radius.0 - 40.0).abs() < f32::EPSILON,
            "expected max_radius 40.0, got {}",
            max_radius.0
        );
    }

    #[test]
    fn reverse_is_noop_shockwave_entity_remains() {
        let mut world = World::new();
        let entity = world.spawn(Transform::from_xyz(0.0, 0.0, 0.0)).id();

        fire(entity, 24.0, 8.0, 1, 50.0, &mut world);

        // Verify shockwave exists before reverse
        let mut query = world.query::<&ShockwaveSource>();
        assert_eq!(query.iter(&world).count(), 1);

        reverse(entity, &mut world);

        // Shockwave entity should still exist after reverse (no-op)
        assert_eq!(query.iter(&world).count(), 1);
    }

    // ── system tests ────────────────────────────────────────────────

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<crate::shared::game_state::GameState>();
        app.add_sub_state::<PlayingState>();
        app.add_systems(Update, tick_shockwave);
        app.add_systems(Update, despawn_finished_shockwave);
        app
    }

    fn enter_playing(app: &mut App) {
        app.world_mut()
            .resource_mut::<NextState<crate::shared::game_state::GameState>>()
            .set(crate::shared::game_state::GameState::Playing);
        app.update();
    }

    #[test]
    fn tick_shockwave_expands_radius_by_speed_times_dt() {
        let mut app = test_app();
        enter_playing(&mut app);

        let shockwave = app
            .world_mut()
            .spawn((
                ShockwaveRadius(0.0),
                ShockwaveMaxRadius(100.0),
                ShockwaveSpeed(50.0),
            ))
            .id();

        app.update();

        let radius = app.world().get::<ShockwaveRadius>(shockwave).unwrap();
        // After one update tick, radius should have increased by speed * dt.
        // dt is not zero since MinimalPlugins provides Time.
        assert!(
            radius.0 > 0.0,
            "shockwave radius should expand after tick, got {}",
            radius.0
        );
    }

    #[test]
    fn despawn_finished_shockwave_removes_entity_when_radius_ge_max() {
        let mut app = test_app();
        enter_playing(&mut app);

        let shockwave = app
            .world_mut()
            .spawn((
                ShockwaveRadius(100.0),
                ShockwaveMaxRadius(100.0),
                ShockwaveSpeed(50.0),
            ))
            .id();

        app.update();

        // Entity should be despawned because radius >= max_radius
        assert!(
            app.world().get_entity(shockwave).is_err(),
            "shockwave entity should be despawned when radius >= max_radius"
        );
    }
}
