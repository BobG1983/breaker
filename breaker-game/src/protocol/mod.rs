//! Protocol domain — run-long positive upgrades that change how the player
//! plays. Selected at chip-select time alongside chips.

pub mod definition;
pub(crate) mod messages;
pub(crate) mod plugin;
pub mod protocols;
pub mod resources;
pub(crate) mod systems;

use bevy::prelude::*;

use self::{
    definition::ProtocolKind,
    resources::{ActiveProtocols, ProtocolRegistry},
};
use crate::effect_v3::{commands::EffectCommandsExt, types::RootNode};

/// Activate a protocol using the registry-held tuning. Inserts the
/// definition into [`ActiveProtocols`] and either stamps the effect tree
/// onto each provided breaker entity (effect-tree protocols) or calls the
/// per-kind custom-system activator (which inserts a per-kind config
/// resource).
///
/// Returns `false` when no definition exists for `kind`. Mirrors
/// [`crate::hazard::activate_from_registry`] but takes a slice of breaker
/// entities instead of looking them up via a query, since the scenario
/// runner already has the tagged breakers in hand.
pub fn activate_from_registry(
    registry: &ProtocolRegistry,
    kind: ProtocolKind,
    breakers: &[Entity],
    commands: &mut Commands,
    active: &mut ActiveProtocols,
) -> bool {
    let Some(def) = registry.get(kind) else {
        return false;
    };
    let effects = def.tuning.effects().map(<[RootNode]>::to_vec);
    let tuning = def.tuning.clone();
    active.insert(def.clone());

    let Some(roots) = effects else {
        protocols::activate(kind, &tuning, commands);
        return true;
    };

    let source = format!("protocol:{kind:?}");
    for root in roots {
        if let RootNode::Stamp(_target, tree) = root {
            for &entity in breakers {
                commands.stamp_effect(entity, source.clone(), tree.clone());
            }
        }
    }
    true
}
