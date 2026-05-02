//! Breaker domain query filters.

use bevy::prelude::*;

use crate::{
    breaker::components::{BumpFeedbackState, PhantomBreaker},
    prelude::*,
};

/// Query filter for real (non-phantom) breakers.
pub type RealBreakerFilter = (With<Breaker>, Without<PhantomBreaker>);

/// Query filter for breakers eligible for a new bump visual (not already animating).
pub type BumpTriggerFilter = (
    With<Breaker>,
    Without<BumpFeedbackState>,
    Without<PhantomBreaker>,
);

/// Breaker entities for collision queries (excludes bolt for query disjointness).
pub(crate) type CollisionFilterBreaker = (With<Breaker>, Without<Bolt>);
