//! `GravityWellConfig` — gravity well pulling bolts.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rantzsoft_spatial2d::components::Position2D;
use rantzsoft_stateflow::CleanupOnExit;
use serde::{Deserialize, Serialize};

use super::components::*;
use crate::{
    effect_v3::{components::EffectSourceChip, traits::Fireable},
    state::types::NodeState,
};

/// Configuration for a point attractor that pulls bolts toward it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GravityWellConfig {
    /// How strongly bolts are pulled toward the well center per tick.
    pub strength: OrderedFloat<f32>,
    /// How long the well exists before despawning.
    pub duration: OrderedFloat<f32>,
    /// How far from the well center bolts are affected.
    pub radius:   OrderedFloat<f32>,
    /// Maximum active wells per owner entity — oldest removed when exceeded.
    pub max:      u32,
}

impl Fireable for GravityWellConfig {
    fn fire(&self, entity: Entity, source: &str, world: &mut World) {
        let pos = world.get::<Position2D>(entity).map_or(Vec2::ZERO, |p| p.0);

        let chip = EffectSourceChip(if source.is_empty() {
            None
        } else {
            Some(source.to_owned())
        });

        // Enforce max active wells per owner — despawn oldest if limit reached.
        let owned: Vec<Entity> = world
            .query_filtered::<(Entity, &GravityWellOwner), With<GravityWellSource>>()
            .iter(world)
            .filter(|(_, owner)| owner.0 == entity)
            .map(|(e, _)| e)
            .collect();

        if owned.len() >= self.max as usize {
            // Despawn oldest (first spawned).
            if let Some(&oldest) = owned.first() {
                world.despawn(oldest);
            }
        }

        world.spawn((
            GravityWellSource,
            GravityWellOwner(entity),
            GravityWellStrength(self.strength.0),
            GravityWellRadius(self.radius.0),
            GravityWellLifetime(self.duration.0),
            Position2D(pos),
            chip,
            CleanupOnExit::<NodeState>::default(),
        ));
    }

    fn register(app: &mut App) {
        use super::systems::*;
        use crate::effect_v3::EffectV3Systems;

        app.add_systems(
            FixedUpdate,
            (tick_gravity_well, despawn_expired_wells)
                .chain()
                .in_set(EffectV3Systems::Tick),
        );
    }
}
