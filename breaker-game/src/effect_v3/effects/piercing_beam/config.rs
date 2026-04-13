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
}
