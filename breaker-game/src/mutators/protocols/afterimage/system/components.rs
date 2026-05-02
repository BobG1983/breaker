use bevy::prelude::*;

/// Remaining lifetime in seconds on a `PhantomBreaker` entity. Ticked
/// down by `afterimage_tick_phantom_breaker`; the entity is despawned when
/// the value reaches `0.0` or below.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub(crate) struct PhantomBreakerLifetime(pub f32);
