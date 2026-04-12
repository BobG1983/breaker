//! `DeathPipelinePlugin` — registers the unified damage -> death -> despawn pipeline.

use bevy::prelude::*;

use super::{
    damage_dealt::DamageDealt, despawn_entity::DespawnEntity, destroyed::Destroyed,
    kill_yourself::KillYourself, sets::DeathPipelineSystems, systems,
};
use crate::{
    bolt::components::Bolt, breaker::components::Breaker, cells::components::Cell,
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

        app.add_message::<KillYourself<Cell>>();
        app.add_message::<KillYourself<Bolt>>();
        app.add_message::<KillYourself<Wall>>();
        app.add_message::<KillYourself<Breaker>>();

        app.add_message::<Destroyed<Cell>>();
        app.add_message::<Destroyed<Bolt>>();
        app.add_message::<Destroyed<Wall>>();
        app.add_message::<Destroyed<Breaker>>();

        app.add_message::<DespawnEntity>();

        // System set ordering: ApplyDamage before DetectDeaths
        app.configure_sets(
            FixedUpdate,
            (
                DeathPipelineSystems::ApplyDamage,
                DeathPipelineSystems::DetectDeaths.after(DeathPipelineSystems::ApplyDamage),
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
            )
                .in_set(DeathPipelineSystems::DetectDeaths),
        );

        // Deferred despawn — runs after all FixedUpdate processing
        app.add_systems(FixedPostUpdate, systems::process_despawn_requests);
    }
}
