//! Per-protocol custom-system modules.
//!
//! Each module under this path owns one custom-system protocol's config
//! resource, activation logic, and runtime system(s). Effect-tree protocols
//! (Deadline, Ricochet, Anchor, Kickstart) have no module here — their
//! behaviour comes from the effect tree stamped by `dispatch_protocol_selection`.

use bevy::prelude::*;

use super::definition::{ProtocolKind, ProtocolTuning};

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
