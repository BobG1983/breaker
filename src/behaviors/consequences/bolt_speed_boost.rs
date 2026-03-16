//! Bolt speed boost — init-time consequence that stamps multiplier components.

use bevy::prelude::*;

use crate::{
    behaviors::definition::{BehaviorBinding, Consequence, Trigger},
    breaker::components::{BumpPerfectMultiplier, BumpWeakMultiplier},
};

/// Applies `BoltSpeedBoost` consequences from behavior bindings as components
/// on the breaker entity.
///
/// Called by `init_archetype` at run start. Maps trigger+consequence pairs to
/// existing bump multiplier components that the bolt domain already reads.
pub fn apply_bolt_speed_boosts(
    commands: &mut Commands,
    entity: Entity,
    behaviors: &[BehaviorBinding],
) {
    for binding in behaviors {
        if let Consequence::BoltSpeedBoost(multiplier) = binding.consequence {
            for trigger in &binding.triggers {
                match trigger {
                    Trigger::PerfectBump => {
                        commands
                            .entity(entity)
                            .insert(BumpPerfectMultiplier(multiplier));
                    }
                    Trigger::EarlyBump | Trigger::LateBump => {
                        commands
                            .entity(entity)
                            .insert(BumpWeakMultiplier(multiplier));
                    }
                    _ => {}
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::behaviors::definition::BehaviorBinding;
    use crate::breaker::components::Breaker;

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app
    }

    #[test]
    fn perfect_bump_stamps_perfect_multiplier() {
        let mut app = test_app();
        let entity = app.world_mut().spawn(Breaker).id();

        let bindings = vec![BehaviorBinding {
            triggers: vec![Trigger::PerfectBump],
            consequence: Consequence::BoltSpeedBoost(1.5),
        }];

        apply_bolt_speed_boosts(&mut app.world_mut().commands(), entity, &bindings);
        app.world_mut().flush();

        let mult = app.world().get::<BumpPerfectMultiplier>(entity).unwrap();
        assert!((mult.0 - 1.5).abs() < f32::EPSILON);
        assert!(app.world().get::<BumpWeakMultiplier>(entity).is_none());
    }

    #[test]
    fn early_late_stamps_weak_multiplier() {
        let mut app = test_app();
        let entity = app.world_mut().spawn(Breaker).id();

        let bindings = vec![BehaviorBinding {
            triggers: vec![Trigger::EarlyBump, Trigger::LateBump],
            consequence: Consequence::BoltSpeedBoost(1.1),
        }];

        apply_bolt_speed_boosts(&mut app.world_mut().commands(), entity, &bindings);
        app.world_mut().flush();

        let mult = app.world().get::<BumpWeakMultiplier>(entity).unwrap();
        assert!((mult.0 - 1.1).abs() < f32::EPSILON);
        assert!(app.world().get::<BumpPerfectMultiplier>(entity).is_none());
    }

    #[test]
    fn aegis_bindings_stamp_both_multipliers() {
        let mut app = test_app();
        let entity = app.world_mut().spawn(Breaker).id();

        let bindings = vec![
            BehaviorBinding {
                triggers: vec![Trigger::PerfectBump],
                consequence: Consequence::BoltSpeedBoost(1.5),
            },
            BehaviorBinding {
                triggers: vec![Trigger::EarlyBump, Trigger::LateBump],
                consequence: Consequence::BoltSpeedBoost(1.1),
            },
        ];

        apply_bolt_speed_boosts(&mut app.world_mut().commands(), entity, &bindings);
        app.world_mut().flush();

        let perfect = app.world().get::<BumpPerfectMultiplier>(entity).unwrap();
        assert!((perfect.0 - 1.5).abs() < f32::EPSILON);

        let weak = app.world().get::<BumpWeakMultiplier>(entity).unwrap();
        assert!((weak.0 - 1.1).abs() < f32::EPSILON);
    }

    #[test]
    fn non_speed_boost_consequences_ignored() {
        let mut app = test_app();
        let entity = app.world_mut().spawn(Breaker).id();

        let bindings = vec![BehaviorBinding {
            triggers: vec![Trigger::BoltLost],
            consequence: Consequence::LoseLife,
        }];

        apply_bolt_speed_boosts(&mut app.world_mut().commands(), entity, &bindings);
        app.world_mut().flush();

        assert!(app.world().get::<BumpPerfectMultiplier>(entity).is_none());
        assert!(app.world().get::<BumpWeakMultiplier>(entity).is_none());
    }
}
