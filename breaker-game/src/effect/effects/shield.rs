use bevy::prelude::*;

use crate::shared::playing_state::PlayingState;

/// Marks an active shield on the owning entity.
#[derive(Component)]
pub struct ShieldActive {
    /// Remaining duration in seconds.
    pub remaining: f32,
    /// Entity that owns this shield.
    pub owner: Entity,
}

pub(crate) fn fire(
    entity: Entity,
    base_duration: f32,
    duration_per_level: f32,
    stacks: u32,
    world: &mut World,
) {
    let extra_stacks = u16::try_from(stacks.saturating_sub(1)).unwrap_or(u16::MAX);
    let effective_duration = base_duration + f32::from(extra_stacks) * duration_per_level;

    // If entity already has a shield, extend remaining time.
    if let Some(mut shield) = world.get_mut::<ShieldActive>(entity) {
        shield.remaining += effective_duration;
    } else {
        world.entity_mut(entity).insert(ShieldActive {
            remaining: effective_duration,
            owner: entity,
        });
    }
}

pub(crate) fn reverse(entity: Entity, world: &mut World) {
    world.entity_mut(entity).remove::<ShieldActive>();
}

/// Tick shield timers and remove expired shields.
fn tick_shield(
    mut commands: Commands,
    time: Res<Time>,
    mut query: Query<(Entity, &mut ShieldActive)>,
) {
    let dt = time.delta_secs();
    for (entity, mut shield) in &mut query {
        shield.remaining -= dt;
        if shield.remaining <= 0.0 {
            commands.entity(entity).remove::<ShieldActive>();
        }
    }
}

pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        tick_shield.run_if(in_state(PlayingState::Active)),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── fire tests ──────────────────────────────────────────────────

    #[test]
    fn fire_inserts_shield_active_with_effective_duration() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        // stacks=1, base=5.0, per_level=2.0 → effective = 5.0 + 0*2.0 = 5.0
        fire(entity, 5.0, 2.0, 1, &mut world);

        let shield = world.get::<ShieldActive>(entity).unwrap();
        assert!(
            (shield.remaining - 5.0).abs() < f32::EPSILON,
            "expected remaining 5.0, got {}",
            shield.remaining
        );
        assert_eq!(shield.owner, entity);
    }

    #[test]
    fn fire_extends_existing_shield_duration() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        // First fire: effective = 5.0
        fire(entity, 5.0, 2.0, 1, &mut world);

        // Second fire: effective = 5.0 + (2-1)*2.0 = 7.0
        fire(entity, 5.0, 2.0, 2, &mut world);

        let shield = world.get::<ShieldActive>(entity).unwrap();
        // 5.0 from first + 7.0 from second = 12.0
        assert!(
            (shield.remaining - 12.0).abs() < f32::EPSILON,
            "expected remaining 12.0, got {}",
            shield.remaining
        );
    }

    #[test]
    fn reverse_removes_shield_active() {
        let mut world = World::new();
        let entity = world.spawn_empty().id();

        fire(entity, 5.0, 2.0, 1, &mut world);
        assert!(world.get::<ShieldActive>(entity).is_some());

        reverse(entity, &mut world);
        assert!(
            world.get::<ShieldActive>(entity).is_none(),
            "shield should be removed after reverse"
        );
    }

    // ── system tests ────────────────────────────────────────────────

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(bevy::state::app::StatesPlugin);
        app.init_state::<crate::shared::game_state::GameState>();
        app.add_sub_state::<PlayingState>();
        app.add_systems(Update, tick_shield);
        app
    }

    fn enter_playing(app: &mut App) {
        app.world_mut()
            .resource_mut::<NextState<crate::shared::game_state::GameState>>()
            .set(crate::shared::game_state::GameState::Playing);
        app.update();
    }

    #[test]
    fn tick_shield_decrements_remaining_and_removes_on_expiry() {
        let mut app = test_app();
        enter_playing(&mut app);

        let entity = app
            .world_mut()
            .spawn(ShieldActive {
                remaining: 0.0,
                owner: Entity::PLACEHOLDER,
            })
            .id();

        // After a tick, remaining should drop to <= 0 and shield gets removed
        app.update();

        assert!(
            app.world().get::<ShieldActive>(entity).is_none(),
            "shield should be removed when remaining <= 0"
        );
    }

    #[test]
    fn tick_shield_decrements_but_keeps_when_time_remains() {
        let mut app = test_app();
        enter_playing(&mut app);

        let entity = app
            .world_mut()
            .spawn(ShieldActive {
                remaining: 999.0,
                owner: Entity::PLACEHOLDER,
            })
            .id();

        app.update();

        let shield = app.world().get::<ShieldActive>(entity).unwrap();
        assert!(
            shield.remaining < 999.0,
            "shield remaining should have decremented"
        );
        assert!(
            shield.remaining > 0.0,
            "shield should still have time remaining"
        );
    }
}
