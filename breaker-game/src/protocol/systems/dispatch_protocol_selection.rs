//! System to dispatch `ProtocolSelected` messages into `ActiveProtocols`
//! and stamp effect trees onto every Breaker.
//!
//! Consumes each `ProtocolSelected`, looks up the definition in
//! `ProtocolRegistry`, inserts the definition into `ActiveProtocols`, and —
//! for effect-tree protocols — stamps each `RootNode::Stamp` tree onto every
//! `Breaker` entity. Custom-system protocols (where `tuning.effects()`
//! returns `None`) are inserted into `ActiveProtocols` only; their per-kind
//! runtime systems are registered independently by each protocol module.

use bevy::prelude::*;

use crate::{
    effect_v3::{commands::EffectCommandsExt, types::RootNode},
    prelude::*,
    protocol::{
        messages::ProtocolSelected,
        protocols,
        resources::{ActiveProtocols, ProtocolRegistry},
    },
};

/// Dispatch selected protocols: insert into [`ActiveProtocols`] and stamp
/// effect trees onto every Breaker.
pub(crate) fn dispatch_protocol_selection(
    mut reader: MessageReader<ProtocolSelected>,
    registry: Res<ProtocolRegistry>,
    mut active: ResMut<ActiveProtocols>,
    breakers: Query<Entity, With<Breaker>>,
    mut commands: Commands,
) {
    for msg in reader.read() {
        let kind = msg.kind;
        let Some(def) = registry.get(kind) else {
            warn!("protocol {kind:?} not found in ProtocolRegistry");
            continue;
        };

        let effects = def.tuning.effects().map(<[RootNode]>::to_vec);
        let tuning = def.tuning.clone();
        active.insert(def.clone());

        let Some(roots) = effects else {
            // Custom-system protocol — register its per-kind config + runtime
            // via the `protocols::` fan-in; runtime systems are gated by
            // `protocol_active(kind)`.
            protocols::activate(kind, &tuning, &mut commands);
            continue;
        };

        let source = format!("protocol:{kind:?}");
        for root in roots {
            match root {
                RootNode::Stamp(_target, tree) => {
                    for breaker_entity in breakers.iter() {
                        commands.stamp_effect(breaker_entity, source.clone(), tree.clone());
                    }
                }
                RootNode::Spawn(_entity_kind, _tree) => {
                    // Spawn-based roots are not produced by protocol effect trees.
                }
            }
        }
    }
}

#[cfg(test)]
mod tests;
