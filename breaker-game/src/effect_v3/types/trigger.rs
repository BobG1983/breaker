//! Trigger — game events that gate effect tree evaluation.

use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use super::EntityKind;

/// A game event that gates effect tree evaluation.
///
/// Local triggers fire on specific participating entities.
/// Global triggers (suffix `Occurred`) fire on all entities with bound effects.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Trigger {
    /// Local: bolt bumped with perfect timing. Fires on bolt + breaker.
    PerfectBumped,
    /// Local: bolt bumped with early timing. Fires on bolt + breaker.
    EarlyBumped,
    /// Local: bolt bumped with late timing. Fires on bolt + breaker.
    LateBumped,
    /// Local: bolt bumped with any successful timing. Fires on bolt + breaker.
    Bumped,
    /// Global: a perfect bump happened somewhere. Fires on all entities.
    PerfectBumpOccurred,
    /// Global: an early bump happened somewhere. Fires on all entities.
    EarlyBumpOccurred,
    /// Global: a late bump happened somewhere. Fires on all entities.
    LateBumpOccurred,
    /// Global: any successful bump happened somewhere. Fires on all entities.
    BumpOccurred,
    /// Global: bump timing window expired without contact. Fires on all entities.
    BumpWhiffOccurred,
    /// Global: bolt hit breaker with no bump input. Fires on all entities.
    NoBumpOccurred,
    /// Local: this entity collided with an entity of the given kind.
    Impacted(EntityKind),
    /// Global: a collision involving the given entity kind happened somewhere.
    ImpactOccurred(EntityKind),
    /// Local: this entity died. Fires on victim only.
    Died,
    /// Local: this entity killed an entity of the given kind. Fires on killer only.
    Killed(EntityKind),
    /// Global: an entity of the given kind died somewhere. Fires on all entities.
    DeathOccurred(EntityKind),
    /// Global: a bolt fell off the bottom. Fires on all entities.
    BoltLostOccurred,
    /// Global: a new node started. Fires on all entities.
    NodeStartOccurred,
    /// Global: the current node ended. Fires on all entities.
    NodeEndOccurred,
    /// Global: node timer ratio crossed the given threshold (0.0-1.0).
    NodeTimerThresholdOccurred(OrderedFloat<f32>),
    /// Self: countdown of the given seconds reached zero. Fires on owner only.
    TimeExpires(OrderedFloat<f32>),
}
