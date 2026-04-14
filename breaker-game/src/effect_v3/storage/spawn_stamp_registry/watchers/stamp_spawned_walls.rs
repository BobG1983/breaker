//! Watcher system that stamps `SpawnStampRegistry` entries onto newly spawned
//! wall entities.

use bevy::prelude::*;

use super::super::resource::SpawnStampRegistry;
use crate::{
    effect_v3::{commands::EffectCommandsExt, types::EntityKind},
    walls::components::Wall,
};

/// Stamps registered `(name, tree)` pairs into `BoundEffects` on every newly
/// spawned `Wall` entity.
///
/// Iterates the `SpawnStampRegistry.entries` for each newly-added `Wall` and
/// delegates to `commands.stamp_effect` for every entry whose `EntityKind`
/// exactly matches `EntityKind::Wall`. `EntityKind::Any` entries are ignored —
/// wildcarding is reserved for trigger-side matching, not spawn-time stamping.
pub(crate) fn stamp_spawned_walls(
    registry: Res<SpawnStampRegistry>,
    new_walls: Query<Entity, Added<Wall>>,
    mut commands: Commands,
) {
    const KIND: EntityKind = EntityKind::Wall;

    if registry.entries.is_empty() {
        return;
    }

    for entity in &new_walls {
        for (entry_kind, name, tree) in &registry.entries {
            if *entry_kind == KIND {
                commands.stamp_effect(entity, name.clone(), tree.clone());
            }
        }
    }
}
