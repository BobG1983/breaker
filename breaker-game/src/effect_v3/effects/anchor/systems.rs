//! Anchor systems — tick lock/unlock and movement detection.

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Velocity2D;

use super::components::{AnchorActive, AnchorPlanted, AnchorTimer};
use crate::effect_v3::{effects::piercing::PiercingConfig, stacking::EffectStack};

/// Detects horizontal breaker movement and resets the anchor timer.
///
/// If the breaker is moving horizontally (`velocity.0.x.abs() > f32::EPSILON`),
/// the anchor timer is reset to `plant_delay` and `AnchorPlanted` is removed.
pub fn detect_breaker_movement(
    mut query: Query<(
        Entity,
        &Velocity2D,
        &mut AnchorTimer,
        &AnchorActive,
        Option<&AnchorPlanted>,
    )>,
    mut commands: Commands,
) {
    for (entity, velocity, mut timer, active, planted) in &mut query {
        if velocity.0.x.abs() > f32::EPSILON {
            timer.0 = active.plant_delay;
            if planted.is_some() {
                commands.entity(entity).remove::<AnchorPlanted>();
            }
        }
    }
}

type TickAnchorQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static mut AnchorTimer,
        Option<&'static mut EffectStack<PiercingConfig>>,
    ),
    Without<AnchorPlanted>,
>;

