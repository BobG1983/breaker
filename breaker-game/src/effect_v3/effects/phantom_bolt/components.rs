//! Phantom bolt runtime components.

use bevy::prelude::*;

// PhantomBolt now lives in bolt/components/phantom.rs.
// This re-export keeps all existing `use crate::effect_v3::effects::phantom_bolt::components::PhantomBolt`
// imports compiling without changes during the migration (Wave 1–4). Deleted in Wave 5.
pub use crate::bolt::components::PhantomBolt;

/// Remaining lifetime in seconds before the phantom bolt despawns.
#[derive(Component, Debug, Clone)]
pub struct PhantomLifetime(pub f32);

/// Entity that spawned this phantom bolt (for ownership tracking).
#[derive(Component, Debug, Clone)]
pub struct PhantomOwner(pub Entity);
