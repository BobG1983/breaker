//! Protocol domain — run-long positive upgrades that change how the player
//! plays. Selected at chip-select time alongside chips.
//!
//! Each per-protocol module under this path owns one custom-system
//! protocol's config resource, activation logic, and runtime system(s).
//! Effect-tree protocols (Deadline, Ricochet, Anchor, Kickstart) have no
//! module here — their behaviour comes from the effect tree stamped by
//! `dispatch_protocol_selection`.

pub mod definition;
pub(crate) mod messages;
pub mod resources;
pub(crate) mod systems;

pub(crate) mod afterimage;
pub mod burnout;
pub(crate) mod conductor;
pub mod debt_collector;
pub mod echo_strike;
pub mod fission;
pub mod greed;
pub(crate) mod iron_curtain;
pub mod reckless_dash;
pub mod siphon;
pub(crate) mod tier_regression;

#[cfg(test)]
pub(crate) mod test_utils;

use bevy::prelude::*;

use self::{
    definition::{ProtocolKind, ProtocolTuning},
    resources::{ActiveProtocols, ProtocolRegistry},
};
use crate::effect_v3::{commands::EffectCommandsExt, types::RootNode};

/// Entry point called by `dispatch_protocol_selection` when a custom-system
/// protocol is selected. Unpacks tuning into the appropriate config resource.
pub(crate) fn activate(kind: ProtocolKind, tuning: &ProtocolTuning, commands: &mut Commands) {
    match kind {
        ProtocolKind::DebtCollector => debt_collector::activate(tuning, commands),
        ProtocolKind::IronCurtain => iron_curtain::activate(tuning, commands),
        ProtocolKind::EchoStrike => echo_strike::activate(tuning, commands),
        ProtocolKind::Siphon => siphon::activate(tuning, commands),
        ProtocolKind::Greed => greed::activate(tuning, commands),
        ProtocolKind::RecklessDash => reckless_dash::activate(tuning, commands),
        ProtocolKind::Burnout => burnout::activate(tuning, commands),
        ProtocolKind::Conductor => conductor::activate(tuning, commands),
        ProtocolKind::Afterimage => afterimage::activate(tuning, commands),
        ProtocolKind::Fission => fission::activate(tuning, commands),
        ProtocolKind::TierRegression => tier_regression::activate(tuning, commands),
        // Effect-tree protocols handle themselves via the effect system.
        ProtocolKind::Deadline
        | ProtocolKind::Ricochet
        | ProtocolKind::Anchor
        | ProtocolKind::Kickstart => {}
    }
}

/// Fan-out registration — each custom-system protocol registers its runtime
/// systems via its own `register(app)` function.
pub(crate) fn register(app: &mut App) {
    debt_collector::register(app);
    iron_curtain::register(app);
    echo_strike::register(app);
    siphon::register(app);
    greed::register(app);
    reckless_dash::register(app);
    burnout::register(app);
    conductor::register(app);
    afterimage::register(app);
    fission::register(app);
    tier_regression::register(app);
}

/// Activate a protocol using the registry-held tuning. Inserts the
/// definition into [`ActiveProtocols`] and either stamps the effect tree
/// onto each provided breaker entity (effect-tree protocols) or calls the
/// per-kind custom-system activator (which inserts a per-kind config
/// resource).
///
/// Returns `false` when no definition exists for `kind`. Mirrors
/// [`crate::mutators::hazards::activate_from_registry`] but takes a slice of
/// breaker entities instead of looking them up via a query, since the
/// scenario runner already has the tagged breakers in hand.
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
        activate(kind, &tuning, commands);
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
