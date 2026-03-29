use bevy::prelude::*;
use breaker::effect::effects::shield::ShieldActive;

use crate::{invariants::*, types::InvariantKind};

/// Checks that [`ShieldActive`] entities never have `charges == 0`.
///
/// `ShieldActive` represents a live shield — when charges hit zero the shield
/// should be removed. A `ShieldActive { charges: 0 }` that lingers means the
/// removal system failed to clean up, which will prevent future bolt-saves from
/// granting correct shield behavior.
///
/// Fires immediately when a zero-charge shield component is detected.
pub fn check_shield_charges_consistent(
    shields: Query<(Entity, &ShieldActive)>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
) {
    for (entity, shield) in &shields {
        if shield.charges == 0 {
            log.0.push(ViolationEntry {
                frame: frame.0,
                invariant: InvariantKind::ShieldChargesConsistent,
                entity: Some(entity),
                message: format!(
                    "ShieldChargesConsistent FAIL frame={} entity={entity:?} \
                    ShieldActive exists with charges=0 (should have been removed)",
                    frame.0,
                ),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn test_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ViolationLog::default())
            .insert_resource(ScenarioFrame::default())
            .add_systems(FixedUpdate, check_shield_charges_consistent);
        app
    }

    #[test]
    fn no_violation_when_no_shield_exists() {
        let mut app = test_app();
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "no violation expected when no ShieldActive entities exist"
        );
    }

    #[test]
    fn no_violation_when_shield_has_positive_charges() {
        let mut app = test_app();
        app.world_mut().spawn(ShieldActive { charges: 3 });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "no violation expected when ShieldActive has charges > 0"
        );
    }

    #[test]
    fn no_violation_when_shield_has_exactly_one_charge() {
        let mut app = test_app();
        app.world_mut().spawn(ShieldActive { charges: 1 });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "no violation expected when ShieldActive has charges == 1 (minimum valid)"
        );
    }

    #[test]
    fn fires_when_shield_has_zero_charges() {
        let mut app = test_app();
        app.world_mut().spawn(ShieldActive { charges: 0 });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected exactly one violation when ShieldActive has charges=0, got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::ShieldChargesConsistent);
    }

    #[test]
    fn fires_per_entity_with_zero_charges() {
        // Two entities with zero charges: two violations.
        let mut app = test_app();
        app.world_mut().spawn(ShieldActive { charges: 0 });
        app.world_mut().spawn(ShieldActive { charges: 0 });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0
                .iter()
                .filter(|v| v.invariant == InvariantKind::ShieldChargesConsistent)
                .count(),
            2,
            "expected one violation per zero-charge shield entity"
        );
    }

    #[test]
    fn mixed_shields_only_fires_for_zero_charge_ones() {
        let mut app = test_app();
        app.world_mut().spawn(ShieldActive { charges: 2 }); // healthy — no violation
        app.world_mut().spawn(ShieldActive { charges: 0 }); // dead — violation
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0
                .iter()
                .filter(|v| v.invariant == InvariantKind::ShieldChargesConsistent)
                .count(),
            1,
            "expected exactly one violation (only the zero-charge shield)"
        );
    }

    #[test]
    fn violation_message_includes_entity_and_frame() {
        let mut app = test_app();
        app.world_mut().spawn(ShieldActive { charges: 0 });
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0[0].message.contains("charges=0"),
            "violation message should mention charges=0, got: {}",
            log.0[0].message
        );
        assert!(
            log.0[0].entity.is_some(),
            "violation entry should carry the entity"
        );
    }
}
