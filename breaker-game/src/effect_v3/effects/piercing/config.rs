//! `PiercingConfig` — additive passive piercing charges.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::effect_v3::traits::{Fireable, PassiveEffect, Reversible};

/// Number of cells the bolt can pass through without bouncing.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PiercingConfig {
    /// Number of piercing charges granted.
    pub charges: u32,
}

impl Fireable for PiercingConfig {
    fn fire(&self, _entity: Entity, _source: &str, _world: &mut World) {
        todo!()
    }
}

impl Reversible for PiercingConfig {
    fn reverse(&self, _entity: Entity, _source: &str, _world: &mut World) {
        todo!()
    }
}

impl PassiveEffect for PiercingConfig {
    fn aggregate(entries: &[(String, Self)]) -> f32 {
        entries
            .iter()
            .map(|(_, c)| {
                #[allow(
                    clippy::cast_precision_loss,
                    reason = "piercing charges are small u32 values"
                )]
                let v = c.charges as f32;
                v
            })
            .sum()
    }
}
