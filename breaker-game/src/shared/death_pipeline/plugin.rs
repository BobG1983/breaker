//! `DeathPipelinePlugin` — registers the unified damage -> death -> despawn pipeline.

use bevy::prelude::*;

use super::{
    damage_dealt::DamageDealt, despawn_entity::DespawnEntity, destroyed::Destroyed,
    kill_yourself::KillYourself, sets::DeathPipelineSystems, systems,
};
use crate::{
    bolt::components::Bolt,
    breaker::components::Breaker,
    cells::{behaviors::survival::salvo::components::Salvo, components::Cell},
    effect_v3::sets::EffectV3Systems,
    walls::components::Wall,
};

/// Plugin for the unified death pipeline.
///
/// Registers `DamageDealt<T>`, `KillYourself<T>`, and `Destroyed<T>` messages
/// for all entity types, wires up `apply_damage<T>` and `detect_deaths<T>`
/// systems, and schedules `process_despawn_requests` in `FixedPostUpdate`.
pub(crate) struct DeathPipelinePlugin;

impl Plugin for DeathPipelinePlugin {
    fn build(&self, app: &mut App) {
        // Message registration — one queue per entity type per message kind
        app.add_message::<DamageDealt<Cell>>();
        app.add_message::<DamageDealt<Bolt>>();
        app.add_message::<DamageDealt<Wall>>();
        app.add_message::<DamageDealt<Breaker>>();
        app.add_message::<DamageDealt<Salvo>>();

        app.add_message::<KillYourself<Cell>>();
        app.add_message::<KillYourself<Bolt>>();
        app.add_message::<KillYourself<Wall>>();
        app.add_message::<KillYourself<Breaker>>();
        app.add_message::<KillYourself<Salvo>>();

        app.add_message::<Destroyed<Cell>>();
        app.add_message::<Destroyed<Bolt>>();
        app.add_message::<Destroyed<Wall>>();
        app.add_message::<Destroyed<Breaker>>();
        app.add_message::<Destroyed<Salvo>>();

        app.add_message::<DespawnEntity>();

        // System set ordering: ApplyDamage after effect tick, DetectDeaths after
        // ApplyDamage, HandleKill after DetectDeaths.
        app.configure_sets(
            FixedUpdate,
            (
                DeathPipelineSystems::ApplyDamage.after(EffectV3Systems::Tick),
                DeathPipelineSystems::DetectDeaths.after(DeathPipelineSystems::ApplyDamage),
                DeathPipelineSystems::HandleKill.after(DeathPipelineSystems::DetectDeaths),
            ),
        );

        // Damage application — monomorphized per entity type
        app.add_systems(
            FixedUpdate,
            (
                systems::apply_damage::<Cell>,
                systems::apply_damage::<Bolt>,
                systems::apply_damage::<Wall>,
                systems::apply_damage::<Breaker>,
                systems::apply_damage::<Salvo>,
            )
                .in_set(DeathPipelineSystems::ApplyDamage),
        );

        // Death detection — monomorphized per entity type
        app.add_systems(
            FixedUpdate,
            (
                systems::detect_deaths::<Cell>,
                systems::detect_deaths::<Bolt>,
                systems::detect_deaths::<Wall>,
                systems::detect_deaths::<Breaker>,
                systems::detect_deaths::<Salvo>,
            )
                .in_set(DeathPipelineSystems::DetectDeaths),
        );

        // Kill handling — monomorphized per entity type. Consumes
        // `KillYourself<T>`, marks `Dead`, emits `Destroyed<T>`, and
        // enqueues `DespawnEntity`.
        //
        // `Cell` and `Bolt` are the active producers today. `Wall` is wired
        // as a future-proofing measure: walls have no death producer in the
        // current game, but the generic handler is harmless — if no
        // `KillYourself<Wall>` messages are written, the system is a no-op.
        // `Breaker` is handled separately by
        // [`handle_breaker_death`](crate::state::run::node::lifecycle::systems::handle_breaker_death)
        // because the breaker must survive through the end-of-run flow and
        // therefore cannot use the generic `DespawnEntity`-emitting handler.
        app.add_systems(
            FixedUpdate,
            (
                systems::handle_kill::<Cell>,
                systems::handle_kill::<Bolt>,
                systems::handle_kill::<Wall>,
                systems::handle_kill::<Salvo>,
            )
                .in_set(DeathPipelineSystems::HandleKill),
        );

        // Deferred despawn — runs after all FixedUpdate processing
        app.add_systems(FixedPostUpdate, systems::process_despawn_requests);
    }
}
