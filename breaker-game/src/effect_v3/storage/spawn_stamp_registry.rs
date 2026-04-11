//! `SpawnStampRegistry` — tracks Spawn root nodes for applying trees to newly spawned entities.

use bevy::prelude::*;

use crate::effect_v3::types::{EntityKind, Tree};

/// Tracks `Spawn` root nodes so that trees can be applied to newly
/// spawned entities of the matching kind.
///
/// Populated during chip dispatch when a chip's definition includes
/// `Spawn(kind, tree)` root nodes.
#[derive(Resource, Default)]
pub struct SpawnStampRegistry {
    /// Registered spawn stamps: `(entity_kind, name, tree)`.
    pub entries: Vec<(EntityKind, String, Tree)>,
}
