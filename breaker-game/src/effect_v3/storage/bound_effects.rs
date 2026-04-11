//! `BoundEffects` — permanent effect trees installed on an entity.

use bevy::prelude::*;

use crate::effect_v3::types::Tree;

/// Permanent effect trees installed on an entity.
///
/// Each entry is a `(name, tree)` pair where `name` identifies the chip
/// or definition that installed the tree. Trees re-arm after each trigger match.
#[derive(Component, Clone, Default)]
pub struct BoundEffects(pub Vec<(String, Tree)>);
