//! Ramping damage systems — reset accumulator on node start.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use super::components::RampingDamageAccumulator;

/// Resets all `RampingDamageAccumulator` components to zero at the start of each node.
pub fn reset_ramping_damage(mut query: Query<&mut RampingDamageAccumulator>) {
    for mut acc in &mut query {
        acc.0 = OrderedFloat(0.0);
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use ordered_float::OrderedFloat;

    use super::*;
    use crate::shared::test_utils::{TestAppBuilder, tick};

    fn reset_test_app() -> App {
        TestAppBuilder::new()
            .with_system(FixedUpdate, reset_ramping_damage)
            .build()
    }

    // ── reset_ramping_damage zeroes a single accumulator ──────────────

    #[test]
    fn reset_zeroes_single_accumulator() {
        let mut app = reset_test_app();

        let entity = app
            .world_mut()
            .spawn(RampingDamageAccumulator(OrderedFloat(5.0)))
            .id();

        tick(&mut app);

        let acc = app.world().get::<RampingDamageAccumulator>(entity).unwrap();
        assert_eq!(
            acc.0,
            OrderedFloat(0.0),
            "accumulator should be zeroed after reset, got {:?}",
            acc.0,
        );
    }

    #[test]
    fn reset_on_already_zero_accumulator_remains_zero() {
        let mut app = reset_test_app();

        let entity = app
            .world_mut()
            .spawn(RampingDamageAccumulator(OrderedFloat(0.0)))
            .id();

        tick(&mut app);

        let acc = app.world().get::<RampingDamageAccumulator>(entity).unwrap();
        assert_eq!(
            acc.0,
            OrderedFloat(0.0),
            "already-zero accumulator should remain 0.0",
        );
    }

    // ── reset_ramping_damage zeroes multiple accumulators ──────────────

    #[test]
    fn reset_zeroes_multiple_accumulators() {
        let mut app = reset_test_app();

        let entity_a = app
            .world_mut()
            .spawn(RampingDamageAccumulator(OrderedFloat(3.0)))
            .id();
        let entity_b = app
            .world_mut()
            .spawn(RampingDamageAccumulator(OrderedFloat(7.5)))
            .id();

        tick(&mut app);

        let acc_a = app
            .world()
            .get::<RampingDamageAccumulator>(entity_a)
            .unwrap();
        let acc_b = app
            .world()
            .get::<RampingDamageAccumulator>(entity_b)
            .unwrap();
        assert_eq!(
            acc_a.0,
            OrderedFloat(0.0),
            "entity A accumulator should be zeroed"
        );
        assert_eq!(
            acc_b.0,
            OrderedFloat(0.0),
            "entity B accumulator should be zeroed"
        );
    }

    #[test]
    fn reset_with_zero_entities_does_not_panic() {
        let mut app = reset_test_app();

        // No entities with RampingDamageAccumulator.
        tick(&mut app);

        // Should not panic — just a no-op.
    }

    // ── reset does not remove the component ───────────────────────────

    #[test]
    fn reset_does_not_remove_accumulator_component() {
        let mut app = reset_test_app();

        let entity = app
            .world_mut()
            .spawn(RampingDamageAccumulator(OrderedFloat(2.0)))
            .id();

        tick(&mut app);

        assert!(
            app.world()
                .get::<RampingDamageAccumulator>(entity)
                .is_some(),
            "reset should zero the accumulator, not remove the component",
        );
        assert_eq!(
            app.world()
                .get::<RampingDamageAccumulator>(entity)
                .unwrap()
                .0,
            OrderedFloat(0.0),
        );
    }
}
