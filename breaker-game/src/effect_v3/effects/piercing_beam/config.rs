//! `PiercingBeamConfig` — fire-and-forget piercing beam line.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rantzsoft_spatial2d::components::{Position2D, Velocity2D};
use serde::{Deserialize, Serialize};

use crate::{
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
        let base_damage = 10.0; // base damage from bolt

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
