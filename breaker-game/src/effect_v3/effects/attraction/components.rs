//! Attraction runtime components.

use bevy::prelude::*;

use crate::effect_v3::types::AttractionType;

/// Collection of active attraction forces applied to the bolt.
/// Not an `EffectStack` — uses custom storage with named entries.
#[derive(Component, Debug, Clone)]
pub struct ActiveAttractions(pub Vec<AttractionEntry>);

/// A single attraction force entry, keyed by source name.
#[derive(Debug, Clone)]
pub struct AttractionEntry {
    /// Identifier for the source of this attraction (for removal by reverse).
    pub source:          String,
    /// Type of attraction behavior.
    pub attraction_type: AttractionType,
    /// Base force magnitude.
    pub force:           f32,
    /// Optional cap on the applied force.
    pub max_force:       Option<f32>,
}
