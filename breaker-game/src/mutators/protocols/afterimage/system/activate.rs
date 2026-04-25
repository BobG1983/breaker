use bevy::prelude::*;

use super::config::AfterimageConfig;
use crate::mutators::protocols::definition::ProtocolTuning;

// ── activate ────────────────────────────────────────────────────────────────

/// Inserts `AfterimageConfig` from `ProtocolTuning::Afterimage`. Warns and
/// no-ops on a non-`Afterimage` tuning variant, leaving any existing config
/// intact.
pub(crate) fn activate(tuning: &ProtocolTuning, commands: &mut Commands) {
    let ProtocolTuning::Afterimage {
        phantom_duration,
        phantom_bolt_duration,
    } = *tuning
    else {
        warn!("afterimage::activate called with non-Afterimage tuning");
        return;
    };
    commands.insert_resource(AfterimageConfig {
        phantom_duration,
        phantom_bolt_duration,
    });
}
