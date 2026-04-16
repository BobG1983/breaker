//! Cells plugin registration.

use bevy::prelude::*;

use crate::{
    bolt::sets::BoltSystems,
    cells::{
        behaviors::{
            armored::systems::check_armor_direction::check_armor_direction,
            guarded::systems::slide_guardian_cells,
            locked::systems::{check_lock_release, sync_lock_invulnerable::sync_lock_invulnerable},
            magnetic::systems::apply_magnetic_fields,
            phantom::systems::tick_phantom_phase,
            portal::systems::{check_portal_entry, handle_portal_completed, handle_portal_entered},
            regen::systems::tick_cell_regen,
            sequence::systems::{
                advance_sequence::advance_sequence, init_sequence_groups::init_sequence_groups,
                reset_inactive_sequence_hp::reset_inactive_sequence_hp,
            },
            survival::{
                salvo::systems::{
                    fire_survival_turret::fire_survival_turret,
                    salvo_bolt_collision::salvo_bolt_collision,
                    salvo_breaker_collision::salvo_breaker_collision,
                    salvo_cell_collision::salvo_cell_collision,
                    salvo_wall_collision::salvo_wall_collision,
                    tick_salvo_fire_timer::tick_salvo_fire_timer,
                    tick_survival_timer::tick_survival_timer,
                },
                systems::suppress_bolt_immune_damage::suppress_bolt_immune_damage,
            },
        },
        messages::{CellImpactWall, PortalCompleted, PortalEntered, SalvoImpactBreaker},
        resources::CellConfig,
        systems::{cell_wall_collision, update_cell_damage_visuals},
    },
    effect_v3::sets::EffectV3Systems,
    prelude::*,
    shared::death_pipeline::sets::DeathPipelineSystems,
    state::run::node::{sets::NodeSystems, systems::dispatch_cell_effects},
};

/// Plugin for the cells domain.
///
/// Owns cell components, damage handling, and destruction logic.
pub(crate) struct CellsPlugin;

impl Plugin for CellsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<CellImpactWall>()
            .add_message::<SalvoImpactBreaker>()
            .add_message::<PortalEntered>()
            .add_message::<PortalCompleted>()
            .init_resource::<CellConfig>()
            .add_systems(
                OnEnter(NodeState::Loading),
                dispatch_cell_effects.after(NodeSystems::Spawn),
            )
            .add_systems(OnEnter(NodeState::Playing), init_sequence_groups)
            .add_systems(
                FixedUpdate,
                (
                    check_lock_release.after(DeathPipelineSystems::HandleKill),
                    sync_lock_invulnerable.after(check_lock_release),
                    tick_cell_regen,
                    tick_phantom_phase,
                    slide_guardian_cells,
                    apply_magnetic_fields,
                    cell_wall_collision,
                    update_cell_damage_visuals
                        .after(DeathPipelineSystems::ApplyDamage)
                        .before(DeathPipelineSystems::HandleKill),
                    reset_inactive_sequence_hp
                        .after(DeathPipelineSystems::ApplyDamage)
                        .before(DeathPipelineSystems::DetectDeaths),
                    advance_sequence.after(EffectV3Systems::Death),
                    check_armor_direction
                        .after(BoltSystems::CellCollision)
                        .before(DeathPipelineSystems::ApplyDamage),
                    suppress_bolt_immune_damage
                        .after(check_armor_direction)
                        .before(DeathPipelineSystems::ApplyDamage),
                )
                    .run_if(in_state(NodeState::Playing)),
            )
            .add_systems(
                FixedUpdate,
                (
                    tick_survival_timer.before(DeathPipelineSystems::ApplyDamage),
                    tick_salvo_fire_timer.after(tick_survival_timer),
                    fire_survival_turret.after(tick_salvo_fire_timer),
                    salvo_cell_collision.before(DeathPipelineSystems::ApplyDamage),
                    salvo_bolt_collision,
                    salvo_breaker_collision.before(EffectV3Systems::Bridge),
                    salvo_wall_collision,
                    check_portal_entry.after(BoltSystems::CellCollision),
                    handle_portal_entered.after(check_portal_entry),
                    handle_portal_completed
                        .after(handle_portal_entered)
                        .before(DeathPipelineSystems::HandleKill),
                )
                    .run_if(in_state(NodeState::Playing)),
            );
    }
}
