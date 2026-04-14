//! Watcher system that stamps `SpawnStampRegistry` entries onto newly spawned
//! breaker entities.

use bevy::prelude::*;

use super::super::resource::SpawnStampRegistry;
use crate::{
    breaker::components::Breaker,
    effect_v3::{commands::EffectCommandsExt, types::EntityKind},
};

/// Stamps registered `(name, tree)` pairs into `BoundEffects` on every newly
/// spawned `Breaker` entity.
///
/// Iterates the `SpawnStampRegistry.entries` for each newly-added `Breaker` and
/// delegates to `commands.stamp_effect` for every entry whose `EntityKind`
/// exactly matches `EntityKind::Breaker`. `EntityKind::Any` entries are ignored
/// — wildcarding is reserved for trigger-side matching, not spawn-time
/// stamping.
pub(crate) fn stamp_spawned_breakers(
    registry: Res<SpawnStampRegistry>,
    new_breakers: Query<Entity, Added<Breaker>>,
    mut commands: Commands,
) {
    const KIND: EntityKind = EntityKind::Breaker;

    if registry.entries.is_empty() {
        return;
    }

    for entity in &new_breakers {
        for (entry_kind, name, tree) in &registry.entries {
            if *entry_kind == KIND {
                commands.stamp_effect(entity, name.clone(), tree.clone());
            }
        }
    }
}
