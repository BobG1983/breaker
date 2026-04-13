//! Reset system — clears `NodeTimerThresholdRegistry.fired` set.

use bevy::prelude::*;

use super::resources::NodeTimerThresholdRegistry;

/// Clears the `fired` set in `NodeTimerThresholdRegistry` so thresholds
/// can fire again in the next node.
pub fn reset_threshold_fired(mut registry: ResMut<NodeTimerThresholdRegistry>) {
    registry.fired.clear();
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use ordered_float::OrderedFloat;

    use super::*;
    use crate::{
        effect_v3::triggers::node::resources::NodeTimerThresholdRegistry,
        shared::test_utils::{TestAppBuilder, tick},
    };

    fn reset_test_app() -> App {
        TestAppBuilder::new()
            .with_resource::<NodeTimerThresholdRegistry>()
            .with_system(FixedUpdate, reset_threshold_fired)
            .build()
    }

    // ── B3-5: Reset clears the fired set ──

    #[test]
    fn reset_clears_fired_set() {
        let mut app = reset_test_app();

        // Prepopulate registry with thresholds and fired set
        {
            let mut registry = app.world_mut().resource_mut::<NodeTimerThresholdRegistry>();
            registry.thresholds.push(OrderedFloat(0.5));
            registry.thresholds.push(OrderedFloat(0.75));
            registry.fired.insert(OrderedFloat(0.5));
            registry.fired.insert(OrderedFloat(0.75));
        }

        tick(&mut app);

        let registry = app.world().resource::<NodeTimerThresholdRegistry>();
        assert!(
            registry.fired.is_empty(),
            "fired set should be empty after reset, got {:?}",
            registry.fired,
        );
        // Thresholds should be unchanged
        assert_eq!(
            registry.thresholds.len(),
            2,
            "thresholds should be unchanged after reset, got {:?}",
            registry.thresholds,
        );
    }

    #[test]
    fn reset_on_already_empty_fired_set_is_noop() {
        let mut app = reset_test_app();

        // Prepopulate registry with thresholds but empty fired set
        {
            let mut registry = app.world_mut().resource_mut::<NodeTimerThresholdRegistry>();
            registry.thresholds.push(OrderedFloat(0.5));
        }

        tick(&mut app);

        let registry = app.world().resource::<NodeTimerThresholdRegistry>();
        assert!(
            registry.fired.is_empty(),
            "fired set should remain empty, got {:?}",
            registry.fired,
        );
    }
}
