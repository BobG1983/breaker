//! `MirrorConfig` — fire-and-forget bolt duplication.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::effect_v3::traits::Fireable;

/// Duplicates a bolt with a mirrored velocity.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MirrorConfig {
    /// Whether the mirrored bolt copies the source bolt's effect trees.
    pub inherit: bool,
}

impl Fireable for MirrorConfig {
    fn fire(&self, _entity: Entity, _source: &str, _world: &mut World) {
        // TODO: full implementation requires bolt builder integration
        warn!(
            "MirrorProtocol::fire() called but bolt builder is not yet integrated (inherit={})",
            self.inherit,
        );
    }
}