/// Decrements anchor timer and plants the anchor when it reaches zero.
///
/// On plant, also pushes a piercing charge onto `EffectStack<PiercingConfig>`.
pub fn tick_anchor(mut query: TickAnchorQuery, time: Res<Time>, mut commands: Commands) {
    let dt = time.delta_secs();
    for (entity, mut timer, piercing_stack) in &mut query {
        timer.0 -= dt;
        if timer.0 <= 0.0 {
            commands.entity(entity).insert(AnchorPlanted);
            if let Some(mut stack) = piercing_stack {
                stack.push("anchor_piercing".to_owned(), PiercingConfig { charges: 1 });
            } else {
                let mut stack = EffectStack::<PiercingConfig>::default();
                stack.push("anchor_piercing".to_owned(), PiercingConfig { charges: 1 });
                commands.entity(entity).insert(stack);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use ordered_float::OrderedFloat;
    use rantzsoft_spatial2d::components::Velocity2D;

    use super::*;
    use crate::{
        effect_v3::{
            effects::{
                anchor::components::{AnchorActive, AnchorPlanted, AnchorTimer},
                piercing::PiercingConfig,
            },
            stacking::EffectStack,
        },
        shared::test_utils::{TestAppBuilder, tick},
    };

    fn anchor_movement_app() -> App {
        TestAppBuilder::new()
            .with_system(FixedUpdate, detect_breaker_movement)
            .build()
    }

    fn anchor_tick_app() -> App {
        TestAppBuilder::new()
            .with_system(FixedUpdate, tick_anchor)
            .build()
    }

    fn spawn_breaker_with_anchor(
        app: &mut App,
        velocity: Vec2,
        timer_val: f32,
        plant_delay: f32,
        planted: bool,
    ) -> Entity {
        let mut entity = app.world_mut().spawn((
            Velocity2D(velocity),
            AnchorTimer(timer_val),
            AnchorActive {
                bump_force_multiplier: 2.0,
                perfect_window_multiplier: 1.5,
                plant_delay,
            },
        ));
        if planted {
            entity.insert(AnchorPlanted);
        }
        entity.id()
    }

    // ── C10-1: Breaker movement resets anchor timer and removes AnchorPlanted ──

    #[test]
    fn moving_breaker_resets_timer_and_removes_planted() {
        let mut app = anchor_movement_app();

        let entity = spawn_breaker_with_anchor(
            &mut app,
            Vec2::new(100.0, 0.0), // moving horizontally
            0.3,                   // timer partially elapsed
            1.0,                   // plant_delay
            true,                  // planted
        );

        tick(&mut app);

        let timer = app.world().get::<AnchorTimer>(entity).unwrap();
        assert!(
            (timer.0 - 1.0).abs() < f32::EPSILON,
            "timer should be reset to plant_delay (1.0), got {}",
            timer.0,
        );

        assert!(
            app.world().get::<AnchorPlanted>(entity).is_none(),
            "AnchorPlanted should be removed when breaker is moving",
        );
    }

    #[test]
    fn velocity_at_exactly_epsilon_is_not_considered_moving() {
        let mut app = anchor_movement_app();

        let entity = spawn_breaker_with_anchor(
            &mut app,
            Vec2::new(f32::EPSILON, 0.0), // exactly epsilon
            0.3,
            1.0,
            true,
        );

        tick(&mut app);

        let timer = app.world().get::<AnchorTimer>(entity).unwrap();
        assert!(
            (timer.0 - 0.3).abs() < f32::EPSILON,
            "timer should remain 0.3 when velocity.x == EPSILON (not moving), got {}",
            timer.0,
        );

        assert!(
            app.world().get::<AnchorPlanted>(entity).is_some(),
            "AnchorPlanted should remain when velocity.x is exactly EPSILON",
        );
    }

    #[test]
    fn vertical_only_movement_does_not_reset_anchor() {
        let mut app = anchor_movement_app();

        let entity = spawn_breaker_with_anchor(
            &mut app,
            Vec2::new(0.0, 200.0), // only vertical movement
            0.5,
            1.0,
            true,
        );

        tick(&mut app);

        let timer = app.world().get::<AnchorTimer>(entity).unwrap();
        assert!(
            (timer.0 - 0.5).abs() < f32::EPSILON,
            "timer should remain 0.5 for vertical-only movement, got {}",
            timer.0,
        );

        assert!(
            app.world().get::<AnchorPlanted>(entity).is_some(),
            "AnchorPlanted should remain with vertical-only movement",
        );
    }

    // ── C10-2: Breaker at rest does not reset anchor timer ──

    #[test]
    fn stationary_breaker_does_not_reset_timer() {
        let mut app = anchor_movement_app();

        let entity = spawn_breaker_with_anchor(
            &mut app,
            Vec2::new(0.0, 0.0), // at rest
            0.5,
            1.0,
            false,
        );

        tick(&mut app);

        let timer = app.world().get::<AnchorTimer>(entity).unwrap();
        assert!(
            (timer.0 - 0.5).abs() < f32::EPSILON,
            "timer should remain 0.5 for stationary breaker, got {}",
            timer.0,
        );
    }

    #[test]
    fn sub_epsilon_velocity_does_not_reset_timer() {
        let mut app = anchor_movement_app();

        let entity = spawn_breaker_with_anchor(
            &mut app,
            Vec2::new(f32::EPSILON * 0.5, 0.0), // below threshold
            0.3,
            1.0,
            false,
        );

        tick(&mut app);

        let timer = app.world().get::<AnchorTimer>(entity).unwrap();
        assert!(
            (timer.0 - 0.3).abs() < f32::EPSILON,
            "timer should remain 0.3 for sub-epsilon velocity, got {}",
            timer.0,
        );
    }

    // ── C10-3: Moving breaker without AnchorPlanted still resets timer ──

    #[test]
    fn moving_breaker_without_planted_still_resets_timer() {
        let mut app = anchor_movement_app();

        let entity = spawn_breaker_with_anchor(
            &mut app,
            Vec2::new(50.0, 0.0), // moving
            0.3,
            2.0,   // plant_delay
            false, // NOT planted
        );

        tick(&mut app);

        let timer = app.world().get::<AnchorTimer>(entity).unwrap();
        assert!(
            (timer.0 - 2.0).abs() < f32::EPSILON,
            "timer should be reset to plant_delay (2.0), got {}",
            timer.0,
        );

        // Should not panic when AnchorPlanted is already absent
        assert!(
            app.world().get::<AnchorPlanted>(entity).is_none(),
            "AnchorPlanted should still be absent",
        );
    }

    // ── C10-4: Planting inserts AnchorPlanted and pushes piercing to EffectStack ──

    #[test]
    fn planting_inserts_planted_and_piercing_stack() {
        let mut app = anchor_tick_app();

        let entity = app
            .world_mut()
            .spawn((
                AnchorTimer(0.01), // will reach zero within one tick
                AnchorActive {
                    bump_force_multiplier:     2.0,
                    perfect_window_multiplier: 1.5,
                    plant_delay:               1.0,
                },
            ))
            .id();

        tick(&mut app);

        assert!(
            app.world().get::<AnchorPlanted>(entity).is_some(),
            "AnchorPlanted should be inserted when timer reaches zero",
        );

        let stack = app.world().get::<EffectStack<PiercingConfig>>(entity);
        assert!(
            stack.is_some(),
            "EffectStack<PiercingConfig> should be created on plant",
        );
        let stack = stack.unwrap();
        assert_eq!(
            stack.len(),
            1,
            "piercing stack should have 1 entry from anchor, got {}",
            stack.len(),
        );

        // Check the entry has the correct source name
        let entry = stack.iter().next().unwrap();
        assert_eq!(
            entry.0, "anchor_piercing",
            "piercing source should be 'anchor_piercing', got '{}'",
            entry.0,
        );
        assert_eq!(
            entry.1,
            PiercingConfig { charges: 1 },
            "piercing config should have 1 charge",
        );
    }

    #[test]
    fn planting_adds_to_existing_piercing_stack() {
        let mut app = anchor_tick_app();

        // Pre-existing piercing from another source
        let mut stack = EffectStack::<PiercingConfig>::default();
        stack.push("drill_chip".to_owned(), PiercingConfig { charges: 2 });

        let entity = app
            .world_mut()
            .spawn((
                AnchorTimer(0.01),
                AnchorActive {
                    bump_force_multiplier:     2.0,
                    perfect_window_multiplier: 1.5,
                    plant_delay:               1.0,
                },
                stack,
            ))
            .id();

        tick(&mut app);

        let stack = app
            .world()
            .get::<EffectStack<PiercingConfig>>(entity)
            .unwrap();
        assert_eq!(
            stack.len(),
            2,
            "piercing stack should have 2 entries (existing + anchor), got {}",
            stack.len(),
        );
    }

    // ── C10-5: Reversal removes anchor piercing from EffectStack ──

    #[test]
    fn reversal_removes_anchor_piercing_from_stack() {
        use crate::effect_v3::traits::Reversible;

        let mut world = World::new();

        // Set up entity with anchor components and piercing stack
        let mut stack = EffectStack::<PiercingConfig>::default();
        stack.push("anchor_piercing".to_owned(), PiercingConfig { charges: 1 });
        stack.push("drill_chip".to_owned(), PiercingConfig { charges: 2 });

        let entity = world
            .spawn((
                AnchorActive {
                    bump_force_multiplier:     2.0,
                    perfect_window_multiplier: 1.5,
                    plant_delay:               1.0,
                },
                AnchorTimer(0.5),
                AnchorPlanted,
                stack,
            ))
            .id();

        let config = crate::effect_v3::effects::AnchorConfig {
            bump_force_multiplier:     OrderedFloat(2.0),
            perfect_window_multiplier: OrderedFloat(1.5),
            plant_delay:               OrderedFloat(1.0),
        };

        config.reverse(entity, "test", &mut world);

        // AnchorActive, AnchorTimer, AnchorPlanted should be removed
        assert!(
            world.get::<AnchorActive>(entity).is_none(),
            "AnchorActive should be removed by reverse",
        );
        assert!(
            world.get::<AnchorTimer>(entity).is_none(),
            "AnchorTimer should be removed by reverse",
        );
        assert!(
            world.get::<AnchorPlanted>(entity).is_none(),
            "AnchorPlanted should be removed by reverse",
        );

        // Piercing stack should have only the drill_chip entry
        let stack = world.get::<EffectStack<PiercingConfig>>(entity).unwrap();
        assert_eq!(
            stack.len(),
            1,
            "piercing stack should have 1 entry remaining after reversal, got {}",
            stack.len(),
        );
        let entry = stack.iter().next().unwrap();
        assert_eq!(
            entry.0, "drill_chip",
            "remaining entry should be drill_chip, got '{}'",
            entry.0,
        );
    }

    #[test]
    fn reversal_without_piercing_stack_does_not_panic() {
        use crate::effect_v3::traits::Reversible;

        let mut world = World::new();

        let entity = world
            .spawn((
                AnchorActive {
                    bump_force_multiplier:     2.0,
                    perfect_window_multiplier: 1.5,
                    plant_delay:               1.0,
                },
                AnchorTimer(0.5),
                AnchorPlanted,
            ))
            .id();

        let config = crate::effect_v3::effects::AnchorConfig {
            bump_force_multiplier:     OrderedFloat(2.0),
            perfect_window_multiplier: OrderedFloat(1.5),
            plant_delay:               OrderedFloat(1.0),
        };

        // Should not panic
        config.reverse(entity, "test", &mut world);

        assert!(
            world.get::<AnchorActive>(entity).is_none(),
            "AnchorActive should be removed even without piercing stack",
        );
    }
}
