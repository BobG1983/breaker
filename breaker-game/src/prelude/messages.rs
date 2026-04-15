//! Re-exports of cross-domain messages.

// Note: `NoBump` message is not re-exported here because it collides with the
// `NoBump` typestate marker in `breaker/builder/core/types.rs`. Consumers that
// need the `NoBump` message must import it explicitly from `breaker::messages`.
pub(crate) use crate::{
    bolt::messages::{BoltImpactBreaker, BoltImpactCell, BoltImpactWall, BoltLost},
    breaker::messages::{BreakerImpactCell, BreakerImpactWall, BumpPerformed, BumpWhiffed},
    cells::messages::CellImpactWall,
    state::run::{
        chip_select::messages::ChipSelected,
        messages::{HighlightTriggered, RunLost},
    },
};
