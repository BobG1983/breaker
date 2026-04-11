//! `EffectV3Plugin` — registers the effect v3 system sets and shared resources.

use bevy::prelude::*;

use super::{sets::EffectV3Systems, storage::SpawnStampRegistry};

/// Plugin for the effect v3 domain.
///
/// Configures `EffectV3Systems` system sets with ordering and initializes
/// shared resources. System registration is deferred to Phase 2 — this
/// plugin currently registers no systems.
pub struct EffectV3Plugin;

impl Plugin for EffectV3Plugin {
    fn build(&self, app: &mut App) {
        // System set ordering: Bridge → Tick → Conditions
        app.configure_sets(
            FixedUpdate,
            (
                EffectV3Systems::Bridge,
                EffectV3Systems::Tick.after(EffectV3Systems::Bridge),
                EffectV3Systems::Conditions.after(EffectV3Systems::Tick),
            ),
        );

        // Shared resources
        app.init_resource::<SpawnStampRegistry>();
    }
}
