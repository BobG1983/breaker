//! `EffectV3Plugin` — registers the effect v3 system sets, triggers, and shared resources.

use bevy::prelude::*;

use super::{sets::EffectV3Systems, storage::SpawnStampRegistry, triggers};

/// Plugin for the effect v3 domain.
///
/// Configures `EffectV3Systems` system sets with ordering, registers all trigger
/// bridge systems, and initializes shared resources.
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

        // Triggers — each category registers its own bridges and game systems
        triggers::bump::register::register(app);
        triggers::impact::register::register(app);
        triggers::death::register::register(app);
        triggers::bolt_lost::register::register(app);
        triggers::node::register::register(app);
        triggers::time::register::register(app);
    }
}
