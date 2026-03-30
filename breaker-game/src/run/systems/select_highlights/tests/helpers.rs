//! Shared test helpers for `select_highlights` tests.

use crate::run::{
    definition::HighlightConfig,
    resources::{HighlightKind, RunHighlight},
};

pub(super) fn default_config() -> HighlightConfig {
    HighlightConfig::default()
}

pub(super) fn highlight(kind: HighlightKind, value: f32) -> RunHighlight {
    RunHighlight {
        kind,
        node_index: 0,
        value,
        detail: None,
    }
}
