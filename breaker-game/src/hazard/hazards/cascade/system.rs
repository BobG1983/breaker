//! Cascade hazard — heals neighbouring cells when one dies. Stacks add
//! linear heal per neighbour. "Adjacent" is approximated as any cell whose
//! centre is within `ADJACENCY_RADIUS` world units of the destroyed cell's
//! position; this avoids depending on grid-coordinate lookup which the
//! cells domain doesn't expose today.

use std::marker::PhantomData;

use bevy::prelude::*;
use rantzsoft_spatial2d::components::Position2D;

use crate::{
    cells::components::{ADJACENCY_RADIUS_SQ, Cell},
    hazard::{
        definition::{HazardKind, HazardTuning},
        resources::{ActiveHazards, hazard_active},
    },
    prelude::*,
    shared::death_pipeline::{
        Destroyed, HealCap, Hp, heal_dealt::HealDealt, sets::DeathPipelineSystems,
    },
};

/// Per-run tuning extracted from [`HazardTuning::Cascade`] at activation.
#[derive(Resource, Debug, Clone, Copy)]
pub(crate) struct CascadeConfig {
    /// HP healed to each adjacent cell at stack 1.
    pub(crate) base_heal:      f32,
    /// Additional HP healed per stack beyond the first.
    pub(crate) per_level_heal: f32,
}

impl CascadeConfig {
    /// Heal amount per neighbour for the given stack count.
    /// Returns 0.0 when `stacks == 0`.
    #[must_use]
    pub(crate) const fn heal_per_neighbour(self, stacks: u32) -> f32 {
        if stacks == 0 {
            return 0.0;
        }
        let extra = stacks.saturating_sub(1) as f32;
        self.per_level_heal.mul_add(extra, self.base_heal)
    }
}

/// Inserts `CascadeConfig` from `HazardTuning::Cascade`. Called each time
/// the player picks Cascade; last write wins (overwrites any prior config).
/// Warns and no-ops on a non-Cascade tuning variant.
pub(crate) fn activate(tuning: &HazardTuning, commands: &mut Commands) {
    let HazardTuning::Cascade {
        base_heal,
        per_level_heal,
    } = *tuning
    else {
        warn!("cascade::activate called with non-Cascade tuning");
        return;
    };
    commands.insert_resource(CascadeConfig {
        base_heal,
        per_level_heal,
    });
}

/// Registers `cascade_heal_on_death` in `FixedUpdate` between
/// `DeathPipelineSystems::HandleKill` (which emits `Destroyed<Cell>`) and
/// `DeathPipelineSystems::ApplyHeal` (which consumes `HealDealt<Cell>`).
/// Gated by `hazard_active(Cascade)` and `NodeState::Playing`.
pub(crate) fn register(app: &mut App) {
    app.add_systems(
        FixedUpdate,
        cascade_heal_on_death
            .after(DeathPipelineSystems::HandleKill)
            .before(DeathPipelineSystems::ApplyHeal)
            .run_if(hazard_active(HazardKind::Cascade))
            .run_if(in_state(NodeState::Playing)),
    );
}

/// For every `Destroyed<Cell>` message, emit one `HealDealt<Cell>` per
/// living cell within [`ADJACENCY_RADIUS_SQ`] of the victim, with
/// `amount = heal_per_neighbour(stacks)`, `cap = HealCap::Starting`, and
/// `source = Some("hazard:cascade")`. The unified heal pipeline's
/// `apply_heal::<Cell>` applies and clamps.
pub(crate) fn cascade_heal_on_death(
    mut reader: MessageReader<Destroyed<Cell>>,
    active: Res<ActiveHazards>,
    config: Option<Res<CascadeConfig>>,
    cells: Query<(Entity, &Position2D, &Hp), With<Cell>>,
    mut writer: MessageWriter<HealDealt<Cell>>,
) {
    let Some(config) = config else { return };
    let stacks = active.stacks(HazardKind::Cascade);
    let heal = config.heal_per_neighbour(stacks);
    if heal <= 0.0 {
        return;
    }
    let deaths: Vec<(Entity, Vec2)> = reader.read().map(|m| (m.victim, m.victim_pos)).collect();
    if deaths.is_empty() {
        return;
    }
    for (entity, position, hp) in &cells {
        if hp.current <= 0.0 {
            continue;
        }
        for (victim, victim_pos) in &deaths {
            if *victim == entity {
                continue;
            }
            if position.0.distance_squared(*victim_pos) > ADJACENCY_RADIUS_SQ {
                continue;
            }
            writer.write(HealDealt::<Cell> {
                healer:  None,
                target:  entity,
                amount:  heal,
                cap:     HealCap::Starting,
                source:  Some("hazard:cascade".to_string()),
                _marker: PhantomData,
            });
        }
    }
}
