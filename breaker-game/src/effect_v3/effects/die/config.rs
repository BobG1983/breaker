//! `DieConfig` — fire-and-forget entity death.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::effect_v3::traits::Fireable;

/// Sends the entity into the death pipeline. Empty struct for trait uniformity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DieConfig {}

impl Fireable for DieConfig {
    fn fire(&self, _entity: Entity, _source: &str, _world: &mut World) {
        todo!()
    }
}
