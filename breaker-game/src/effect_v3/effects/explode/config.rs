//! `ExplodeConfig` — fire-and-forget area explosion.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rantzsoft_spatial2d::components::Position2D;
use serde::{Deserialize, Serialize};

use crate::{
    cells::components::Cell,
    effect_v3::traits::Fireable,
    shared::death_pipeline::{DamageDealt, Dead},
};

/// Area explosion dealing flat damage to all cells within range.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExplodeConfig {
    /// Radius of the explosion in world units.
    pub range:  OrderedFloat<f32>,
    /// Flat damage dealt to every cell within range.
    pub damage: OrderedFloat<f32>,
}

impl Fireable for ExplodeConfig {
    fn fire(&self, entity: Entity, source: &str, world: &mut World) {
        // Snapshot position from the source entity.
        let pos = world.get::<Position2D>(entity).map_or(Vec2::ZERO, |p| p.0);

        let source_chip = if source.is_empty() {
            None
        } else {
            Some(source.to_owned())
        };

        // Collect cells within range (flat damage, no boost multiplier).
        let targets: Vec<Entity> = world
            .query_filtered::<(Entity, &Position2D), (With<Cell>, Without<Dead>)>()
            .iter(world)
            .filter(|(_, cell_pos)| pos.distance(cell_pos.0) <= self.range.0)
            .map(|(e, _)| e)
            .collect();

        // Send damage messages.
        for target in targets {
            world.write_message(DamageDealt {
                dealer: Some(entity),
                target,
                amount: self.damage.0,
                source_chip: source_chip.clone(),
                _marker: std::marker::PhantomData::<Cell>,
            });
        }
    }
}
