//! `TetherBeamConfig` — damage-dealing beam between two bolts.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::effect_v3::traits::Fireable;

/// Configuration for a tether beam that links two bolts and damages cells crossing it.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TetherBeamConfig {
    /// Multiplier applied to base damage for cells the beam crosses each tick.
    pub damage_mult: OrderedFloat<f32>,
    /// false = spawn a new bolt and beam to it; true = connect existing bolts.
    pub chain:       bool,
}

impl Fireable for TetherBeamConfig {
    fn fire(&self, _entity: Entity, _source: &str, _world: &mut World) {
        // TODO: full implementation requires bolt builder integration
        warn!(
            "TetherBeam::fire() called but bolt builder is not yet integrated (damage_mult={}, chain={})",
            self.damage_mult, self.chain,
        );
    }

    fn register(app: &mut App) {
        use super::systems::*;
        use crate::effect_v3::EffectV3Systems;

        app.add_systems(
            FixedUpdate,
            (tick_tether_beam, cleanup_tether_beams)
                .chain()
                .in_set(EffectV3Systems::Tick),
        );
    }
}
