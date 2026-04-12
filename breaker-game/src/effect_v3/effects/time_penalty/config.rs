//! `TimePenaltyConfig` — fire-and-forget time subtraction.

use bevy::prelude::*;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::{effect_v3::traits::Fireable, state::run::node::resources::NodeTimer};

/// Subtracts seconds from the node timer.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TimePenaltyConfig {
    /// Number of seconds subtracted from the node timer.
    pub seconds: OrderedFloat<f32>,
}

impl Fireable for TimePenaltyConfig {
    fn fire(&self, _entity: Entity, _source: &str, world: &mut World) {
        if let Some(mut timer) = world.get_resource_mut::<NodeTimer>() {
            timer.remaining = (timer.remaining - self.seconds.0).max(0.0);
        }
    }
}
