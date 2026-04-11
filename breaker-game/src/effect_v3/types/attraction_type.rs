//! `AttractionType` — target entity kind for attraction steering.

use serde::{Deserialize, Serialize};

/// Which entity kind to steer toward when attraction is active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AttractionType {
    /// Steer toward nearest breaker entity.
    Breaker,
    /// Steer toward nearest bolt entity.
    Bolt,
    /// Steer toward nearest cell entity.
    Cell,
    /// Steer toward nearest wall entity.
    Wall,
}
