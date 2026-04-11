//! `StagedEffects` — one-shot effect trees on an entity.

use bevy::prelude::*;

use crate::effect_v3::types::Tree;

/// One-shot effect trees installed on an entity.
///
/// Each entry is a `(name, tree)` pair. Trees are consumed after one
/// trigger match.
#[derive(Component, Clone, Default)]
pub struct StagedEffects(pub Vec<(String, Tree)>);
