//! Participant target enums for On node resolution.

use serde::{Deserialize, Serialize};

/// A role in a bump event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BumpTarget {
    /// The bolt that was bumped.
    Bolt,
    /// The breaker that did the bumping.
    Breaker,
}

/// A role in a collision event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ImpactTarget {
    /// The entity that initiated the collision.
    Impactor,
    /// The entity that was hit.
    Impactee,
}

/// A role in a death event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeathTarget {
    /// The entity that died.
    Victim,
    /// The entity that caused the death (may not exist for environmental deaths).
    Killer,
}

/// A role in a bolt lost event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BoltLostTarget {
    /// The bolt that was lost.
    Bolt,
    /// The breaker that lost the bolt.
    Breaker,
}

/// Wrapper for participant role enums, used by On nodes to redirect
/// terminals to a specific participant in the trigger event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ParticipantTarget {
    /// A role in a bump event.
    Bump(BumpTarget),
    /// A role in a collision event.
    Impact(ImpactTarget),
    /// A role in a death event.
    Death(DeathTarget),
    /// A role in a bolt lost event.
    BoltLost(BoltLostTarget),
}
