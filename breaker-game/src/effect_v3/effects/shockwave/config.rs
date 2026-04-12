//! `ShockwaveConfig` — expanding damage shockwave.

use std::collections::HashSet;

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rantzsoft_spatial2d::components::Position2D;
use rantzsoft_stateflow::CleanupOnExit;
use serde::{Deserialize, Serialize};

use super::components::*;
use crate::{
    effect_v3::{
        components::EffectSourceChip, effects::DamageBoostConfig, stacking::EffectStack,
        traits::Fireable,
    },
    state::types::NodeState,
};

/// Configuration for an expanding radial shockwave that damages cells.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShockwaveConfig {
    /// How far the shockwave ring expands before disappearing.
    pub base_range:      OrderedFloat<f32>,
    /// Extra range added per stack beyond the first.
    pub range_per_level: OrderedFloat<f32>,
    /// Current stack count — effective range is `base_range` + `range_per_level` * (stacks - 1).
    pub stacks:          u32,
    /// How fast the ring expands outward in world units per second.
    pub speed:           OrderedFloat<f32>,
}

impl Fireable for ShockwaveConfig {
    fn fire(&self, entity: Entity, source: &str, world: &mut World) {
        // Snapshot position from the source entity
        let pos = world.get::<Position2D>(entity).map_or(Vec2::ZERO, |p| p.0);

        // Snapshot damage multiplier from source entity's active boosts
        let damage_mult = world
            .get::<EffectStack<DamageBoostConfig>>(entity)
            .map_or(1.0, EffectStack::aggregate);

        // Calculate effective max radius from stacking
        let stacks_f32 = self.stacks.saturating_sub(1) as f32;
        let max_radius = self
            .range_per_level
            .0
            .mul_add(stacks_f32, self.base_range.0);

        // Source chip attribution
        let chip = EffectSourceChip(if source.is_empty() {
            None
        } else {
            Some(source.to_owned())
        });

        // Spawn the shockwave entity
        world.spawn((
            ShockwaveSource,
            ShockwaveRadius(0.0),
            ShockwaveMaxRadius(max_radius),
            ShockwaveSpeed(self.speed.0),
            ShockwaveDamaged(HashSet::new()),
            ShockwaveBaseDamage(10.0), // base damage from bolt's base_damage
            ShockwaveDamageMultiplier(damage_mult),
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
            (
                tick_shockwave,
                apply_shockwave_damage,
                despawn_finished_shockwave,
            )
                .chain()
                .in_set(EffectV3Systems::Tick),
        );
    }
}
