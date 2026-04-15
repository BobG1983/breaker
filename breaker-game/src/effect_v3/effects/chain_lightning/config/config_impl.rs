//! `ChainLightningConfig` — chain lightning arcs between cells.

use std::collections::HashSet;

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rantzsoft_spatial2d::components::Position2D;
use rantzsoft_stateflow::CleanupOnExit;
use serde::{Deserialize, Serialize};

use super::super::components::*;
use crate::{
    bolt::{components::BoltBaseDamage, resources::DEFAULT_BOLT_BASE_DAMAGE},
    effect_v3::{
        components::EffectSourceChip, effects::DamageBoostConfig, stacking::EffectStack,
        traits::Fireable,
    },
    state::types::NodeState,
};

/// Configuration for chain lightning that arcs between nearby cells.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChainLightningConfig {
    /// Number of times the lightning jumps between cells.
    pub arcs:        u32,
    /// Maximum distance each arc can jump to find a new target.
    pub range:       OrderedFloat<f32>,
    /// Multiplier applied to base damage for each arc hit.
    pub damage_mult: OrderedFloat<f32>,
    /// How fast each lightning arc travels between cells in world units per second.
    pub arc_speed:   OrderedFloat<f32>,
}

impl Fireable for ChainLightningConfig {
    fn fire(&self, entity: Entity, source: &str, world: &mut World) {
        let pos = world.get::<Position2D>(entity).map_or(Vec2::ZERO, |p| p.0);
        let base_damage = world
            .get::<BoltBaseDamage>(entity)
            .map_or(DEFAULT_BOLT_BASE_DAMAGE, |d| d.0);
        let damage_boost = world
            .get::<EffectStack<DamageBoostConfig>>(entity)
            .map_or(1.0, EffectStack::aggregate);

        let chip = EffectSourceChip::from_source(source);

        world.spawn((
            ChainLightningChain {
                remaining_jumps: self.arcs,
                damage:          base_damage * self.damage_mult.0 * damage_boost,
                hit_set:         HashSet::new(),
                state:           ChainState::Idle,
                range:           self.range.0,
                arc_speed:       self.arc_speed.0,
                source_pos:      pos,
            },
            chip,
            CleanupOnExit::<NodeState>::default(),
        ));
    }

    fn register(app: &mut App) {
        use super::super::systems::tick_chain_lightning;
        use crate::effect_v3::EffectV3Systems;

        app.add_systems(
            FixedUpdate,
            tick_chain_lightning.in_set(EffectV3Systems::Tick),
        );
    }
}
