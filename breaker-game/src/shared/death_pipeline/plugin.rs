//! `DeathPipelinePlugin` — registers the unified damage -> death -> despawn pipeline.

use bevy::prelude::*;

use super::{despawn_entity::DespawnEntity, sets::DeathPipelineSystems, systems};

/// Plugin for the unified death pipeline.
///
/// Configures `DeathPipelineSystems` system sets with ordering constraints and
/// registers `process_despawn_requests` in `FixedPostUpdate`.
///
/// Generic system registrations (`apply_damage::<T>`, `detect_deaths::<T>`) are
/// deferred until `GameEntity` impls exist for domain marker types (Cell, Bolt,
/// Wall, Breaker).
pub struct DeathPipelinePlugin;

impl Plugin for DeathPipelinePlugin {
    fn build(&self, app: &mut App) {
        // Message registration
        app.add_message::<DespawnEntity>();

        // System set ordering: ApplyDamage before DetectDeaths
        app.configure_sets(
            FixedUpdate,
            (
                DeathPipelineSystems::ApplyDamage,
                DeathPipelineSystems::DetectDeaths.after(DeathPipelineSystems::ApplyDamage),
            ),
        );

        // Deferred despawn — runs after all FixedUpdate processing
        app.add_systems(FixedPostUpdate, systems::process_despawn_requests);
    }
}
