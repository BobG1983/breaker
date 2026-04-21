use bevy::prelude::*;

// ── PhantomBreaker ──────────────────────────────────────────────────────────

/// Zero-field marker — identifies an entity as a phantom-breaker ghost
/// spawned by `afterimage_spawn_phantom_breaker`.
#[derive(Component, Debug)]
pub(crate) struct PhantomBreaker;

/// Remaining lifetime in seconds on a `PhantomBreaker` entity. Ticked
/// down by `afterimage_tick_phantom_breaker`; the entity is despawned when
/// the value reaches `0.0` or below.
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub(crate) struct PhantomBreakerLifetime(pub f32);
