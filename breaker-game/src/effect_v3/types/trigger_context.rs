//! `TriggerContext` — carries entity participants from a trigger event for On node resolution.

use bevy::prelude::*;

/// Carries the entities involved in a trigger event so that On nodes
/// can resolve `ParticipantTarget` values during tree walking.
#[derive(Debug, Clone)]
pub enum TriggerContext {
    /// Participants in a bump event.
    Bump {
        /// The bolt that was bumped (None for NoBump/BumpWhiff without a bolt).
        bolt:    Option<Entity>,
        /// The breaker that did the bumping.
        breaker: Entity,
    },
    /// Both participants in a collision.
    Impact {
        /// The entity that initiated the collision.
        impactor: Entity,
        /// The entity that was hit.
        impactee: Entity,
    },
    /// The victim and optionally the killer.
    Death {
        /// The entity that died.
        victim: Entity,
        /// The entity that caused the death (None for environmental deaths).
        killer: Option<Entity>,
    },
    /// The bolt that was lost and the breaker that lost it.
    BoltLost {
        /// The bolt that was lost.
        bolt:    Entity,
        /// The breaker that lost the bolt.
        breaker: Entity,
    },
    /// No participants. Used for global triggers with no event-specific entities.
    None,
}
