//! Condition — state predicates for During node evaluation.

use serde::{Deserialize, Serialize};

/// A state predicate evaluated each frame for During nodes.
///
/// During nodes apply their inner effects while the condition is true
/// and reverse them when the condition becomes false.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Condition {
    /// True while a node is playing or paused.
    Node,
    /// True while at least one `ShieldWall` entity exists in the world.
    Shield,
    /// True while the consecutive perfect bump streak is at or above the given count.
    Combo(u32),
}
