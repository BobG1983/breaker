//! `PiercingBeamConfig` — fire-and-forget piercing beam line.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};
use serde::{Deserialize, Serialize};

use crate::{
    bolt::{components::BoltBaseDamage, resources::DEFAULT_BOLT_BASE_DAMAGE},
    cells::components::Cell,
    effect_v3::traits::Fireable,
    shared::death_pipeline::{DamageDealt, Dead},
};

/// Fires a beam that damages all cells along a line.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PiercingBeamConfig {
    /// Multiplier applied to base damage for cells hit by the beam.
    pub damage_mult: OrderedFloat<f32>,
    /// Width of the beam rectangle in world units.
    pub width:       OrderedFloat<f32>,
}

impl Fireable for PiercingBeamConfig {
    fn fire(&self, entity: Entity, source: &str, world: &mut World) {
        // Snapshot position and velocity direction from the source entity.
        let pos = world.get::<Position2D>(entity).map_or(Vec2::ZERO, |p| p.0);
        let dir = world
            .get::<Velocity2D>(entity)
            .map_or(Vec2::Y, |v| v.0.normalize_or(Vec2::Y));

        let source_chip = if source.is_empty() {
            None
        } else {
            Some(source.to_owned())
        };

        let half_width = self.width.0 / 2.0;
        // Normal perpendicular to beam direction.
        let normal = Vec2::new(-dir.y, dir.x);
        let base_damage = world
            .get::<BoltBaseDamage>(entity)
            .map_or(DEFAULT_BOLT_BASE_DAMAGE, |d| d.0);

        // Find cells within the beam rectangle — project each cell onto the
        // beam direction and check perpendicular distance.
        let targets: Vec<Entity> = world
            .query_filtered::<(Entity, &Position2D), (With<Cell>, Without<Dead>)>()
            .iter(world)
            .filter(|(_, cell_pos)| {
                let offset = cell_pos.0 - pos;
                let along = offset.dot(dir);
                let perp = offset.dot(normal).abs();
                // Only hit cells ahead of the bolt (along >= 0) and within width.
                along >= 0.0 && perp <= half_width
            })
            .map(|(e, _)| e)
            .collect();

        let damage = base_damage * self.damage_mult.0;
        for target in targets {
            world.write_message(DamageDealt {
                dealer: Some(entity),
                target,
                amount: damage,
                source_chip: source_chip.clone(),
                _marker: std::marker::PhantomData::<Cell>,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use ordered_float::OrderedFloat;
    use rantzsoft_spatial2d::components::{Position2D, Velocity2D};

    use super::*;
    use crate::{
        bolt::{components::BoltBaseDamage, resources::DEFAULT_BOLT_BASE_DAMAGE},
        cells::components::Cell,
        effect_v3::traits::Fireable,
        shared::{
            death_pipeline::DamageDealt,
            test_utils::{MessageCollector, TestAppBuilder},
        },
    };

    fn piercing_test_app() -> App {
        TestAppBuilder::new()
            .with_message_capture::<DamageDealt<Cell>>()
            .build()
    }

    fn make_config() -> PiercingBeamConfig {
        PiercingBeamConfig {
            damage_mult: OrderedFloat(2.0),
            width:       OrderedFloat(20.0),
        }
    }

    // ── C8: PiercingBeam base damage reads BoltBaseDamage from source entity ──

    #[test]
    fn piercing_beam_uses_bolt_base_damage_from_source_entity() {
        let mut app = piercing_test_app();

        let source = app
            .world_mut()
            .spawn((
                BoltBaseDamage(30.0),
                Position2D(Vec2::new(0.0, 0.0)),
                Velocity2D(Vec2::new(0.0, 400.0)),
            ))
            .id();

        // Spawn a cell directly ahead of the bolt.
        app.world_mut()
            .spawn((Cell, Position2D(Vec2::new(0.0, 100.0))));

        make_config().fire(source, "laser", app.world_mut());
        // Run an update cycle so the message collector picks up the written message.
        app.update();

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(msgs.0.len(), 1, "expected 1 DamageDealt<Cell> message");
        let expected_damage = 30.0 * 2.0;
        assert!(
            (msgs.0[0].amount - expected_damage).abs() < f32::EPSILON,
            "piercing beam damage should be 30.0 * 2.0 = {expected_damage}, got {}",
            msgs.0[0].amount,
        );
    }

    #[test]
    fn piercing_beam_zero_bolt_base_damage() {
        let mut app = piercing_test_app();

        let source = app
            .world_mut()
            .spawn((
                BoltBaseDamage(0.0),
                Position2D(Vec2::new(0.0, 0.0)),
                Velocity2D(Vec2::new(0.0, 400.0)),
            ))
            .id();

        app.world_mut()
            .spawn((Cell, Position2D(Vec2::new(0.0, 100.0))));

        make_config().fire(source, "laser", app.world_mut());
        app.update();

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(msgs.0.len(), 1, "expected 1 DamageDealt<Cell> message");
        assert!(
            msgs.0[0].amount.abs() < f32::EPSILON,
            "piercing beam damage should be 0.0 * 2.0 = 0.0, got {}",
            msgs.0[0].amount,
        );
    }

    #[test]
    fn piercing_beam_falls_back_to_default_when_bolt_base_damage_absent() {
        let mut app = piercing_test_app();

        let source = app
            .world_mut()
            .spawn((
                Position2D(Vec2::new(0.0, 0.0)),
                Velocity2D(Vec2::new(0.0, 400.0)),
            ))
            .id();

        app.world_mut()
            .spawn((Cell, Position2D(Vec2::new(0.0, 100.0))));

        make_config().fire(source, "laser", app.world_mut());
        app.update();

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(msgs.0.len(), 1, "expected 1 DamageDealt<Cell> message");
        let expected_damage = DEFAULT_BOLT_BASE_DAMAGE * 2.0;
        assert!(
            (msgs.0[0].amount - expected_damage).abs() < f32::EPSILON,
            "piercing beam damage should fall back to DEFAULT_BOLT_BASE_DAMAGE * 2.0 = {expected_damage}, got {}",
            msgs.0[0].amount,
        );
    }

    // ── Geometry helper ───────────────────────────────────────────────

    fn geometry_config() -> PiercingBeamConfig {
        PiercingBeamConfig {
            damage_mult: OrderedFloat(1.0),
            width:       OrderedFloat(20.0),
        }
    }

    // ── B1: Cell directly ahead is hit ────────────────────────────────

    #[test]
    fn cell_directly_ahead_of_bolt_is_hit() {
        let mut app = piercing_test_app();

        let source = app
            .world_mut()
            .spawn((
                BoltBaseDamage(10.0),
                Position2D(Vec2::new(0.0, 0.0)),
                Velocity2D(Vec2::new(0.0, 400.0)),
            ))
            .id();

        let cell = app
            .world_mut()
            .spawn((Cell, Position2D(Vec2::new(0.0, 200.0))))
            .id();

        geometry_config().fire(source, "beam", app.world_mut());
        app.update();

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(msgs.0.len(), 1, "cell directly ahead should be hit");
        assert_eq!(msgs.0[0].target, cell);
        assert!(
            (msgs.0[0].amount - 10.0).abs() < f32::EPSILON,
            "damage should be 10.0 * 1.0 = 10.0, got {}",
            msgs.0[0].amount,
        );
        assert_eq!(msgs.0[0].dealer, Some(source));
    }

    #[test]
    fn cell_barely_ahead_of_bolt_is_hit() {
        let mut app = piercing_test_app();

        let source = app
            .world_mut()
            .spawn((
                BoltBaseDamage(10.0),
                Position2D(Vec2::new(0.0, 0.0)),
                Velocity2D(Vec2::new(0.0, 400.0)),
            ))
            .id();

        app.world_mut()
            .spawn((Cell, Position2D(Vec2::new(0.0, 1.0))));

        geometry_config().fire(source, "beam", app.world_mut());
        app.update();

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(msgs.0.len(), 1, "cell barely ahead (1 unit) should be hit");
    }

    // ── B1: Cell behind bolt is NOT hit ───────────────────────────────

    #[test]
    fn cell_behind_bolt_is_not_hit() {
        let mut app = piercing_test_app();

        let source = app
            .world_mut()
            .spawn((
                BoltBaseDamage(10.0),
                Position2D(Vec2::new(0.0, 100.0)),
                Velocity2D(Vec2::new(0.0, 400.0)),
            ))
            .id();

        app.world_mut()
            .spawn((Cell, Position2D(Vec2::new(0.0, 50.0))));

        geometry_config().fire(source, "beam", app.world_mut());
        app.update();

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(msgs.0.len(), 0, "cell behind bolt should NOT be hit");
    }

    #[test]
    fn cell_one_unit_behind_bolt_is_not_hit() {
        let mut app = piercing_test_app();

        let source = app
            .world_mut()
            .spawn((
                BoltBaseDamage(10.0),
                Position2D(Vec2::new(0.0, 100.0)),
                Velocity2D(Vec2::new(0.0, 400.0)),
            ))
            .id();

        app.world_mut()
            .spawn((Cell, Position2D(Vec2::new(0.0, 99.0))));

        geometry_config().fire(source, "beam", app.world_mut());
        app.update();

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(msgs.0.len(), 0, "cell 1 unit behind should NOT be hit");
    }

    // ── B1: Cell at bolt position (along == 0) is hit ─────────────────

    #[test]
    fn cell_at_bolt_position_is_hit() {
        let mut app = piercing_test_app();

        let source = app
            .world_mut()
            .spawn((
                BoltBaseDamage(10.0),
                Position2D(Vec2::new(50.0, 50.0)),
                Velocity2D(Vec2::new(0.0, 400.0)),
            ))
            .id();

        let cell = app
            .world_mut()
            .spawn((Cell, Position2D(Vec2::new(50.0, 50.0))))
            .id();

        geometry_config().fire(source, "beam", app.world_mut());
        app.update();

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(
            msgs.0.len(),
            1,
            "cell at exact bolt position (along == 0) should be hit"
        );
        assert_eq!(msgs.0[0].target, cell);
    }

    // ── B2: Width threshold — within half_width ───────────────────────

    #[test]
    fn cell_within_half_width_is_hit() {
        let mut app = piercing_test_app();

        let source = app
            .world_mut()
            .spawn((
                BoltBaseDamage(10.0),
                Position2D(Vec2::new(0.0, 0.0)),
                Velocity2D(Vec2::new(0.0, 400.0)),
            ))
            .id();

        // 9 units to the right, within half_width of 10.
        app.world_mut()
            .spawn((Cell, Position2D(Vec2::new(9.0, 100.0))));

        geometry_config().fire(source, "beam", app.world_mut());
        app.update();

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(
            msgs.0.len(),
            1,
            "cell within half_width (9 < 10) should be hit"
        );
    }

    #[test]
    fn cell_within_half_width_left_side_is_hit() {
        let mut app = piercing_test_app();

        let source = app
            .world_mut()
            .spawn((
                BoltBaseDamage(10.0),
                Position2D(Vec2::new(0.0, 0.0)),
                Velocity2D(Vec2::new(0.0, 400.0)),
            ))
            .id();

        // 9 units to the left — symmetric check.
        app.world_mut()
            .spawn((Cell, Position2D(Vec2::new(-9.0, 100.0))));

        geometry_config().fire(source, "beam", app.world_mut());
        app.update();

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(
            msgs.0.len(),
            1,
            "cell 9 units to the left should also be hit (symmetric)"
        );
    }

    // ── B2: Width threshold — exactly at boundary ─────────────────────

    #[test]
    fn cell_exactly_at_half_width_boundary_is_hit() {
        let mut app = piercing_test_app();

        let source = app
            .world_mut()
            .spawn((
                BoltBaseDamage(10.0),
                Position2D(Vec2::new(0.0, 0.0)),
                Velocity2D(Vec2::new(0.0, 400.0)),
            ))
            .id();

        // Exactly at half_width boundary (10.0).
        app.world_mut()
            .spawn((Cell, Position2D(Vec2::new(10.0, 100.0))));

        geometry_config().fire(source, "beam", app.world_mut());
        app.update();

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(
            msgs.0.len(),
            1,
            "cell exactly at half_width (perp <= half_width) should be hit"
        );
    }

    #[test]
    fn cell_exactly_at_negative_half_width_boundary_is_hit() {
        let mut app = piercing_test_app();

        let source = app
            .world_mut()
            .spawn((
                BoltBaseDamage(10.0),
                Position2D(Vec2::new(0.0, 0.0)),
                Velocity2D(Vec2::new(0.0, 400.0)),
            ))
            .id();

        app.world_mut()
            .spawn((Cell, Position2D(Vec2::new(-10.0, 100.0))));

        geometry_config().fire(source, "beam", app.world_mut());
        app.update();

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(
            msgs.0.len(),
            1,
            "cell at -10.0 (negative side boundary) should also be hit"
        );
    }

    // ── B2: Width threshold — outside half_width ──────────────────────

    #[test]
    fn cell_outside_half_width_is_not_hit() {
        let mut app = piercing_test_app();

        let source = app
            .world_mut()
            .spawn((
                BoltBaseDamage(10.0),
                Position2D(Vec2::new(0.0, 0.0)),
                Velocity2D(Vec2::new(0.0, 400.0)),
            ))
            .id();

        app.world_mut()
            .spawn((Cell, Position2D(Vec2::new(11.0, 100.0))));

        geometry_config().fire(source, "beam", app.world_mut());
        app.update();

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(
            msgs.0.len(),
            0,
            "cell outside half_width (11 > 10) should NOT be hit"
        );
    }

    #[test]
    fn cell_outside_negative_half_width_is_not_hit() {
        let mut app = piercing_test_app();

        let source = app
            .world_mut()
            .spawn((
                BoltBaseDamage(10.0),
                Position2D(Vec2::new(0.0, 0.0)),
                Velocity2D(Vec2::new(0.0, 400.0)),
            ))
            .id();

        app.world_mut()
            .spawn((Cell, Position2D(Vec2::new(-11.0, 100.0))));

        geometry_config().fire(source, "beam", app.world_mut());
        app.update();

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(msgs.0.len(), 0, "cell at -11 should NOT be hit");
    }

    // ── B3: Diagonal beam direction ───────────────────────────────────

    #[test]
    fn beam_fires_along_diagonal_velocity_direction() {
        let mut app = piercing_test_app();

        let source = app
            .world_mut()
            .spawn((
                BoltBaseDamage(10.0),
                Position2D(Vec2::new(0.0, 0.0)),
                Velocity2D(Vec2::new(300.0, 400.0)),
            ))
            .id();

        // Cell exactly along the beam direction at distance 50.
        app.world_mut()
            .spawn((Cell, Position2D(Vec2::new(30.0, 40.0))));

        geometry_config().fire(source, "beam", app.world_mut());
        app.update();

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(msgs.0.len(), 1, "cell along diagonal beam should be hit");
    }

    #[test]
    fn cell_behind_diagonal_beam_is_not_hit() {
        let mut app = piercing_test_app();

        let source = app
            .world_mut()
            .spawn((
                BoltBaseDamage(10.0),
                Position2D(Vec2::new(0.0, 0.0)),
                Velocity2D(Vec2::new(300.0, 400.0)),
            ))
            .id();

        // Cell in the opposite direction.
        app.world_mut()
            .spawn((Cell, Position2D(Vec2::new(-30.0, -40.0))));

        geometry_config().fire(source, "beam", app.world_mut());
        app.update();

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(
            msgs.0.len(),
            0,
            "cell behind diagonal beam should NOT be hit"
        );
    }

    // ── B4: Dead cell filtering ───────────────────────────────────────

    #[test]
    fn dead_cells_are_excluded_from_beam() {
        let mut app = piercing_test_app();

        let source = app
            .world_mut()
            .spawn((
                BoltBaseDamage(10.0),
                Position2D(Vec2::new(0.0, 0.0)),
                Velocity2D(Vec2::new(0.0, 400.0)),
            ))
            .id();

        app.world_mut()
            .spawn((Cell, Position2D(Vec2::new(0.0, 100.0)), Dead));

        geometry_config().fire(source, "beam", app.world_mut());
        app.update();

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(
            msgs.0.len(),
            0,
            "dead cells should be excluded from beam hits"
        );
    }

    #[test]
    fn dead_cell_excluded_alive_cell_hit() {
        let mut app = piercing_test_app();

        let source = app
            .world_mut()
            .spawn((
                BoltBaseDamage(10.0),
                Position2D(Vec2::new(0.0, 0.0)),
                Velocity2D(Vec2::new(0.0, 400.0)),
            ))
            .id();

        // Dead cell ahead.
        app.world_mut()
            .spawn((Cell, Position2D(Vec2::new(0.0, 100.0)), Dead));
        // Alive cell ahead.
        let alive = app
            .world_mut()
            .spawn((Cell, Position2D(Vec2::new(0.0, 200.0))))
            .id();

        geometry_config().fire(source, "beam", app.world_mut());
        app.update();

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(msgs.0.len(), 1, "only alive cell should produce damage");
        assert_eq!(msgs.0[0].target, alive);
    }

    // ── B5: Damage multiplier propagation ─────────────────────────────

    #[test]
    fn damage_is_base_damage_times_damage_mult() {
        let mut app = piercing_test_app();

        let source = app
            .world_mut()
            .spawn((
                BoltBaseDamage(15.0),
                Position2D(Vec2::ZERO),
                Velocity2D(Vec2::new(0.0, 400.0)),
            ))
            .id();

        app.world_mut()
            .spawn((Cell, Position2D(Vec2::new(0.0, 50.0))));

        let config = PiercingBeamConfig {
            damage_mult: OrderedFloat(3.0),
            width:       OrderedFloat(20.0),
        };
        config.fire(source, "beam", app.world_mut());
        app.update();

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(msgs.0.len(), 1);
        assert!(
            (msgs.0[0].amount - 45.0).abs() < f32::EPSILON,
            "damage should be 15.0 * 3.0 = 45.0, got {}",
            msgs.0[0].amount,
        );
    }

    #[test]
    fn damage_with_half_multiplier() {
        let mut app = piercing_test_app();

        let source = app
            .world_mut()
            .spawn((
                BoltBaseDamage(15.0),
                Position2D(Vec2::ZERO),
                Velocity2D(Vec2::new(0.0, 400.0)),
            ))
            .id();

        app.world_mut()
            .spawn((Cell, Position2D(Vec2::new(0.0, 50.0))));

        let config = PiercingBeamConfig {
            damage_mult: OrderedFloat(0.5),
            width:       OrderedFloat(20.0),
        };
        config.fire(source, "beam", app.world_mut());
        app.update();

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(msgs.0.len(), 1);
        assert!(
            (msgs.0[0].amount - 7.5).abs() < f32::EPSILON,
            "damage should be 15.0 * 0.5 = 7.5, got {}",
            msgs.0[0].amount,
        );
    }

    // ── B6: Source chip propagation ────────────────────────────────────

    #[test]
    fn non_empty_source_propagates_as_some_source_chip() {
        let mut app = piercing_test_app();

        let source = app
            .world_mut()
            .spawn((
                BoltBaseDamage(10.0),
                Position2D(Vec2::ZERO),
                Velocity2D(Vec2::new(0.0, 400.0)),
            ))
            .id();

        app.world_mut()
            .spawn((Cell, Position2D(Vec2::new(0.0, 50.0))));

        geometry_config().fire(source, "laser_chip", app.world_mut());
        app.update();

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(msgs.0.len(), 1);
        assert_eq!(
            msgs.0[0].source_chip,
            Some("laser_chip".to_string()),
            "non-empty source should propagate as Some(source_chip)",
        );
    }

    #[test]
    fn empty_source_propagates_as_none_source_chip() {
        let mut app = piercing_test_app();

        let source = app
            .world_mut()
            .spawn((
                BoltBaseDamage(10.0),
                Position2D(Vec2::ZERO),
                Velocity2D(Vec2::new(0.0, 400.0)),
            ))
            .id();

        app.world_mut()
            .spawn((Cell, Position2D(Vec2::new(0.0, 50.0))));

        geometry_config().fire(source, "", app.world_mut());
        app.update();

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(msgs.0.len(), 1);
        assert_eq!(
            msgs.0[0].source_chip, None,
            "empty source string should propagate as None",
        );
    }

    // ── B7: Multiple cells hit ────────────────────────────────────────

    #[test]
    fn multiple_cells_within_beam_all_hit() {
        let mut app = piercing_test_app();

        let source = app
            .world_mut()
            .spawn((
                BoltBaseDamage(10.0),
                Position2D(Vec2::ZERO),
                Velocity2D(Vec2::new(0.0, 400.0)),
            ))
            .id();

        let cell_a = app
            .world_mut()
            .spawn((Cell, Position2D(Vec2::new(0.0, 50.0))))
            .id();
        let cell_b = app
            .world_mut()
            .spawn((Cell, Position2D(Vec2::new(0.0, 150.0))))
            .id();
        let cell_c = app
            .world_mut()
            .spawn((Cell, Position2D(Vec2::new(0.0, 300.0))))
            .id();

        let config = PiercingBeamConfig {
            damage_mult: OrderedFloat(2.0),
            width:       OrderedFloat(20.0),
        };
        config.fire(source, "beam", app.world_mut());
        app.update();

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(msgs.0.len(), 3, "all 3 cells in beam should be hit");

        let targets: std::collections::HashSet<Entity> = msgs.0.iter().map(|m| m.target).collect();
        assert!(targets.contains(&cell_a));
        assert!(targets.contains(&cell_b));
        assert!(targets.contains(&cell_c));

        for msg in &msgs.0 {
            assert!(
                (msg.amount - 20.0).abs() < f32::EPSILON,
                "each message should have amount 10.0 * 2.0 = 20.0, got {}",
                msg.amount,
            );
        }
    }

    #[test]
    fn mixed_inside_outside_dead_only_alive_inside_hit() {
        let mut app = piercing_test_app();

        let source = app
            .world_mut()
            .spawn((
                BoltBaseDamage(10.0),
                Position2D(Vec2::ZERO),
                Velocity2D(Vec2::new(0.0, 400.0)),
            ))
            .id();

        // Inside beam, alive.
        let alive_inside = app
            .world_mut()
            .spawn((Cell, Position2D(Vec2::new(0.0, 50.0))))
            .id();
        // Outside beam (perp > half_width).
        app.world_mut()
            .spawn((Cell, Position2D(Vec2::new(50.0, 100.0))));
        // Dead cell inside beam.
        app.world_mut()
            .spawn((Cell, Position2D(Vec2::new(0.0, 200.0)), Dead));

        let config = PiercingBeamConfig {
            damage_mult: OrderedFloat(2.0),
            width:       OrderedFloat(20.0),
        };
        config.fire(source, "beam", app.world_mut());
        app.update();

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(msgs.0.len(), 1, "only alive cell inside beam should be hit");
        assert_eq!(msgs.0[0].target, alive_inside);
    }

    // ── B8: No cells in range ─────────────────────────────────────────

    #[test]
    fn fire_with_no_cells_emits_nothing_and_does_not_panic() {
        let mut app = piercing_test_app();

        let source = app
            .world_mut()
            .spawn((
                BoltBaseDamage(10.0),
                Position2D(Vec2::ZERO),
                Velocity2D(Vec2::new(0.0, 400.0)),
            ))
            .id();

        // No Cell entities.
        let config = PiercingBeamConfig {
            damage_mult: OrderedFloat(2.0),
            width:       OrderedFloat(20.0),
        };
        config.fire(source, "beam", app.world_mut());
        app.update();

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(msgs.0.len(), 0, "no cells should mean no damage messages");
    }

    // ── B9: Missing source entity components ──────────────────────────

    #[test]
    fn source_without_velocity_defaults_direction_to_y() {
        let mut app = piercing_test_app();

        // No Velocity2D component.
        let source = app
            .world_mut()
            .spawn((BoltBaseDamage(10.0), Position2D(Vec2::ZERO)))
            .id();

        // Cell directly above (along Vec2::Y).
        app.world_mut()
            .spawn((Cell, Position2D(Vec2::new(0.0, 100.0))));

        geometry_config().fire(source, "beam", app.world_mut());
        app.update();

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(
            msgs.0.len(),
            1,
            "should fall back to Vec2::Y and hit cell above"
        );
    }

    #[test]
    fn source_without_velocity_does_not_hit_perpendicular_cell() {
        let mut app = piercing_test_app();

        let source = app
            .world_mut()
            .spawn((BoltBaseDamage(10.0), Position2D(Vec2::ZERO)))
            .id();

        // Cell far to the right — perpendicular to Y direction, outside beam width.
        app.world_mut()
            .spawn((Cell, Position2D(Vec2::new(100.0, 0.0))));

        geometry_config().fire(source, "beam", app.world_mut());
        app.update();

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(
            msgs.0.len(),
            0,
            "cell 100 units perpendicular to Y should not be hit"
        );
    }

    #[test]
    fn source_with_zero_velocity_defaults_direction_to_y() {
        let mut app = piercing_test_app();

        let source = app
            .world_mut()
            .spawn((
                BoltBaseDamage(10.0),
                Position2D(Vec2::ZERO),
                Velocity2D(Vec2::ZERO),
            ))
            .id();

        app.world_mut()
            .spawn((Cell, Position2D(Vec2::new(0.0, 100.0))));

        geometry_config().fire(source, "beam", app.world_mut());
        app.update();

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(
            msgs.0.len(),
            1,
            "zero-magnitude velocity should fall back to Vec2::Y via normalize_or",
        );
    }

    #[test]
    fn source_without_position_defaults_origin_to_zero() {
        let mut app = piercing_test_app();

        // No Position2D component.
        let source = app
            .world_mut()
            .spawn((BoltBaseDamage(10.0), Velocity2D(Vec2::new(0.0, 400.0))))
            .id();

        app.world_mut()
            .spawn((Cell, Position2D(Vec2::new(0.0, 100.0))));

        geometry_config().fire(source, "beam", app.world_mut());
        app.update();

        let msgs = app
            .world()
            .resource::<MessageCollector<DamageDealt<Cell>>>();
        assert_eq!(
            msgs.0.len(),
            1,
            "missing Position2D should default to Vec2::ZERO as origin",
        );
    }
}
