use bevy::prelude::*;

use super::super::{
    damage_dealt::DamageDealt, despawn_entity::DespawnEntity, destroyed::Destroyed,
    heal_dealt::HealDealt, kill_yourself::KillYourself, sets::DeathPipelineSystems, systems,
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
/// Registers `DamageDealt<T>`, `HealDealt<T>`, `KillYourself<T>`, and
/// `Destroyed<T>` messages for all entity types, wires up `apply_damage<T>`,
/// `detect_deaths<T>`, `handle_kill<T>`, and `apply_heal<T>` systems in the
/// four-stage `FixedUpdate` chain `ApplyDamage -> DetectDeaths -> HandleKill
/// -> ApplyHeal`, and schedules `process_despawn_requests` in
/// `FixedPostUpdate`. `ApplyHeal` runs after `HandleKill` so the
/// `Without<Dead>` filter on `apply_heal<T>`'s query prevents same-tick
/// revival of entities that were damage-killed earlier in the tick.
pub(crate) struct DeathPipelinePlugin;

impl Plugin for DeathPipelinePlugin {
    fn build(&self, app: &mut App) {
        // Message registration — one queue per entity type per message kind
        app.add_message::<DamageDealt<Cell>>();
        app.add_message::<DamageDealt<Bolt>>();
        app.add_message::<DamageDealt<Wall>>();
        app.add_message::<DamageDealt<Breaker>>();
        app.add_message::<DamageDealt<Salvo>>();

        app.add_message::<HealDealt<Cell>>();
        app.add_message::<HealDealt<Bolt>>();
        app.add_message::<HealDealt<Wall>>();
        app.add_message::<HealDealt<Breaker>>();
        app.add_message::<HealDealt<Salvo>>();

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
                DeathPipelineSystems::ApplyHeal.after(DeathPipelineSystems::HandleKill),
            ),
        );

        // Damage application — monomorphized per entity type.
        //
        // NOTE: `Cell` is intentionally absent here. Cell damage application is
        // owned by `cells::systems::apply_damage_to_cells` in the cells domain
        // (registered by `CellsPlugin`) so that Diffusion's BFS redistribution
        // logic lives alongside the cell-specific damage path. The other four
        // monomorphizations continue to use the generic `apply_damage::<T>`.
        app.add_systems(
            FixedUpdate,
            (
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

        // Heal application — monomorphized per entity type
        app.add_systems(
            FixedUpdate,
            (
                systems::apply_heal::<Cell>,
                systems::apply_heal::<Bolt>,
                systems::apply_heal::<Wall>,
                systems::apply_heal::<Breaker>,
                systems::apply_heal::<Salvo>,
            )
                .in_set(DeathPipelineSystems::ApplyHeal),
        );

        // Deferred despawn — runs after all FixedUpdate processing
        app.add_systems(FixedPostUpdate, systems::process_despawn_requests);
    }
}
