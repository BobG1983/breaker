//! `PulseConfig` — periodic shockwave emitter.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use super::super::components::PulseEmitter;
use crate::effect_v3::{
    components::EffectSourceChip,
    traits::{Fireable, Reversible},
};

/// Configuration for periodic pulse shockwave emission.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PulseConfig {
    /// Radius of each pulse shockwave.
    pub base_range:      OrderedFloat<f32>,
    /// Extra range per stack.
    pub range_per_level: OrderedFloat<f32>,
    /// Current stack count.
    pub stacks:          u32,
    /// Expansion speed of each pulse ring.
    pub speed:           OrderedFloat<f32>,
    /// Seconds between each pulse emission.
    pub interval:        OrderedFloat<f32>,
}

impl Fireable for PulseConfig {
    fn fire(&self, entity: Entity, source: &str, world: &mut World) {
        if world.get_entity(entity).is_err() {
            return;
        }
        world.entity_mut(entity).insert(PulseEmitter {
            base_range:      self.base_range.0,
            range_per_level: self.range_per_level.0,
            stacks:          self.stacks,
            speed:           self.speed.0,
            interval:        self.interval.0,
            timer:           self.interval.0,
            source_chip:     EffectSourceChip::from_source(source),
        });
    }

    fn register(app: &mut App) {
        use super::super::systems::{
            apply_pulse_damage, despawn_finished_pulse_ring, tick_pulse, tick_pulse_ring,
        };
        use crate::effect_v3::EffectV3Systems;

        app.add_systems(
            FixedUpdate,
            (
                tick_pulse,
                tick_pulse_ring,
                apply_pulse_damage,
                despawn_finished_pulse_ring,
            )
                .chain()
                .in_set(EffectV3Systems::Tick),
        );
    }
}

impl Reversible for PulseConfig {
    fn reverse(&self, entity: Entity, _source: &str, world: &mut World) {
        if world.get_entity(entity).is_ok() {
            world.entity_mut(entity).remove::<PulseEmitter>();
        }
    }
}
