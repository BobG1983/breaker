//! `DeathPipelinePlugin` — registers the unified damage -> death -> despawn pipeline.

use bevy::prelude::*;

use super::sets::DeathPipelineSystems;

/// Plugin for the unified death pipeline.
///
/// Configures `DeathPipelineSystems` system sets with ordering constraints.
/// System registration is deferred to Phase 2 — this plugin currently only
/// configures set ordering.
pub struct DeathPipelinePlugin;

impl Plugin for DeathPipelinePlugin {
    fn build(&self, app: &mut App) {
        // System set ordering: ApplyDamage before DetectDeaths
        app.configure_sets(
            FixedUpdate,
            (
                DeathPipelineSystems::ApplyDamage,
                DeathPipelineSystems::DetectDeaths.after(DeathPipelineSystems::ApplyDamage),
            ),
        );
    }
}
