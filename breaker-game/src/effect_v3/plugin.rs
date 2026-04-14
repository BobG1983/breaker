//! `EffectV3Plugin` — registers the effect v3 system sets, triggers, shared resources, and all
//! 30 effect configs via [`Fireable::register`]. Effect reset systems (e.g. `reset_entropy_counter`,
//! `reset_ramping_damage`) are registered into [`EffectV3Systems::Reset`] and run on
//! `OnEnter(NodeState::Loading)`, outside the `FixedUpdate` ordering chain.

use bevy::prelude::*;

use super::{
    conditions, effects,
    sets::EffectV3Systems,
    storage::{
        SpawnStampRegistry, stamp_spawned_bolts, stamp_spawned_breakers, stamp_spawned_cells,
        stamp_spawned_walls,
    },
    traits::Fireable,
    triggers,
};

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

        // Condition evaluation
        app.add_systems(
            FixedUpdate,
            conditions::evaluate_conditions.in_set(EffectV3Systems::Conditions),
        );

        // Shared resources
        app.init_resource::<SpawnStampRegistry>();

        // SpawnStampRegistry watchers — install matching entries on newly
        // spawned entities.
        app.add_systems(
            FixedUpdate,
            (
                stamp_spawned_bolts,
                stamp_spawned_cells,
                stamp_spawned_walls,
                stamp_spawned_breakers,
            )
                .in_set(EffectV3Systems::Bridge),
        );

        // Triggers — each category registers its own bridges and game systems
        triggers::bump::register::register(app);
        triggers::impact::register::register(app);
        triggers::death::register::register(app);
        triggers::bolt_lost::register::register(app);
        triggers::node::register::register(app);
        triggers::time::register::register(app);

        // Effects — each registers its own tick/update/cleanup/reset systems.
        // All 30 configs are registered even if their current register() is a
        // no-op, so that adding systems later cannot be silently dropped.
        effects::AnchorConfig::register(app);
        effects::AttractionConfig::register(app);
        effects::BumpForceConfig::register(app);
        effects::ChainBoltConfig::register(app);
        effects::ChainLightningConfig::register(app);
        effects::CircuitBreakerConfig::register(app);
        effects::DamageBoostConfig::register(app);
        effects::DieConfig::register(app);
        effects::EntropyConfig::register(app);
        effects::ExplodeConfig::register(app);
        effects::FlashStepConfig::register(app);
        effects::GravityWellConfig::register(app);
        effects::LoseLifeConfig::register(app);
        effects::MirrorConfig::register(app);
        effects::PiercingConfig::register(app);
        effects::PiercingBeamConfig::register(app);
        effects::PulseConfig::register(app);
        effects::QuickStopConfig::register(app);
        effects::RampingDamageConfig::register(app);
        effects::RandomEffectConfig::register(app);
        effects::SecondWindConfig::register(app);
        effects::ShieldConfig::register(app);
        effects::ShockwaveConfig::register(app);
        effects::SizeBoostConfig::register(app);
        effects::SpawnBoltsConfig::register(app);
        effects::SpawnPhantomConfig::register(app);
        effects::SpeedBoostConfig::register(app);
        effects::TetherBeamConfig::register(app);
        effects::TimePenaltyConfig::register(app);
        effects::VulnerableConfig::register(app);
    }
}
