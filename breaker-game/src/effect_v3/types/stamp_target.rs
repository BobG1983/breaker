//! `StampTarget` — identifies which entities a Stamp root node installs effects on.

use serde::{Deserialize, Serialize};

/// Identifies which entities a `Stamp` root node installs effects on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StampTarget {
    /// Primary bolt entity.
    Bolt,
    /// Primary breaker entity.
    Breaker,
    /// All bolt entities that exist right now.
    ActiveBolts,
    /// All existing bolts + all bolts spawned in the future.
    EveryBolt,
    /// All bolts with the `PrimaryBolt` marker.
    PrimaryBolts,
    /// All bolts with the `ExtraBolt` marker.
    ExtraBolts,
    /// All cell entities that exist right now.
    ActiveCells,
    /// All existing cells + all cells spawned in the future.
    EveryCell,
    /// All wall entities that exist right now.
    ActiveWalls,
    /// All existing walls + all walls spawned in the future.
    EveryWall,
    /// All breaker entities that exist right now.
    ActiveBreakers,
    /// All existing breakers + all breakers spawned in the future.
    EveryBreaker,
}
