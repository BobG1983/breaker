use bevy::prelude::*;
use breaker::effect::{EffectiveSizeMultiplier, effects::size_boost::ActiveSizeBoosts};

use crate::{invariants::*, types::InvariantKind};

/// Epsilon for floating-point comparison of size multipliers.
const SIZE_EPSILON: f32 = 1e-4;

/// Checks that [`EffectiveSizeMultiplier`] matches the product of [`ActiveSizeBoosts`].
///
/// `recalculate_size` runs each tick to keep `EffectiveSizeMultiplier` in sync
/// with `ActiveSizeBoosts`. If an ordering bug or missing system run causes them
/// to diverge, entity sizes will be incorrect.
///
/// The check recomputes `product(ActiveSizeBoosts)` and compares to
/// `EffectiveSizeMultiplier.0` within [`SIZE_EPSILON`]. A divergence by more
/// than epsilon is a violation.
///
/// Entities with `EffectiveSizeMultiplier` but no `ActiveSizeBoosts` are not
/// checked -- the invariant only applies when both components exist on the same entity.
pub fn check_size_boost_in_range(
    query: Query<(Entity, &ActiveSizeBoosts, &EffectiveSizeMultiplier)>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
) {
    for (entity, active, effective) in &query {
        let expected = active.multiplier();
        let actual = effective.0;
        if (actual - expected).abs() > SIZE_EPSILON {
            log.0.push(ViolationEntry {
                frame: frame.0,
                invariant: InvariantKind::SizeBoostInRange,
                entity: Some(entity),
                message: format!(
                    "SizeBoostInRange FAIL frame={} entity={entity:?} \
                    effective={actual:.6} expected={expected:.6} \
                    (delta={:.6} > epsilon={SIZE_EPSILON:.6})",
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
            .add_systems(FixedUpdate, check_size_boost_in_range);
        app
    }

    #[test]
    fn no_violation_when_no_entities_exist() {
        let mut app = test_app();
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "no violation expected when no entities with size components exist"
        );
    }

    #[test]
    fn no_violation_when_components_are_consistent_empty_boosts() {
        // Empty boosts -> product = 1.0 -> EffectiveSizeMultiplier should be 1.0
        let mut app = test_app();
        app.world_mut()
            .spawn((ActiveSizeBoosts(vec![]), EffectiveSizeMultiplier(1.0)));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "no violation expected when ActiveSizeBoosts is empty and effective is 1.0"
        );
    }

    #[test]
    fn no_violation_when_single_boost_matches_effective() {
        let mut app = test_app();
        app.world_mut()
            .spawn((ActiveSizeBoosts(vec![1.5]), EffectiveSizeMultiplier(1.5)));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "no violation expected when effective matches 1 boost of 1.5"
        );
    }

    #[test]
    fn no_violation_when_single_boost_is_exactly_one() {
        let mut app = test_app();
        app.world_mut()
            .spawn((ActiveSizeBoosts(vec![1.0]), EffectiveSizeMultiplier(1.0)));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "no violation expected when boost is 1.0 and effective is 1.0"
        );
    }

    #[test]
    fn no_violation_when_multiple_boosts_match_product() {
        // 1.5 * 2.0 = 3.0
        let mut app = test_app();
        app.world_mut().spawn((
            ActiveSizeBoosts(vec![1.5, 2.0]),
            EffectiveSizeMultiplier(3.0),
        ));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "no violation expected when effective matches product of 1.5 * 2.0 = 3.0"
        );
    }

    #[test]
    fn no_violation_when_three_boosts_match_product() {
        // 1.2 * 1.5 * 2.0 = 3.6
        let mut app = test_app();
        app.world_mut().spawn((
            ActiveSizeBoosts(vec![1.2, 1.5, 2.0]),
            EffectiveSizeMultiplier(3.6),
        ));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "no violation expected when effective matches product of 1.2 * 1.5 * 2.0 = 3.6"
        );
    }

    #[test]
    fn fires_when_effective_does_not_match_boosts() {
        // Boosts product = 1.5, but effective = 2.0 (stale/wrong)
        let mut app = test_app();
        app.world_mut()
            .spawn((ActiveSizeBoosts(vec![1.5]), EffectiveSizeMultiplier(2.0)));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected one violation when effective (2.0) diverges from boosts product (1.5)"
        );
        assert_eq!(log.0[0].invariant, InvariantKind::SizeBoostInRange);
    }

    #[test]
    fn fires_when_effective_is_lower_than_expected() {
        // Boosts product = 2.0, but effective = 1.0
        let mut app = test_app();
        app.world_mut()
            .spawn((ActiveSizeBoosts(vec![2.0]), EffectiveSizeMultiplier(1.0)));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected one violation when effective (1.0) is lower than boosts product (2.0)"
        );
        assert_eq!(log.0[0].invariant, InvariantKind::SizeBoostInRange);
    }

    #[test]
    fn fires_per_entity_only_inconsistent_entities_produce_violations() {
        let mut app = test_app();
        // Entity A: consistent
        app.world_mut()
            .spawn((ActiveSizeBoosts(vec![1.5]), EffectiveSizeMultiplier(1.5)));
        // Entity B: inconsistent
        app.world_mut()
            .spawn((ActiveSizeBoosts(vec![2.0]), EffectiveSizeMultiplier(1.0)));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0
                .iter()
                .filter(|v| v.invariant == InvariantKind::SizeBoostInRange)
                .count(),
            1,
            "expected exactly one violation (only the inconsistent entity)"
        );
    }

    #[test]
    fn fires_per_entity_both_inconsistent_produces_two_violations() {
        let mut app = test_app();
        // Entity A: inconsistent
        app.world_mut()
            .spawn((ActiveSizeBoosts(vec![1.5]), EffectiveSizeMultiplier(3.0)));
        // Entity B: inconsistent
        app.world_mut()
            .spawn((ActiveSizeBoosts(vec![2.0]), EffectiveSizeMultiplier(1.0)));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0
                .iter()
                .filter(|v| v.invariant == InvariantKind::SizeBoostInRange)
                .count(),
            2,
            "expected exactly two violations when both entities are inconsistent"
        );
    }

    #[test]
    fn no_violation_within_epsilon() {
        // Floating-point rounding: effective is just barely within epsilon
        let mut app = test_app();
        let boosts = vec![1.1_f32, 1.2_f32];
        let exact_product: f32 = boosts.iter().product();
        let near_product = SIZE_EPSILON.mul_add(0.5, exact_product); // within epsilon
        app.world_mut().spawn((
            ActiveSizeBoosts(boosts),
            EffectiveSizeMultiplier(near_product),
        ));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "no violation expected when effective is within epsilon of product"
        );
    }

    #[test]
    fn no_violation_when_delta_exactly_equals_epsilon() {
        // delta == SIZE_EPSILON should NOT fire (use > not >=).
        // Use zero-product boosts so delta = (eps - 0.0) = eps exactly,
        // avoiding any fp cancellation from non-zero bases.
        let mut app = test_app();
        let boosts = vec![0.0_f32];
        app.world_mut().spawn((
            ActiveSizeBoosts(boosts),
            EffectiveSizeMultiplier(SIZE_EPSILON),
        ));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "no violation expected when delta is exactly epsilon (use > not >=)"
        );
    }

    #[test]
    fn violation_message_includes_effective_and_expected() {
        let mut app = test_app();
        let entity = app
            .world_mut()
            .spawn((ActiveSizeBoosts(vec![1.5]), EffectiveSizeMultiplier(2.0)))
            .id();
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert_eq!(log.0.len(), 1, "expected one violation");
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
        assert_eq!(
            log.0[0].entity,
            Some(entity),
            "violation entry should carry the entity"
        );
    }

    #[test]
    fn entity_without_active_size_boosts_is_skipped() {
        // Only EffectiveSizeMultiplier -- no ActiveSizeBoosts -- not checked
        let mut app = test_app();
        app.world_mut().spawn(EffectiveSizeMultiplier(99.0));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "entity with only EffectiveSizeMultiplier (no ActiveSizeBoosts) should not be checked"
        );
    }

    #[test]
    fn entity_without_effective_size_multiplier_is_skipped() {
        // Only ActiveSizeBoosts -- no EffectiveSizeMultiplier -- not checked
        let mut app = test_app();
        app.world_mut().spawn(ActiveSizeBoosts(vec![2.0, 3.0]));
        tick(&mut app);
        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "entity with only ActiveSizeBoosts (no EffectiveSizeMultiplier) should not be checked"
        );
    }
}
