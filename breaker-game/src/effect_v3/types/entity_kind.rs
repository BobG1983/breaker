//! `EntityKind` — classifies entity types for trigger matching.

use serde::{Deserialize, Serialize};

/// Classifies an entity type for trigger and participant matching.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EntityKind {
    /// Matches cell entities.
    Cell,
    /// Matches bolt entities.
    Bolt,
    /// Matches wall entities.
    Wall,
    /// Matches breaker entities.
    Breaker,
    /// Matches any entity type.
    Any,
}
