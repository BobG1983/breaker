//! Watcher system that stamps `SpawnStampRegistry` entries onto newly spawned
//! bolt entities.

use bevy::prelude::*;

use super::super::resource::SpawnStampRegistry;
use crate::{
    bolt::components::Bolt,
    effect_v3::{commands::EffectCommandsExt, types::EntityKind},
};

/// Stamps registered `(name, tree)` pairs into `BoundEffects` on every newly
/// spawned `Bolt` entity.
///
/// Iterates the `SpawnStampRegistry.entries` for each newly-added `Bolt` and
/// delegates to `commands.stamp_effect` for every entry whose `EntityKind`
/// exactly matches `EntityKind::Bolt`. `EntityKind::Any` entries are ignored —
/// wildcarding is reserved for trigger-side matching, not spawn-time stamping.
pub(crate) fn stamp_spawned_bolts(
    registry: Res<SpawnStampRegistry>,
    new_bolts: Query<Entity, Added<Bolt>>,
    mut commands: Commands,
) {
    const KIND: EntityKind = EntityKind::Bolt;

    if registry.entries.is_empty() {
        return;
    }

    for entity in &new_bolts {
        for (entry_kind, name, tree) in &registry.entries {
            if *entry_kind == KIND {
                commands.stamp_effect(entity, name.clone(), tree.clone());
            }
        }
    }
}
