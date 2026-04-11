//! Shockwave runtime components.

use std::collections::HashSet;

use bevy::prelude::*;

/// Marker component identifying an entity as a shockwave source.
#[derive(Component, Debug, Clone)]
pub struct ShockwaveSource;

/// Current expanding radius of the shockwave.
#[derive(Component, Debug, Clone)]
pub struct ShockwaveRadius(pub f32);

/// Maximum radius the shockwave can reach before despawning.
#[derive(Component, Debug, Clone)]
pub struct ShockwaveMaxRadius(pub f32);

/// Expansion speed of the shockwave in units per second.
#[derive(Component, Debug, Clone)]
pub struct ShockwaveSpeed(pub f32);

/// Set of entities already damaged by this shockwave (prevents double-hit).
#[derive(Component, Debug, Clone)]
pub struct ShockwaveDamaged(pub HashSet<Entity>);

/// Base damage dealt by the shockwave before multipliers.
#[derive(Component, Debug, Clone)]
pub struct ShockwaveBaseDamage(pub f32);

/// Multiplier applied to shockwave damage (from stacking or upgrades).
#[derive(Component, Debug, Clone)]
pub struct ShockwaveDamageMultiplier(pub f32);
