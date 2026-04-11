//! Shield effect — timed visible floor wall that protects the breaker.

use bevy::{ecs::world::CommandQueue, prelude::*};

use crate::prelude::*;

/// Marker for the shield wall entity spawned by the Shield effect.
#[derive(Component)]
pub struct ShieldWall;

/// Timer component that counts down the shield wall's remaining lifetime.
///
/// When the inner timer finishes, the shield wall is despawned by
/// `tick_shield_wall_timer`.
#[derive(Component)]
pub struct ShieldWallTimer(pub Timer);

/// Per-reflection time cost stored on the shield wall entity.
///
/// Each bolt reflection off this wall subtracts this amount from `ShieldWallTimer`.
#[derive(Component)]
pub struct ShieldReflectionCost(pub f32);

/// Spawns a visible floor wall at the playfield bottom, or resets the timer
/// if a shield wall already exists.
pub(crate) fn fire(
    _entity: Entity,
    duration: f32,
    reflection_cost: f32,
    _source_chip: &str,
    world: &mut World,
) {
    // Guard: if ShieldWall exists, reset its timer and return early.
    let existing: Vec<Entity> = world
        .query_filtered::<Entity, With<ShieldWall>>()
        .iter(world)
        .collect();
    if let Some(&wall_entity) = existing.first() {
        if let Some(mut timer) = world.get_mut::<ShieldWallTimer>(wall_entity) {
            timer.0 = Timer::from_seconds(duration, TimerMode::Once);
        }
        return;
    }

    let playfield = world.resource::<PlayfieldConfig>().clone();

    // Create visual handles sequentially to avoid double &mut World borrow.
    // (.visible() requires both &mut Assets<Mesh> and &mut Assets<ColorMaterial>
    // simultaneously, which is impossible with exclusive &mut World access.)
    let color = crate::shared::color_from_rgb([0.3, 0.6, 2.0]);
    let mesh = world
        .resource_mut::<Assets<Mesh>>()
        .add(Rectangle::new(1.0, 1.0));
    let material = world
        .resource_mut::<Assets<ColorMaterial>>()
        .add(ColorMaterial::from_color(color));

    let mut queue = CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, world);
        let entity = Wall::builder()
            .floor(&playfield)
            .with_half_thickness(playfield.wall_half_thickness())
            .timed(duration)
            .spawn(&mut commands);
        commands.entity(entity).insert((
            ShieldWall,
            ShieldWallTimer(Timer::from_seconds(duration, TimerMode::Once)),
            ShieldReflectionCost(reflection_cost),
            Mesh2d(mesh),
            MeshMaterial2d(material),
        ));
    }
    queue.apply(world);
}

/// Despawns all `ShieldWall` entities.
pub(crate) fn reverse(_entity: Entity, _source_chip: &str, world: &mut World) {
    let walls: Vec<Entity> = world
        .query_filtered::<Entity, With<ShieldWall>>()
        .iter(world)
        .collect();
    for wall in walls {
        world.despawn(wall);
    }
}

/// Registers the `tick_shield_wall_timer` and `deduct_shield_on_reflection`
/// systems in `FixedUpdate`.
pub(crate) fn register(app: &mut App) {
    use crate::bolt::BoltSystems;
    app.add_systems(
        FixedUpdate,
        (deduct_shield_on_reflection, tick_shield_wall_timer)
            .chain()
            .after(BoltSystems::WallCollision),
    );
}

/// Ticks `ShieldWallTimer` on every `ShieldWall` entity and despawns the wall
/// when the timer finishes.
fn tick_shield_wall_timer(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ShieldWallTimer), With<ShieldWall>>,
) {
    for (entity, mut timer) in &mut query {
        timer.0.tick(time.delta());
        if timer.0.is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// Reads `BoltImpactWall` messages and subtracts `ShieldReflectionCost` from
/// `ShieldWallTimer` when the impacted wall has `ShieldWall`.
pub(crate) fn deduct_shield_on_reflection(
    mut reader: MessageReader<BoltImpactWall>,
    mut query: Query<(&mut ShieldWallTimer, &ShieldReflectionCost), With<ShieldWall>>,
) {
    for msg in reader.read() {
        if let Ok((mut timer, cost)) = query.get_mut(msg.wall) {
            let new_remaining = timer.0.remaining_secs() - cost.0;
            let duration = timer.0.duration();
            if new_remaining <= 0.0 {
                timer.0.set_elapsed(duration);
            } else {
                let elapsed =
                    duration.saturating_sub(std::time::Duration::from_secs_f32(new_remaining));
                timer.0.set_elapsed(elapsed);
            }
        }
    }
}
