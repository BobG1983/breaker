//! Entropy engine runtime components.

use bevy::prelude::*;
use ordered_float::OrderedFloat;

use crate::effect_v3::types::EffectType;

/// Tracks bump counts toward randomly firing an effect from a weighted pool.
#[derive(Component, Debug, Clone)]
pub struct EntropyCounter {
    /// Current bump count since last trigger.
    pub count: u32,
    /// Maximum number of effects that can fire per node.
    pub max_effects: u32,
    /// Weighted pool of effects to choose from. Each entry is (weight, `effect_type`).
    pub pool: Vec<(OrderedFloat<f32>, Box<EffectType>)>,
}
