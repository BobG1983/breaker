use bevy::prelude::*;
use breaker::{
    breaker::components::BaseWidth,
    effect_v3::{effects::SizeBoostConfig, stacking::EffectStack},
    shared::PlayfieldConfig,
};
use rantzsoft_spatial2d::components::Position2D;

use crate::{invariants::*, types::InvariantKind};

/// Checks that the tagged breaker's x position stays within `playfield.right() - half_width`.
type BreakerPositionQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Position2D,
        &'static BaseWidth,
        Option<&'static EffectStack<SizeBoostConfig>>,
    ),
    With<ScenarioTagBreaker>,
>;

/// Appends a [`ViolationEntry`] with [`InvariantKind::BreakerPositionClamped`] when the
/// breaker is outside the tight clamping bounds (with 1px tolerance).
pub fn check_breaker_position_clamped(
    breakers: BreakerPositionQuery,
    playfield: Res<PlayfieldConfig>,
    frame: Res<ScenarioFrame>,
    mut log: ResMut<ViolationLog>,
    mut stats: Option<ResMut<ScenarioStats>>,
) {
    if let Some(ref mut s) = stats {
        s.invariant_checks += 1;
    }
    let tolerance = 1.0_f32;
    for (entity, position, width, size_boosts) in &breakers {
        let boost_mult = size_boosts.map_or(1.0, EffectStack::aggregate);
        let effective_half_width = width.half_width() * boost_mult;
        let max_x = playfield.right() - effective_half_width;
        let min_x = playfield.left() + effective_half_width;
        let x = position.0.x;
        if x > max_x + tolerance || x < min_x - tolerance {
            log.0.push(ViolationEntry {
                frame: frame.0,
                invariant: InvariantKind::BreakerPositionClamped,
                entity: Some(entity),
                message: format!(
                    "BreakerPositionClamped FAIL frame={} entity={entity:?} x={x:.1} bounds=[{min_x:.1}, {max_x:.1}] effective_hw={effective_half_width:.1}",
                    frame.0,
                ),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use ordered_float::OrderedFloat;

    fn size_stack(values: &[f32]) -> EffectStack<SizeBoostConfig> {
        let mut stack = EffectStack::default();
        for &v in values {
            stack.push(
                String::new(),
                SizeBoostConfig {
                    multiplier: OrderedFloat(v),
                },
            );
        }
        stack
    }

    use super::*;

    fn tick(app: &mut App) {
        let timestep = app.world().resource::<Time<Fixed>>().timestep();
        app.world_mut()
            .resource_mut::<Time<Fixed>>()
            .accumulate_overstep(timestep);
        app.update();
    }

    fn test_app_breaker_position_clamped() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(ViolationLog::default())
            .insert_resource(ScenarioFrame::default())
            .insert_resource(PlayfieldConfig {
                width:                800.0,
                height:               700.0,
                background_color_rgb: [0.0, 0.0, 0.0],
                wall_thickness:       180.0,
                zone_fraction:        0.667,
            })
            .add_systems(FixedUpdate, check_breaker_position_clamped);
        app
    }

    /// Breaker at x=1000.0 is well outside `right() - half_width` (400.0 - 60.0 = 340.0).
    /// A [`ViolationEntry`] with [`InvariantKind::BreakerPositionClamped`] must fire.
    #[test]
    fn breaker_position_clamped_fires_when_outside_bounds() {
        let mut app = test_app_breaker_position_clamped();

        // BaseWidth(120.0) → half_width = 60.0; right() = 400.0 → clamped max = 340.0
        app.world_mut().spawn((
            ScenarioTagBreaker,
            Position2D(Vec2::new(1000.0, -250.0)),
            BaseWidth(120.0),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected exactly one BreakerPositionClamped violation, got {}",
            log.0.len()
        );
        assert_eq!(log.0[0].invariant, InvariantKind::BreakerPositionClamped);
    }

    /// Breaker at x=0.0 is well within bounds. No violation should fire.
    #[test]
    fn breaker_position_clamped_does_not_fire_when_within_bounds() {
        let mut app = test_app_breaker_position_clamped();

        app.world_mut().spawn((
            ScenarioTagBreaker,
            Position2D(Vec2::new(0.0, -250.0)),
            BaseWidth(120.0),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation for breaker at x=0.0"
        );
    }

    /// Breaker at x = 340.0 (exactly `right() - half_width = 400.0 - 60.0`)
    /// is within tolerance. No violation should fire.
    #[test]
    fn breaker_position_clamped_does_not_fire_at_exact_boundary() {
        let mut app = test_app_breaker_position_clamped();

        // Exact boundary: right() - half_width = 400.0 - 60.0 = 340.0
        app.world_mut().spawn((
            ScenarioTagBreaker,
            Position2D(Vec2::new(340.0, -250.0)),
            BaseWidth(120.0),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation when breaker is exactly at clamped boundary (340.0)"
        );
    }

    // ── Tests for ActiveSizeBoosts support ──────────────────────────────

    /// Boosted breaker (2x) at x=270.0.
    /// Effective `half_width` = 60.0 * 2.0 = 120.0. Effective `max_x` = 400.0 - 120.0 = 280.0.
    /// Position 270.0 < 280.0 + 1.0 tolerance = 281.0 → no violation.
    #[test]
    fn boosted_breaker_within_effective_bounds_no_violation() {
        let mut app = test_app_breaker_position_clamped();

        app.world_mut().spawn((
            ScenarioTagBreaker,
            Position2D(Vec2::new(270.0, -250.0)),
            BaseWidth(120.0),
            size_stack(&[2.0]),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation for boosted breaker at x=270.0 (within effective bounds 280.0)"
        );
    }

    /// Edge case for behavior 1: boosted breaker at exactly the effective boundary (280.0).
    /// Effective `half_width` = 120.0, `max_x` = 280.0. Position == `max_x` → within tolerance → no violation.
    #[test]
    fn boosted_breaker_at_exact_effective_boundary_no_violation() {
        let mut app = test_app_breaker_position_clamped();

        app.world_mut().spawn((
            ScenarioTagBreaker,
            Position2D(Vec2::new(280.0, -250.0)),
            BaseWidth(120.0),
            size_stack(&[2.0]),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation for boosted breaker at exactly effective boundary (280.0)"
        );
    }

    /// Boosted breaker (2x) at x=300.0 is outside effective bounds.
    /// Effective `half_width` = 60.0 * 2.0 = 120.0. Effective `max_x` = 400.0 - 120.0 = 280.0.
    /// Position 300.0 > 280.0 + 1.0 tolerance = 281.0 → violation fires.
    #[test]
    fn boosted_breaker_outside_effective_bounds_fires_violation() {
        let mut app = test_app_breaker_position_clamped();

        app.world_mut().spawn((
            ScenarioTagBreaker,
            Position2D(Vec2::new(300.0, -250.0)),
            BaseWidth(120.0),
            size_stack(&[2.0]),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected exactly one BreakerPositionClamped violation for boosted breaker at x=300.0"
        );
        assert_eq!(log.0[0].invariant, InvariantKind::BreakerPositionClamped);
    }

    /// Edge case for behavior 2: position at 281.5 (just past tolerance).
    /// Effective `max_x` = 280.0 + 1.0 tolerance = 281.0. Position 281.5 > 281.0 → violation.
    #[test]
    fn boosted_breaker_just_past_tolerance_fires_violation() {
        let mut app = test_app_breaker_position_clamped();

        app.world_mut().spawn((
            ScenarioTagBreaker,
            Position2D(Vec2::new(281.5, -250.0)),
            BaseWidth(120.0),
            size_stack(&[2.0]),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected violation for boosted breaker at x=281.5 (past effective bounds + tolerance)"
        );
        assert_eq!(log.0[0].invariant, InvariantKind::BreakerPositionClamped);
    }

    /// Boosted breaker at x=280.5 (0.5 past effective `max_x`=280.0, within 1.0 tolerance).
    /// 280.5 <= 280.0 + 1.0 → no violation.
    #[test]
    fn boosted_breaker_within_tolerance_of_effective_boundary_no_violation() {
        let mut app = test_app_breaker_position_clamped();

        app.world_mut().spawn((
            ScenarioTagBreaker,
            Position2D(Vec2::new(280.5, -250.0)),
            BaseWidth(120.0),
            size_stack(&[2.0]),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation for boosted breaker at x=280.5 (within tolerance of effective boundary 280.0)"
        );
    }

    /// Edge case for behavior 3: position at exactly 281.0 (boundary + tolerance exactly).
    /// The check uses `>` not `>=`, so 281.0 == 280.0 + 1.0 → NOT greater → no violation.
    #[test]
    fn boosted_breaker_at_exact_tolerance_edge_no_violation() {
        let mut app = test_app_breaker_position_clamped();

        app.world_mut().spawn((
            ScenarioTagBreaker,
            Position2D(Vec2::new(281.0, -250.0)),
            BaseWidth(120.0),
            size_stack(&[2.0]),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation at exactly effective boundary + tolerance (281.0); check uses > not >="
        );
    }

    /// Multiple stacked boosts [1.5, 2.0] compose via product.
    /// Effective `half_width` = 60.0 * (1.5 * 2.0) = 180.0. Effective `max_x` = 400.0 - 180.0 = 220.0.
    /// Position 250.0 > 220.0 + 1.0 = 221.0 → violation fires.
    #[test]
    fn stacked_boosts_compose_via_product_fires_violation() {
        let mut app = test_app_breaker_position_clamped();

        app.world_mut().spawn((
            ScenarioTagBreaker,
            Position2D(Vec2::new(250.0, -250.0)),
            BaseWidth(120.0),
            size_stack(&[1.5, 2.0]),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected violation for stacked-boost breaker at x=250.0 (effective max_x=220.0)"
        );
        assert_eq!(log.0[0].invariant, InvariantKind::BreakerPositionClamped);
    }

    /// Edge case for behavior 4: stacked boosts but position 215.0 is within effective bounds.
    /// Effective `max_x` = 220.0, 215.0 < 221.0 → no violation.
    #[test]
    fn stacked_boosts_within_effective_bounds_no_violation() {
        let mut app = test_app_breaker_position_clamped();

        app.world_mut().spawn((
            ScenarioTagBreaker,
            Position2D(Vec2::new(215.0, -250.0)),
            BaseWidth(120.0),
            size_stack(&[1.5, 2.0]),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation for stacked-boost breaker at x=215.0 (within effective max_x=220.0)"
        );
    }

    /// Left boundary respects effective `half_width` symmetrically.
    /// Boost 2.0 → effective `half_width` = 120.0 → `min_x` = -400.0 + 120.0 = -280.0.
    /// Position -270.0 > -280.0 - 1.0 = -281.0 → no violation.
    #[test]
    fn boosted_breaker_left_boundary_within_effective_bounds_no_violation() {
        let mut app = test_app_breaker_position_clamped();

        app.world_mut().spawn((
            ScenarioTagBreaker,
            Position2D(Vec2::new(-270.0, -250.0)),
            BaseWidth(120.0),
            size_stack(&[2.0]),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation for boosted breaker at x=-270.0 (within effective left bound -280.0)"
        );
    }

    /// Edge case for behavior 7: left boundary violation at -300.0 with boost 2.0.
    /// Effective `min_x` = -280.0. Position -300.0 < -280.0 - 1.0 = -281.0 → violation.
    #[test]
    fn boosted_breaker_left_boundary_outside_effective_bounds_fires_violation() {
        let mut app = test_app_breaker_position_clamped();

        app.world_mut().spawn((
            ScenarioTagBreaker,
            Position2D(Vec2::new(-300.0, -250.0)),
            BaseWidth(120.0),
            size_stack(&[2.0]),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert_eq!(
            log.0.len(),
            1,
            "expected violation for boosted breaker at x=-300.0 (outside effective left bound -280.0)"
        );
        assert_eq!(log.0[0].invariant, InvariantKind::BreakerPositionClamped);
    }

    /// Empty `ActiveSizeBoosts` vec defaults to multiplier 1.0 (backward compat).
    /// Effective `half_width` = 60.0 * 1.0 = 60.0 → `max_x` = 340.0.
    /// Position 335.0 < 341.0 → no violation.
    #[test]
    fn empty_active_size_boosts_defaults_to_multiplier_one() {
        let mut app = test_app_breaker_position_clamped();

        app.world_mut().spawn((
            ScenarioTagBreaker,
            Position2D(Vec2::new(335.0, -250.0)),
            BaseWidth(120.0),
            size_stack(&[]),
        ));

        tick(&mut app);

        let log = app.world().resource::<ViolationLog>();
        assert!(
            log.0.is_empty(),
            "expected no violation with empty ActiveSizeBoosts (multiplier defaults to 1.0)"
        );
    }
}
