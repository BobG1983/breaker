//! Breaker definition — RON-deserialized breaker data.

use bevy::prelude::*;
use serde::Deserialize;

use crate::effect::definition::RootEffect;

/// A breaker definition loaded from a RON file.
///
/// Uses `RootEffect` for all behavior bindings. Each top-level effect is
/// wrapped in `On(target, ...)` to specify the target entity.
/// Adding a new breaker = new RON file. Adding a new behavior type =
/// new `Effect` variant + handler.
#[derive(Asset, TypePath, Deserialize, Clone, Debug)]
pub(crate) struct BreakerDefinition {
    /// Display name of the breaker.
    pub name: String,
    /// Optional stat overrides applied on top of `BreakerDefaults`.
    pub stat_overrides: BreakerStatOverrides,
    /// Number of lives, if the breaker uses a life pool.
    pub life_pool: Option<u32>,
    /// All effect chains for this breaker, each scoped to a target entity.
    pub effects: Vec<RootEffect>,
}

/// Optional overrides for `BreakerDefaults` fields.
///
/// Each `Some` field replaces the corresponding base value.
#[derive(Deserialize, Clone, Debug, Default)]
pub(crate) struct BreakerStatOverrides {
    /// Override breaker width.
    pub width: Option<f32>,
    /// Override breaker height.
    pub height: Option<f32>,
    /// Override maximum horizontal speed.
    pub max_speed: Option<f32>,
    /// Override horizontal acceleration.
    pub acceleration: Option<f32>,
    /// Override horizontal deceleration.
    pub deceleration: Option<f32>,
}
