//! Messages sent by the cells domain.

use bevy::prelude::*;

/// Sent when a cell is destroyed.
///
/// Consumed by run (progress tracking) and chips (overclock triggers).
#[derive(Message, Clone, Debug)]
pub(crate) struct CellDestroyed {
    /// Whether this cell counted toward node completion.
    pub was_required_to_clear: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cell_destroyed_debug_format() {
        let msg = CellDestroyed {
            was_required_to_clear: true,
        };
        assert!(format!("{msg:?}").contains("CellDestroyed"));
    }
}
