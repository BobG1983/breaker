//! Chip effect components stamped onto bolt and breaker entities.
//!
//! These components are inserted by `dispatch_chip_effects` and read by gameplay
//! systems (physics, bump, spawn) to modify entity behaviour.

use bevy::prelude::*;

// --- Bolt effect components (applied by Amp chips) ---

/// Bolt pierces through N cells before stopping.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub(crate) struct Piercing(pub u32);

/// Remaining pierces before exhaustion. Reset when new Piercing stacks are applied.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub(crate) struct PiercingRemaining(pub u32);

/// Adds fractional bonus damage per stack.
///
/// Applied as: `damage = BASE_BOLT_DAMAGE * (1.0 + self.0)`.
/// With `DamageBoost(0.5)` (one stack at 0.5 per stack): 10 * 1.5 = 15 damage.
/// With `DamageBoost(1.0)` (two stacks at 0.5 per stack): 10 * 2.0 = 20 damage.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub(crate) struct DamageBoost(pub f32);

/// Adds flat speed to the bolt's base speed.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub(crate) struct BoltSpeedBoost(pub f32);

/// Bolt chains to N additional cells on hit.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub(crate) struct ChainHit(pub u32);

/// Increases bolt radius by a fraction.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub(crate) struct BoltSizeBoost(pub f32);

/// Attraction force magnitude pulling nearby cells toward the bolt.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub(crate) struct AttractionForce(pub f32);

// --- Breaker effect components (applied by Augment chips) ---

/// Adds flat width to the breaker.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub(crate) struct WidthBoost(pub f32);

/// Adds flat speed to the breaker's movement.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub(crate) struct BreakerSpeedBoost(pub f32);

/// Adds flat force applied during a bump.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub(crate) struct BumpForceBoost(pub f32);

/// Adds flat sensitivity to the breaker's tilt control.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub(crate) struct TiltControlBoost(pub f32);
