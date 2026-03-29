use bevy::prelude::*;
use breaker::effect::{EffectiveSpeedMultiplier, effects::speed_boost::ActiveSpeedBoosts};

use crate::{invariants::*, types::InvariantKind};

/// Epsilon for floating-point comparison of speed multipliers.
const SPEED_EPSILON: f32 = 1e-4;

/// Checks that [`EffectiveSpeedMultiplier`] matches the product of [`ActiveSpeedBoosts`].
///
/// `recalculate_speed` runs each tick to keep `EffectiveSpeedMultiplier` in sync
/// with `ActiveSpeedBoosts`. If an ordering bug or missing system run causes them
/// to diverge, bolt/breaker speeds will be incorrect.
///
/// The check recomputes `product(ActiveSpeedBoosts)` and compares to
/// `EffectiveSpeedMultiplier.0` within [`SPEED_EPSILON`]. A divergence by more
/// than epsilon is a violation.
///
/// Entities with `EffectiveSpeedMultiplier` but no `ActiveSpeedBoosts` are not
/// checked — the invariant only applies when both components exist on the same entity.
pub fn check_effective_speed_consistent(
    query: Query<(Entity, &ActiveSpeedBoosts, &EffectiveSpeedMultiplier)>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
) {
    for (entity, active, effective) in &query {
        let expected = active.multiplier();
        let actual = effective.0;
        if (actual - expected).abs() > SPEED_EPSILON {
            log.0.push(ViolationEntry {
                frame: frame.0,
                invariant: InvariantKind::EffectiveSpeedConsistent,
                entity: Some(entity),
                message: format!(
                    "EffectiveSpeedConsistent FAIL frame={} entity={entity:?} \
                    effective={actual:.6} expected={expected:.6} \
                    (delta={:.6} > epsilon={SPEED_EPSILON:.6})",
                    frame.0,
                    (actual - expected).abs(),
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
            .add_systems(FixedUpdate, check_effective_speed_consistent);
        app
    }

    #[test]
    fn no_violation_when_no_entities_exist() {
        let mut app = test_app();
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "no violation expected when no entities with speed components exist"
        );
    }

    #[test]
    fn no_violation_when_components_are_consistent_empty_boosts() {
        // Empty boosts → product = 1.0 → EffectiveSpeedMultiplier should be 1.0
        let mut app = test_app();
        app.world_mut()
            .spawn((ActiveSpeedBoosts(vec![]), EffectiveSpeedMultiplier(1.0)));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "no violation expected when ActiveSpeedBoosts is empty and effective is 1.0"
        );
    }

    #[test]
    fn no_violation_when_single_boost_matches_effective() {
        let mut app = test_app();
        app.world_mut()
            .spawn((ActiveSpeedBoosts(vec![1.5]), EffectiveSpeedMultiplier(1.5)));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "no violation expected when effective matches 1 boost of 1.5"
        );
    }

    #[test]
    fn no_violation_when_multiple_boosts_match_product() {
        // 1.5 * 2.0 = 3.0
        let mut app = test_app();
        app.world_mut().spawn((
            ActiveSpeedBoosts(vec![1.5, 2.0]),
            EffectiveSpeedMultiplier(3.0),
        ));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "no violation expected when effective matches product of 1.5 * 2.0 = 3.0"
        );
    }

    #[test]
    fn fires_when_effective_does_not_match_boosts() {
        // Boosts product = 1.5, but effective = 2.0 (stale/wrong)
        let mut app = test_app();
        app.world_mut()
            .spawn((ActiveSpeedBoosts(vec![1.5]), EffectiveSpeedMultiplier(2.0)));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected one violation when effective (2.0) diverges from boosts product (1.5)"
        );
        assert_eq!(log.0[0].invariant, InvariantKind::EffectiveSpeedConsistent);
    }

    #[test]
    fn fires_per_entity_with_inconsistent_speed() {
        let mut app = test_app();
        // Entity 1: consistent
        app.world_mut()
            .spawn((ActiveSpeedBoosts(vec![1.5]), EffectiveSpeedMultiplier(1.5)));
        // Entity 2: inconsistent
        app.world_mut()
            .spawn((ActiveSpeedBoosts(vec![2.0]), EffectiveSpeedMultiplier(1.0)));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0
                .iter()
                .filter(|v| v.invariant == InvariantKind::EffectiveSpeedConsistent)
                .count(),
            1,
            "expected exactly one violation (only the inconsistent entity)"
        );
    }

    #[test]
    fn no_violation_within_epsilon() {
        // Floating-point rounding: effective is just barely within epsilon
        let mut app = test_app();
        let boosts = vec![1.1_f32, 1.2_f32];
        let exact_product: f32 = boosts.iter().product();
        let near_product = SPEED_EPSILON.mul_add(0.5, exact_product); // within epsilon
        app.world_mut().spawn((
            ActiveSpeedBoosts(boosts),
            EffectiveSpeedMultiplier(near_product),
        ));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "no violation expected when effective is within epsilon of product"
        );
    }

    #[test]
    fn violation_message_includes_effective_and_expected() {
        let mut app = test_app();
        app.world_mut()
            .spawn((ActiveSpeedBoosts(vec![1.5]), EffectiveSpeedMultiplier(2.0)));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0[0].message.contains("effective="),
            "violation message should include 'effective=', got: {}",
            log.0[0].message
        );
        assert!(
            log.0[0].message.contains("expected="),
            "violation message should include 'expected=', got: {}",
            log.0[0].message
        );
        assert!(
            log.0[0].entity.is_some(),
            "violation entry should carry the entity"
        );
    }

    #[test]
    fn entity_without_active_speed_boosts_is_skipped() {
        // Only EffectiveSpeedMultiplier — no ActiveSpeedBoosts — not checked
        let mut app = test_app();
        app.world_mut().spawn(EffectiveSpeedMultiplier(99.0));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "entity with only EffectiveSpeedMultiplier (no ActiveSpeedBoosts) should not be checked"
        );
    }
}
